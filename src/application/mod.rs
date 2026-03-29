use crate::domain::model::{BootContext, TurnEvent, TurnEventSink, TurnIntent};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, EvidenceBudget, GathererCapability, ModelPaths,
    ModelRegistry, PlannerConfig, PlannerStrategyKind,
};
use crate::infrastructure::adapters::context1_gatherer::Context1GathererAdapter;
use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use crate::infrastructure::adapters::sift_autonomous_gatherer::SiftAutonomousGathererAdapter;
use crate::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
use anyhow::Result;
use clap::ValueEnum;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    workspace_root: PathBuf,
    registry: Arc<dyn ModelRegistry>,
    runtime: RwLock<Option<ActiveRuntimeState>>,
    verbose: AtomicU8,
    event_sink: Arc<dyn TurnEventSink>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLaneRole {
    Synthesizer,
    Gatherer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub enum GathererProvider {
    Local,
    SiftAutonomous,
    Context1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLaneConfig {
    synthesizer_model_id: String,
    gatherer_model_id: Option<String>,
    gatherer_provider: GathererProvider,
    context1_harness_ready: bool,
}

impl RuntimeLaneConfig {
    pub fn new(synthesizer_model_id: impl Into<String>, gatherer_model_id: Option<String>) -> Self {
        Self {
            synthesizer_model_id: synthesizer_model_id.into(),
            gatherer_model_id,
            gatherer_provider: GathererProvider::SiftAutonomous,
            context1_harness_ready: false,
        }
    }

    pub fn with_gatherer_provider(mut self, gatherer_provider: GathererProvider) -> Self {
        self.gatherer_provider = gatherer_provider;
        self
    }

    pub fn with_context1_harness_ready(mut self, harness_ready: bool) -> Self {
        self.context1_harness_ready = harness_ready;
        self
    }

    pub fn synthesizer_model_id(&self) -> &str {
        &self.synthesizer_model_id
    }

    pub fn gatherer_model_id(&self) -> Option<&str> {
        self.gatherer_model_id.as_deref()
    }

    pub fn gatherer_provider(&self) -> GathererProvider {
        self.gatherer_provider
    }

    pub fn context1_harness_ready(&self) -> bool {
        self.context1_harness_ready
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
pub struct PreparedGathererLane {
    pub provider: GathererProvider,
    pub label: String,
    pub model_id: Option<String>,
    pub paths: Option<ModelPaths>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedRuntimeLanes {
    pub synthesizer: PreparedModelLane,
    pub gatherer: Option<PreparedGathererLane>,
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

#[derive(Default)]
struct ConsoleTurnEventSink {
    render_lock: Mutex<()>,
}

impl TurnEventSink for ConsoleTurnEventSink {
    fn emit(&self, event: TurnEvent) {
        let _guard = self.render_lock.lock().expect("turn event render lock");
        println!("{}", render_turn_event(&event));
    }
}

fn render_turn_event(event: &TurnEvent) -> String {
    match event {
        TurnEvent::IntentClassified { intent } => {
            format!("• Classified turn\n  └ {}", intent.label())
        }
        TurnEvent::RouteSelected { summary } => {
            format!("• Routed turn\n  └ {}", trim_event_detail(summary, 2))
        }
        TurnEvent::GathererCapability {
            provider,
            capability,
        } => {
            format!("• Checked gatherer capability\n  └ {provider}: {capability}")
        }
        TurnEvent::GathererSummary {
            provider,
            summary,
            sources,
        } => {
            let mut lines = vec![
                format!("• Gathered context with {provider}"),
                format!("  └ {}", trim_event_detail(summary, 3)),
            ];
            if !sources.is_empty() {
                lines.push(format!(
                    "    Sources: {}",
                    trim_event_detail(&sources.join(", "), 2)
                ));
            }
            lines.join("\n")
        }
        TurnEvent::PlannerSummary {
            strategy,
            turns,
            steps,
            stop_reason,
        } => format!(
            "• Reviewed planner trace\n  └ strategy={strategy}, turns={turns}, steps={steps}, stop={}",
            stop_reason.as_deref().unwrap_or("none")
        ),
        TurnEvent::ContextAssembly {
            label,
            hits,
            retained_artifacts,
            pruned_artifacts,
        } => format!(
            "• Assembled workspace context ({label})\n  └ {hits} hit(s), retained {retained_artifacts}, pruned {pruned_artifacts}"
        ),
        TurnEvent::ToolCalled {
            tool_name,
            invocation,
            ..
        } => {
            let title = if *tool_name == "shell" {
                "• Ran shell command".to_string()
            } else {
                format!("• Ran {tool_name}")
            };
            format!("{title}\n  └ {}", trim_event_detail(invocation, 3))
        }
        TurnEvent::ToolFinished {
            tool_name, summary, ..
        } => format!(
            "• Completed {tool_name}\n  └ {}",
            trim_event_detail(summary, 6)
        ),
        TurnEvent::Fallback { stage, reason } => {
            format!("• Fell back\n  └ {stage}: {}", trim_event_detail(reason, 3))
        }
        TurnEvent::SynthesisReady {
            grounded,
            citations,
            insufficient_evidence,
        } => {
            if *insufficient_evidence {
                "• Reported insufficient evidence\n  └ No cited repository sources were available."
                    .to_string()
            } else if *grounded {
                format!(
                    "• Synthesized grounded answer\n  └ Sources: {}",
                    if citations.is_empty() {
                        "none".to_string()
                    } else {
                        trim_event_detail(&citations.join(", "), 2)
                    }
                )
            } else {
                "• Synthesized direct answer\n  └ No repository citations required for this turn."
                    .to_string()
            }
        }
    }
}

fn trim_event_detail(input: &str, max_lines: usize) -> String {
    let lines = input
        .lines()
        .take(max_lines)
        .map(str::trim_end)
        .collect::<Vec<_>>();
    if lines.is_empty() {
        return "(no details)".to_string();
    }

    let mut rendered = lines.join("\n    ");
    if input.lines().count() > max_lines {
        rendered.push_str("\n    …");
    }
    rendered
}

impl MechSuitService {
    pub fn new(workspace_root: impl Into<PathBuf>, registry: Arc<dyn ModelRegistry>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            registry,
            runtime: RwLock::new(None),
            verbose: AtomicU8::new(0),
            event_sink: Arc::new(ConsoleTurnEventSink::default()),
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

    fn build_gatherer_lane(
        provider: GathererProvider,
        label: impl Into<String>,
        model_id: Option<String>,
        paths: Option<ModelPaths>,
    ) -> PreparedGathererLane {
        PreparedGathererLane {
            provider,
            label: label.into(),
            model_id,
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

        let (prepared_gatherer, gatherer) = match config.gatherer_provider() {
            GathererProvider::Local => match config.gatherer_model_id() {
                Some(model_id) => {
                    let paths = self.registry.get_model_paths(model_id).await?;
                    let lane = Self::build_gatherer_lane(
                        GathererProvider::Local,
                        model_id,
                        Some(model_id.to_string()),
                        Some(paths),
                    );
                    let adapter =
                        SiftContextGathererAdapter::new(self.workspace_root.clone(), model_id);
                    adapter.set_verbose(self.verbose.load(Ordering::Relaxed));
                    (
                        Some(lane),
                        Some(Arc::new(adapter) as Arc<dyn ContextGatherer>),
                    )
                }
                None => (None, None),
            },
            GathererProvider::SiftAutonomous => {
                let lane = Self::build_gatherer_lane(
                    GathererProvider::SiftAutonomous,
                    "sift-autonomous",
                    None,
                    None,
                );
                let adapter = SiftAutonomousGathererAdapter::new(self.workspace_root.clone());
                adapter.set_verbose(self.verbose.load(Ordering::Relaxed));
                (
                    Some(lane),
                    Some(Arc::new(adapter) as Arc<dyn ContextGatherer>),
                )
            }
            GathererProvider::Context1 => {
                let lane =
                    Self::build_gatherer_lane(GathererProvider::Context1, "context-1", None, None);
                let adapter = Context1GathererAdapter::new(config.context1_harness_ready());
                (
                    Some(lane),
                    Some(Arc::new(adapter) as Arc<dyn ContextGatherer>),
                )
            }
        };

        let prepared = PreparedRuntimeLanes {
            synthesizer,
            gatherer: prepared_gatherer,
        };

        let engine = Arc::new(SiftAgentAdapter::new(
            self.workspace_root.clone(),
            &prepared.synthesizer.model_id,
        )?);
        engine.set_verbose(self.verbose.load(Ordering::Relaxed));

        *self.runtime.write().await = Some(ActiveRuntimeState {
            prepared: prepared.clone(),
            synthesizer_engine: engine,
            gatherer,
        });

        Ok(prepared)
    }

    /// Process a single prompt using the prepared synthesizer lane.
    pub async fn process_prompt(&self, prompt: &str) -> Result<String> {
        self.process_prompt_with_sink(prompt, Arc::clone(&self.event_sink))
            .await
    }

    pub async fn process_prompt_with_sink(
        &self,
        prompt: &str,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let runtime_guard = self.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
        let execution_plan = select_execution_plan(prompt, runtime.gatherer.is_some());
        event_sink.emit(TurnEvent::IntentClassified {
            intent: execution_plan.intent.clone(),
        });

        if let Some(gatherer) = &runtime.prepared.gatherer {
            match execution_plan.path {
                PromptExecutionPath::GatherThenSynthesize => event_sink.emit(
                    TurnEvent::RouteSelected {
                        summary: format!(
                            "repository question will use gatherer lane '{}' ({:?}) before synthesizer lane '{}'",
                            gatherer.label,
                            gatherer.provider,
                            runtime.prepared.synthesizer.model_id
                        ),
                    },
                ),
                PromptExecutionPath::SynthesizerOnly => event_sink.emit(TurnEvent::RouteSelected {
                    summary: format!(
                        "turn will stay on synthesizer lane '{}' while gatherer lane '{}' ({:?}) remains available",
                        runtime.prepared.synthesizer.model_id,
                        gatherer.label,
                        gatherer.provider
                    ),
                }),
            }
        } else {
            event_sink.emit(TurnEvent::RouteSelected {
                summary: format!(
                    "turn will use synthesizer lane '{}' with no gatherer lane configured",
                    runtime.prepared.synthesizer.model_id
                ),
            });
        }

        let gathered_evidence = match execution_plan.path {
            PromptExecutionPath::GatherThenSynthesize => match runtime.gatherer.as_ref() {
                Some(gatherer) => {
                    let capability = gatherer.capability();
                    let provider = runtime
                        .prepared
                        .gatherer
                        .as_ref()
                        .map(|lane| lane.label.clone())
                        .unwrap_or_else(|| "gatherer".to_string());
                    event_sink.emit(TurnEvent::GathererCapability {
                        provider: provider.clone(),
                        capability: format_gatherer_capability(&capability),
                    });

                    match capability {
                        GathererCapability::Available => {
                            let gather_query = build_gather_query(prompt, &execution_plan.intent);
                            let planning = match execution_plan.intent {
                                TurnIntent::DecompositionResearch => {
                                    PlannerConfig::default().with_step_limit(4)
                                }
                                _ => PlannerConfig::default(),
                            };
                            let request = ContextGatherRequest::new(
                                gather_query,
                                self.workspace_root.clone(),
                                execution_plan.intent.label(),
                                EvidenceBudget::default(),
                            )
                            .with_planning(planning);
                            match gatherer.gather_context(&request).await {
                                Ok(result) if result.is_synthesis_ready() => {
                                    if let Some(bundle) = result.evidence_bundle.as_ref() {
                                        event_sink.emit(TurnEvent::GathererSummary {
                                            provider: provider.clone(),
                                            summary: bundle.summary.clone(),
                                            sources: evidence_sources(&self.workspace_root, bundle),
                                        });
                                        if let Some(planner) = bundle.planner.as_ref() {
                                            event_sink.emit(TurnEvent::PlannerSummary {
                                                strategy: format_planner_strategy(
                                                    &planner.strategy,
                                                )
                                                .to_string(),
                                                turns: planner.turn_count,
                                                steps: planner.steps.len(),
                                                stop_reason: planner.stop_reason.clone(),
                                            });
                                        }
                                    }
                                    result.evidence_bundle
                                }
                                Ok(result) => {
                                    event_sink.emit(TurnEvent::Fallback {
                                        stage: "gatherer".to_string(),
                                        reason: format!(
                                            "gatherer returned non-synthesis-ready result ({})",
                                            format_gatherer_capability(&result.capability)
                                        ),
                                    });
                                    None
                                }
                                Err(err) => {
                                    event_sink.emit(TurnEvent::Fallback {
                                        stage: "gatherer".to_string(),
                                        reason: format!(
                                            "gatherer lane failed ({err:#}); switching to explicit fallback"
                                        ),
                                    });
                                    None
                                }
                            }
                        }
                        GathererCapability::Unsupported { reason }
                        | GathererCapability::HarnessRequired { reason } => {
                            event_sink.emit(TurnEvent::Fallback {
                                stage: "gatherer".to_string(),
                                reason: format!(
                                    "gatherer unavailable for a repository question: {reason}"
                                ),
                            });
                            None
                        }
                    }
                }
                None => None,
            },
            PromptExecutionPath::SynthesizerOnly => None,
        };

        let prompt = prompt.to_string();
        let intent = execution_plan.intent;
        let engine = runtime.synthesizer_engine.clone();
        let event_sink = Arc::clone(&event_sink);
        tokio::task::spawn_blocking(move || {
            engine.respond_for_turn(&prompt, intent, gathered_evidence.as_ref(), event_sink)
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))?
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromptExecutionPlan {
    intent: TurnIntent,
    path: PromptExecutionPath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptExecutionPath {
    SynthesizerOnly,
    GatherThenSynthesize,
}

fn select_execution_plan(prompt: &str, gatherer_available: bool) -> PromptExecutionPlan {
    let intent = classify_turn(prompt);
    let path = if gatherer_available && intent.requires_gathered_evidence() {
        PromptExecutionPath::GatherThenSynthesize
    } else {
        PromptExecutionPath::SynthesizerOnly
    };

    PromptExecutionPlan { intent, path }
}

fn classify_turn(prompt: &str) -> TurnIntent {
    let normalized = normalize_turn_prompt(prompt);

    if is_casual_turn(&normalized) {
        return TurnIntent::Casual;
    }
    if is_deterministic_action_turn(&normalized) {
        return TurnIntent::DeterministicAction;
    }
    if is_decomposition_research_turn(&normalized) {
        return TurnIntent::DecompositionResearch;
    }
    if is_repository_question_turn(&normalized) {
        return TurnIntent::RepositoryQuestion;
    }

    TurnIntent::GeneralQuestion
}

fn is_casual_turn(normalized: &str) -> bool {
    matches!(
        normalized,
        "hi" | "hello"
            | "hey"
            | "howdy"
            | "yo"
            | "sup"
            | "whats up"
            | "what's up"
            | "how are you"
            | "who are you"
            | "thanks"
            | "thank you"
            | "good morning"
            | "good afternoon"
            | "good evening"
            | "bye"
            | "goodbye"
    ) || normalized.starts_with("hello ")
        || normalized.starts_with("hi ")
        || normalized.starts_with("hey ")
        || normalized.starts_with("howdy ")
        || normalized.starts_with("thanks ")
        || normalized.starts_with("thank you ")
}

fn is_deterministic_action_turn(normalized: &str) -> bool {
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
    direct_action
        .iter()
        .any(|needle| normalized.contains(needle))
        || normalized.starts_with("list ")
        || normalized.starts_with("open ")
        || normalized.starts_with("read ")
        || normalized.starts_with("edit ")
        || normalized.starts_with("replace ")
        || normalized.starts_with("write ")
}

fn is_decomposition_research_turn(normalized: &str) -> bool {
    let decomposition_markers = [
        "walk through",
        "trace",
        "dependency",
        "dependencies",
        "path from",
        "flow through",
        "across the repo",
        "across the codebase",
        "end-to-end",
        "research",
        "compare",
        "what references",
        "which files",
        "which modules",
    ];

    decomposition_markers
        .iter()
        .any(|needle| normalized.contains(needle))
}

fn is_repository_question_turn(normalized: &str) -> bool {
    let repo_markers = [
        "paddles",
        "repo",
        "repository",
        "codebase",
        "workspace",
        "module",
        "file",
        "runtime",
        "lane",
        "gatherer",
        "synthesizer",
        "context gathering",
        "architecture",
        "memory",
        "agents.md",
        "keel",
        "turn",
    ];

    repo_markers
        .iter()
        .any(|needle| normalized.contains(needle))
}

fn normalize_turn_prompt(prompt: &str) -> String {
    prompt
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_ascii_lowercase()
}

fn build_gather_query(prompt: &str, intent: &TurnIntent) -> String {
    if !intent.requires_gathered_evidence() {
        return prompt.to_string();
    }

    let normalized = prompt.to_ascii_lowercase();
    let mut terms = normalized
        .split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '.')
        .filter(|term| !term.is_empty())
        .filter(|term| {
            !matches!(
                *term,
                "how"
                    | "does"
                    | "what"
                    | "is"
                    | "are"
                    | "why"
                    | "where"
                    | "the"
                    | "a"
                    | "an"
                    | "in"
                    | "on"
                    | "of"
                    | "to"
                    | "for"
                    | "from"
                    | "and"
                    | "across"
                    | "through"
                    | "work"
                    | "works"
            )
        })
        .map(str::to_string)
        .collect::<Vec<_>>();

    if normalized.contains("memory") {
        terms.extend([
            "agents.md".to_string(),
            "agent_memory".to_string(),
            "reload".to_string(),
        ]);
    }
    if normalized.contains("routing") || normalized.contains("lane") {
        terms.extend([
            "runtime".to_string(),
            "gatherer".to_string(),
            "synthesizer".to_string(),
        ]);
    }
    if normalized.contains("context") {
        terms.extend([
            "evidence".to_string(),
            "gatherer".to_string(),
            "search".to_string(),
        ]);
    }
    if matches!(intent, TurnIntent::DecompositionResearch) {
        terms.extend([
            "trace".to_string(),
            "flow".to_string(),
            "dependencies".to_string(),
        ]);
    }

    terms.extend(["implementation".to_string(), "source".to_string()]);
    terms.sort();
    terms.dedup();

    if terms.is_empty() {
        prompt.to_string()
    } else {
        terms.join(" ")
    }
}

fn format_gatherer_capability(capability: &GathererCapability) -> String {
    match capability {
        GathererCapability::Available => "available".to_string(),
        GathererCapability::Unsupported { reason } => format!("unsupported: {reason}"),
        GathererCapability::HarnessRequired { reason } => {
            format!("harness-required: {reason}")
        }
    }
}

fn format_planner_strategy(strategy: &PlannerStrategyKind) -> &'static str {
    match strategy {
        PlannerStrategyKind::Heuristic => "heuristic",
        PlannerStrategyKind::ModelDriven => "model-driven",
    }
}

fn evidence_sources(
    workspace_root: &std::path::Path,
    bundle: &crate::domain::ports::EvidenceBundle,
) -> Vec<String> {
    let mut sources = Vec::new();
    for item in &bundle.items {
        let source = normalize_event_source(workspace_root, &item.source);
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    if sources.len() > 4 {
        sources.truncate(4);
    }
    sources
}

fn normalize_event_source(workspace_root: &std::path::Path, source: &str) -> String {
    let source_path = std::path::Path::new(source);
    if source_path.is_absolute()
        && let Ok(relative) = source_path.strip_prefix(workspace_root)
    {
        return relative.display().to_string();
    }

    source.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        GathererProvider, MechSuitService, PreparedRuntimeLanes, RuntimeLaneConfig,
        RuntimeLaneRole, TurnIntent,
    };
    use crate::domain::ports::ModelPaths;
    use std::path::PathBuf;

    #[test]
    fn runtime_lane_config_defaults_to_synthesizer_responses() {
        let config = RuntimeLaneConfig::new("qwen-1.5b", None);

        assert_eq!(config.default_response_role(), RuntimeLaneRole::Synthesizer);
        assert_eq!(config.synthesizer_model_id(), "qwen-1.5b");
        assert_eq!(config.gatherer_model_id(), None);
        assert_eq!(config.gatherer_provider(), GathererProvider::SiftAutonomous);
        assert!(!config.context1_harness_ready());
    }

    #[test]
    fn prepared_runtime_lanes_keep_synthesizer_as_default_response_lane() {
        let synthesizer = MechSuitService::build_lane(
            RuntimeLaneRole::Synthesizer,
            "qwen-1.5b",
            sample_model_paths("synth"),
        );
        let gatherer = MechSuitService::build_gatherer_lane(
            GathererProvider::Local,
            "qwen-7b",
            Some("qwen-7b".to_string()),
            Some(sample_model_paths("gather")),
        );
        let lanes = PreparedRuntimeLanes {
            synthesizer: synthesizer.clone(),
            gatherer: Some(gatherer.clone()),
        };

        assert_eq!(lanes.default_response_lane(), &synthesizer);
        assert_eq!(lanes.gatherer.as_ref(), Some(&gatherer));
    }

    #[test]
    fn context1_boundary_can_be_prepared_without_local_model_paths() {
        let gatherer = MechSuitService::build_gatherer_lane(
            GathererProvider::Context1,
            "context-1",
            None,
            None,
        );

        assert_eq!(gatherer.provider, GathererProvider::Context1);
        assert_eq!(gatherer.label, "context-1");
        assert_eq!(gatherer.model_id, None);
        assert_eq!(gatherer.paths, None);
    }

    #[test]
    fn sift_autonomous_boundary_can_be_prepared_without_local_model_paths() {
        let gatherer = MechSuitService::build_gatherer_lane(
            GathererProvider::SiftAutonomous,
            "sift-autonomous",
            None,
            None,
        );

        assert_eq!(gatherer.provider, GathererProvider::SiftAutonomous);
        assert_eq!(gatherer.label, "sift-autonomous");
        assert_eq!(gatherer.model_id, None);
        assert_eq!(gatherer.paths, None);
    }

    #[test]
    fn retrieval_heavy_prompts_use_gatherer_lane_when_available() {
        let plan = super::select_execution_plan(
            "Summarize the runtime lane architecture across the repo",
            true,
        );

        assert_eq!(plan.intent, TurnIntent::DecompositionResearch);
        assert_eq!(plan.path, super::PromptExecutionPath::GatherThenSynthesize);
    }

    #[test]
    fn decomposition_worthy_prompts_use_gatherer_lane_when_available() {
        let plan =
            super::select_execution_plan("Trace the runtime lane architecture end-to-end", true);

        assert_eq!(plan.intent, TurnIntent::DecompositionResearch);
        assert_eq!(plan.path, super::PromptExecutionPath::GatherThenSynthesize);
    }

    #[test]
    fn repository_questions_use_gatherer_lane_when_available() {
        let plan = super::select_execution_plan("How does memory work in paddles?", true);

        assert_eq!(plan.intent, TurnIntent::RepositoryQuestion);
        assert_eq!(plan.path, super::PromptExecutionPath::GatherThenSynthesize);
    }

    #[test]
    fn action_or_casual_prompts_stay_on_synthesizer_lane() {
        assert_eq!(
            super::select_execution_plan("Show me the git status", true).path,
            super::PromptExecutionPath::SynthesizerOnly
        );
        assert_eq!(
            super::select_execution_plan("Hello", true).path,
            super::PromptExecutionPath::SynthesizerOnly
        );
        assert_eq!(
            super::select_execution_plan("Howdy", true).intent,
            TurnIntent::Casual
        );
        assert_eq!(
            super::select_execution_plan("What is a monad?", true).path,
            super::PromptExecutionPath::SynthesizerOnly
        );
    }

    #[test]
    fn decomposition_prompts_without_a_gatherer_lane_stay_on_synthesizer() {
        assert_eq!(
            super::select_execution_plan("Trace the runtime lane architecture end-to-end", false)
                .path,
            super::PromptExecutionPath::SynthesizerOnly
        );
    }

    #[test]
    fn repository_queries_are_normalized_for_gathering() {
        let query = super::build_gather_query(
            "How does memory work in paddles?",
            &TurnIntent::RepositoryQuestion,
        );

        assert!(query.contains("memory"));
        assert!(query.contains("agents.md"));
        assert!(query.contains("agent_memory"));
        assert!(query.contains("implementation"));
    }

    fn sample_model_paths(prefix: &str) -> ModelPaths {
        ModelPaths {
            weights: PathBuf::from(format!("{prefix}-weights.safetensors")),
            tokenizer: PathBuf::from(format!("{prefix}-tokenizer.json")),
            config: PathBuf::from(format!("{prefix}-config.json")),
        }
    }
}
