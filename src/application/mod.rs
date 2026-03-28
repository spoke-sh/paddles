use crate::domain::model::BootContext;
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, EvidenceBudget, ModelPaths, ModelRegistry,
};
use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use crate::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU8, Ordering};
use tokio::sync::RwLock;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    workspace_root: PathBuf,
    registry: Arc<dyn ModelRegistry>,
    runtime: RwLock<Option<ActiveRuntimeState>>,
    verbose: AtomicU8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLaneRole {
    Synthesizer,
    Gatherer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLaneConfig {
    synthesizer_model_id: String,
    gatherer_model_id: Option<String>,
}

impl RuntimeLaneConfig {
    pub fn new(synthesizer_model_id: impl Into<String>, gatherer_model_id: Option<String>) -> Self {
        Self {
            synthesizer_model_id: synthesizer_model_id.into(),
            gatherer_model_id,
        }
    }

    pub fn synthesizer_model_id(&self) -> &str {
        &self.synthesizer_model_id
    }

    pub fn gatherer_model_id(&self) -> Option<&str> {
        self.gatherer_model_id.as_deref()
    }

    pub fn default_response_role(&self) -> RuntimeLaneRole {
        RuntimeLaneRole::Synthesizer
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedModelLane {
    pub role: RuntimeLaneRole,
    pub model_id: String,
    pub paths: ModelPaths,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedRuntimeLanes {
    pub synthesizer: PreparedModelLane,
    pub gatherer: Option<PreparedModelLane>,
}

impl PreparedRuntimeLanes {
    pub fn default_response_lane(&self) -> &PreparedModelLane {
        &self.synthesizer
    }
}

struct ActiveRuntimeState {
    prepared: PreparedRuntimeLanes,
    synthesizer_engine: Arc<SiftAgentAdapter>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
}

impl MechSuitService {
    pub fn new(workspace_root: impl Into<PathBuf>, registry: Arc<dyn ModelRegistry>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            registry,
            runtime: RwLock::new(None),
            verbose: AtomicU8::new(0),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    /// Execute the boot sequence.
    pub fn boot(
        &self,
        credits: u64,
        weight: f64,
        bias: f64,
        hf_token: Option<String>,
        reality_mode: bool,
    ) -> Result<BootContext> {
        BootContext::new(credits, weight, bias, hf_token, reality_mode)
    }

    fn build_lane(
        role: RuntimeLaneRole,
        model_id: impl Into<String>,
        paths: ModelPaths,
    ) -> PreparedModelLane {
        PreparedModelLane {
            role,
            model_id: model_id.into(),
            paths,
        }
    }

    /// Prepare the configured runtime lanes for inference.
    pub async fn prepare_runtime_lanes(
        &self,
        config: &RuntimeLaneConfig,
    ) -> Result<PreparedRuntimeLanes> {
        let synthesizer_paths = self
            .registry
            .get_model_paths(config.synthesizer_model_id())
            .await?;
        let synthesizer = Self::build_lane(
            RuntimeLaneRole::Synthesizer,
            config.synthesizer_model_id(),
            synthesizer_paths,
        );

        let gatherer = if let Some(model_id) = config.gatherer_model_id() {
            let paths = self.registry.get_model_paths(model_id).await?;
            Some(Self::build_lane(RuntimeLaneRole::Gatherer, model_id, paths))
        } else {
            None
        };

        let prepared = PreparedRuntimeLanes {
            synthesizer,
            gatherer,
        };

        let engine = Arc::new(SiftAgentAdapter::new(
            self.workspace_root.clone(),
            &prepared.synthesizer.model_id,
        )?);
        engine.set_verbose(self.verbose.load(Ordering::Relaxed));
        let gatherer = prepared.gatherer.as_ref().map(|lane| {
            let adapter =
                SiftContextGathererAdapter::new(self.workspace_root.clone(), lane.model_id.clone());
            adapter.set_verbose(self.verbose.load(Ordering::Relaxed));
            Arc::new(adapter) as Arc<dyn ContextGatherer>
        });
        *self.runtime.write().await = Some(ActiveRuntimeState {
            prepared: prepared.clone(),
            synthesizer_engine: engine,
            gatherer,
        });

        Ok(prepared)
    }

    /// Process a single prompt using the prepared synthesizer lane.
    pub async fn process_prompt(&self, prompt: &str) -> Result<String> {
        let runtime_guard = self.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
        let routing = select_execution_path(prompt, runtime.gatherer.is_some());

        if self.verbose.load(Ordering::Relaxed) >= 1 {
            if let Some(gatherer) = &runtime.prepared.gatherer {
                match routing {
                    PromptExecutionPath::GatherThenSynthesize => println!(
                        "[LANE] Routing retrieval-heavy prompt through gatherer lane '{}' before synthesizer lane '{}'.",
                        gatherer.model_id, runtime.prepared.synthesizer.model_id,
                    ),
                    PromptExecutionPath::SynthesizerOnly => println!(
                        "[LANE] Using synthesizer lane '{}' for this prompt; gatherer lane '{}' is available but not selected.",
                        runtime.prepared.synthesizer.model_id, gatherer.model_id,
                    ),
                }
            } else {
                println!(
                    "[LANE] Using synthesizer lane '{}' as the default response path.",
                    runtime.prepared.synthesizer.model_id,
                );
            }
        }

        let gathered_evidence = match routing {
            PromptExecutionPath::GatherThenSynthesize => match runtime.gatherer.as_ref() {
                Some(gatherer) => {
                    let request = ContextGatherRequest::new(
                        prompt,
                        self.workspace_root.clone(),
                        "retrieval-heavy prompt routed through the gatherer lane",
                        EvidenceBudget::default(),
                    );
                    match gatherer.gather_context(&request).await {
                        Ok(result) if result.is_synthesis_ready() => result.evidence_bundle,
                        Ok(_) => {
                            if self.verbose.load(Ordering::Relaxed) >= 1 {
                                println!(
                                    "[LANE] Gatherer lane was unavailable for synthesis-ready evidence; falling back to the synthesizer lane."
                                );
                            }
                            None
                        }
                        Err(err) => {
                            if self.verbose.load(Ordering::Relaxed) >= 1 {
                                println!(
                                    "[LANE] Gatherer lane failed ({err:#}); falling back to the synthesizer lane."
                                );
                            }
                            None
                        }
                    }
                }
                None => None,
            },
            PromptExecutionPath::SynthesizerOnly => None,
        };

        let prompt = prompt.to_string();
        let engine = runtime.synthesizer_engine.clone();
        tokio::task::spawn_blocking(move || {
            engine.respond_with_evidence(&prompt, gathered_evidence.as_ref())
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))?
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptExecutionPath {
    SynthesizerOnly,
    GatherThenSynthesize,
}

fn select_execution_path(prompt: &str, gatherer_available: bool) -> PromptExecutionPath {
    if gatherer_available && should_route_through_context_gathering(prompt) {
        PromptExecutionPath::GatherThenSynthesize
    } else {
        PromptExecutionPath::SynthesizerOnly
    }
}

fn should_route_through_context_gathering(prompt: &str) -> bool {
    let normalized = prompt.to_ascii_lowercase();
    let direct_action = [
        "git status",
        "git diff",
        "run ",
        "show ",
        "check ",
        "inspect ",
        "open ",
        "read ",
        "edit ",
        "replace ",
        "write ",
        "apply ",
    ];
    if direct_action
        .iter()
        .any(|needle| normalized.contains(needle))
    {
        return false;
    }

    let retrieval_markers = [
        "summarize",
        "context",
        "architecture",
        "explain how",
        "across the repo",
        "across the codebase",
        "research",
        "compare",
        "what references",
        "where is the context",
    ];

    retrieval_markers
        .iter()
        .any(|needle| normalized.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{MechSuitService, PreparedRuntimeLanes, RuntimeLaneConfig, RuntimeLaneRole};
    use crate::domain::ports::ModelPaths;
    use std::path::PathBuf;

    #[test]
    fn runtime_lane_config_defaults_to_synthesizer_responses() {
        let config = RuntimeLaneConfig::new("qwen-1.5b", None);

        assert_eq!(config.default_response_role(), RuntimeLaneRole::Synthesizer);
        assert_eq!(config.synthesizer_model_id(), "qwen-1.5b");
        assert_eq!(config.gatherer_model_id(), None);
    }

    #[test]
    fn prepared_runtime_lanes_keep_synthesizer_as_default_response_lane() {
        let synthesizer = MechSuitService::build_lane(
            RuntimeLaneRole::Synthesizer,
            "qwen-1.5b",
            sample_model_paths("synth"),
        );
        let gatherer = MechSuitService::build_lane(
            RuntimeLaneRole::Gatherer,
            "qwen-7b",
            sample_model_paths("gather"),
        );
        let lanes = PreparedRuntimeLanes {
            synthesizer: synthesizer.clone(),
            gatherer: Some(gatherer.clone()),
        };

        assert_eq!(lanes.default_response_lane(), &synthesizer);
        assert_eq!(lanes.gatherer.as_ref(), Some(&gatherer));
    }

    #[test]
    fn retrieval_heavy_prompts_use_gatherer_lane_when_available() {
        let routing = super::select_execution_path(
            "Summarize the runtime lane architecture across the repo",
            true,
        );

        assert_eq!(routing, super::PromptExecutionPath::GatherThenSynthesize);
    }

    #[test]
    fn action_or_casual_prompts_stay_on_synthesizer_lane() {
        assert_eq!(
            super::select_execution_path("Show me the git status", true),
            super::PromptExecutionPath::SynthesizerOnly,
        );
        assert_eq!(
            super::select_execution_path("Hello", true),
            super::PromptExecutionPath::SynthesizerOnly,
        );
        assert_eq!(
            super::select_execution_path(
                "Summarize the runtime lane architecture across the repo",
                false,
            ),
            super::PromptExecutionPath::SynthesizerOnly,
        );
    }

    fn sample_model_paths(prefix: &str) -> ModelPaths {
        ModelPaths {
            weights: PathBuf::from(format!("{prefix}-weights.safetensors")),
            tokenizer: PathBuf::from(format!("{prefix}-tokenizer.json")),
            config: PathBuf::from(format!("{prefix}-config.json")),
        }
    }
}
