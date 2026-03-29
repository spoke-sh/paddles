use crate::domain::model::{BootContext, TurnEvent, TurnEventSink, TurnIntent};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, EvidenceBudget, EvidenceBundle, EvidenceItem,
    GathererCapability, InterpretationContext, ModelPaths, ModelRegistry, PlannerAction,
    PlannerBudget, PlannerCapability, PlannerConfig, PlannerLoopState, PlannerRequest,
    PlannerStepRecord, PlannerStrategyKind, PlannerTraceMetadata, PlannerTraceStep,
    RecursivePlanner, RetainedEvidence,
};
use crate::infrastructure::adapters::agent_memory::AgentMemory;
use crate::infrastructure::adapters::context1_gatherer::Context1GathererAdapter;
use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use crate::infrastructure::adapters::sift_autonomous_gatherer::SiftAutonomousGathererAdapter;
use crate::infrastructure::adapters::sift_context_gatherer::SiftContextGathererAdapter;
use crate::infrastructure::adapters::sift_planner::SiftPlannerAdapter;
use anyhow::Result;
use clap::ValueEnum;
use std::fs;
use std::path::Path;
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
    Planner,
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
    planner_model_id: Option<String>,
    synthesizer_model_id: String,
    gatherer_model_id: Option<String>,
    gatherer_provider: GathererProvider,
    context1_harness_ready: bool,
}

impl RuntimeLaneConfig {
    pub fn new(synthesizer_model_id: impl Into<String>, gatherer_model_id: Option<String>) -> Self {
        Self {
            planner_model_id: None,
            synthesizer_model_id: synthesizer_model_id.into(),
            gatherer_model_id,
            gatherer_provider: GathererProvider::SiftAutonomous,
            context1_harness_ready: false,
        }
    }

    pub fn with_planner_model_id(mut self, planner_model_id: Option<String>) -> Self {
        self.planner_model_id = planner_model_id;
        self
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

    pub fn planner_model_id(&self) -> Option<&str> {
        self.planner_model_id.as_deref()
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
    pub planner: PreparedModelLane,
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
    planner_engine: Arc<dyn RecursivePlanner>,
    synthesizer_engine: Arc<SiftAgentAdapter>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
}

struct PlannerLoopContext {
    prepared: PreparedRuntimeLanes,
    planner_engine: Arc<dyn RecursivePlanner>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
    interpretation: InterpretationContext,
    recent_turns: Vec<String>,
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
        TurnEvent::InterpretationContext { summary, sources } => {
            let mut lines = vec![
                "• Assembled interpretation context".to_string(),
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
        TurnEvent::RouteSelected { summary } => {
            format!("• Routed turn\n  └ {}", trim_event_detail(summary, 2))
        }
        TurnEvent::PlannerCapability {
            provider,
            capability,
        } => {
            format!("• Checked planner capability\n  └ {provider}: {capability}")
        }
        TurnEvent::GathererCapability {
            provider,
            capability,
        } => {
            format!("• Checked gatherer capability\n  └ {provider}: {capability}")
        }
        TurnEvent::PlannerActionSelected {
            sequence,
            action,
            rationale,
        } => format!(
            "• Selected planner action\n  └ step {sequence}: {}\n    Rationale: {}",
            trim_event_detail(action, 2),
            trim_event_detail(rationale, 2)
        ),
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
        let planner_model_id = config
            .planner_model_id()
            .unwrap_or(config.synthesizer_model_id())
            .to_string();
        let planner_paths = if planner_model_id == config.synthesizer_model_id() {
            synthesizer_paths.clone()
        } else {
            self.registry.get_model_paths(&planner_model_id).await?
        };
        let planner = Self::build_lane(RuntimeLaneRole::Planner, &planner_model_id, planner_paths);
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
            planner,
            synthesizer,
            gatherer: prepared_gatherer,
        };

        let engine = Arc::new(SiftAgentAdapter::new(
            self.workspace_root.clone(),
            &prepared.synthesizer.model_id,
        )?);
        engine.set_verbose(self.verbose.load(Ordering::Relaxed));
        let planner_engine: Arc<dyn RecursivePlanner> =
            if prepared.planner.model_id == prepared.synthesizer.model_id {
                Arc::new(SiftPlannerAdapter::new(Arc::clone(&engine)))
            } else {
                let planner_model = Arc::new(SiftAgentAdapter::new(
                    self.workspace_root.clone(),
                    &prepared.planner.model_id,
                )?);
                planner_model.set_verbose(self.verbose.load(Ordering::Relaxed));
                Arc::new(SiftPlannerAdapter::new(planner_model))
            };

        *self.runtime.write().await = Some(ActiveRuntimeState {
            prepared: prepared.clone(),
            planner_engine,
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
        let prepared = runtime.prepared.clone();
        let planner_engine = Arc::clone(&runtime.planner_engine);
        let synthesizer_engine = Arc::clone(&runtime.synthesizer_engine);
        let gatherer = runtime.gatherer.clone();
        drop(runtime_guard);

        let execution_plan = select_execution_plan(prompt, true);
        event_sink.emit(TurnEvent::IntentClassified {
            intent: execution_plan.intent.clone(),
        });

        if let Some(gatherer_lane) = &prepared.gatherer {
            match execution_plan.path {
                PromptExecutionPath::PlannerThenSynthesize => event_sink.emit(
                    TurnEvent::RouteSelected {
                        summary: format!(
                            "turn will use planner lane '{}' with gatherer backend '{}' ({:?}) before synthesizer lane '{}'",
                            prepared.planner.model_id,
                            gatherer_lane.label,
                            gatherer_lane.provider,
                            prepared.synthesizer.model_id
                        ),
                    },
                ),
                PromptExecutionPath::SynthesizerOnly => event_sink.emit(TurnEvent::RouteSelected {
                    summary: format!(
                        "turn will stay on synthesizer lane '{}' while planner lane '{}' and gatherer backend '{}' remain available",
                        prepared.synthesizer.model_id,
                        prepared.planner.model_id,
                        gatherer_lane.label,
                    ),
                }),
            }
        } else {
            event_sink.emit(TurnEvent::RouteSelected {
                summary: format!(
                    "turn will use planner lane '{}' and synthesizer lane '{}' with no dedicated gatherer backend configured",
                    prepared.planner.model_id,
                    prepared.synthesizer.model_id
                ),
            });
        }

        let gathered_evidence = match execution_plan.path {
            PromptExecutionPath::PlannerThenSynthesize => {
                let memory = AgentMemory::load(&self.workspace_root);
                let interpretation =
                    memory.build_interpretation_context(prompt, &self.workspace_root);
                event_sink.emit(TurnEvent::InterpretationContext {
                    summary: interpretation.summary.clone(),
                    sources: interpretation.sources(),
                });

                let planner_capability = planner_engine.capability();
                event_sink.emit(TurnEvent::PlannerCapability {
                    provider: prepared.planner.model_id.clone(),
                    capability: format_planner_capability(&planner_capability),
                });

                match planner_capability {
                    PlannerCapability::Available => {
                        let recent_turns = synthesizer_engine.recent_turn_summaries()?;
                        self.execute_recursive_planner_loop(
                            prompt,
                            PlannerLoopContext {
                                prepared: prepared.clone(),
                                planner_engine,
                                gatherer,
                                interpretation,
                                recent_turns,
                            },
                            Arc::clone(&event_sink),
                        )
                        .await?
                    }
                    PlannerCapability::Unsupported { reason } => {
                        event_sink.emit(TurnEvent::Fallback {
                            stage: "planner".to_string(),
                            reason: format!("planner unavailable: {reason}"),
                        });
                        None
                    }
                }
            }
            PromptExecutionPath::SynthesizerOnly => None,
        };

        let prompt = prompt.to_string();
        let intent = execution_plan.intent;
        let engine = synthesizer_engine;
        let event_sink = Arc::clone(&event_sink);
        tokio::task::spawn_blocking(move || {
            engine.respond_for_turn(&prompt, intent, gathered_evidence.as_ref(), event_sink)
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))?
    }

    async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<Option<EvidenceBundle>> {
        let budget = PlannerBudget::default();
        let mut loop_state = PlannerLoopState::default();
        let mut used_workspace_resources = false;
        let mut stop_reason = None;
        let gatherer_provider = context
            .prepared
            .gatherer
            .as_ref()
            .map(|lane| lane.label.clone())
            .unwrap_or_else(|| "workspace".to_string());

        for sequence in 1..=budget.max_steps {
            let request = PlannerRequest::new(
                prompt,
                self.workspace_root.clone(),
                context.interpretation.clone(),
                budget.clone(),
            )
            .with_recent_turns(context.recent_turns.clone())
            .with_loop_state(loop_state.clone());
            let decision = context.planner_engine.select_next_action(&request).await?;
            event_sink.emit(TurnEvent::PlannerActionSelected {
                sequence,
                action: decision.action.summary(),
                rationale: decision.rationale.clone(),
            });

            let outcome = match &decision.action {
                PlannerAction::Search { query, intent } => {
                    if let Some(gatherer) = context.gatherer.as_ref() {
                        event_sink.emit(TurnEvent::GathererCapability {
                            provider: gatherer_provider.clone(),
                            capability: format_gatherer_capability(&gatherer.capability()),
                        });
                        match gatherer.capability() {
                            GathererCapability::Available => {
                                let request = ContextGatherRequest::new(
                                    query.clone(),
                                    self.workspace_root.clone(),
                                    intent
                                        .clone()
                                        .unwrap_or_else(|| "planner-search".to_string()),
                                    EvidenceBudget::default(),
                                )
                                .with_planning(PlannerConfig::default().with_step_limit(1))
                                .with_prior_context(
                                    build_planner_prior_context(
                                        &context.interpretation,
                                        &context.recent_turns,
                                        &loop_state,
                                    ),
                                );
                                match gatherer.gather_context(&request).await {
                                    Ok(result) => {
                                        let bundle = result.evidence_bundle;
                                        if let Some(bundle) = bundle.as_ref() {
                                            event_sink.emit(TurnEvent::GathererSummary {
                                                provider: gatherer_provider.clone(),
                                                summary: bundle.summary.clone(),
                                                sources: evidence_sources(
                                                    &self.workspace_root,
                                                    bundle,
                                                ),
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
                                            merge_evidence_items(
                                                &mut loop_state.evidence_items,
                                                bundle.items.clone(),
                                                budget.max_evidence_items,
                                            );
                                            loop_state.notes.extend(bundle.warnings.clone());
                                            used_workspace_resources = true;
                                            bundle.summary.clone()
                                        } else {
                                            "planner search returned no evidence bundle".to_string()
                                        }
                                    }
                                    Err(err) => format!("planner search failed: {err:#}"),
                                }
                            }
                            GathererCapability::Unsupported { reason }
                            | GathererCapability::HarnessRequired { reason } => {
                                format!("planner search backend unavailable: {reason}")
                            }
                        }
                    } else {
                        "no gatherer backend is configured for planner search".to_string()
                    }
                }
                PlannerAction::Read { path } => {
                    if read_steps(&loop_state) >= budget.max_reads {
                        stop_reason = Some("read-budget-exhausted".to_string());
                        "planner read budget exhausted".to_string()
                    } else {
                        let (source, snippet) = read_planner_file(&self.workspace_root, path)?;
                        append_evidence_item(
                            &mut loop_state.evidence_items,
                            EvidenceItem {
                                source: source.clone(),
                                snippet,
                                rationale: decision.rationale.clone(),
                                rank: 0,
                            },
                            budget.max_evidence_items,
                        );
                        used_workspace_resources = true;
                        format!("read {source}")
                    }
                }
                PlannerAction::Inspect { command } => {
                    if inspect_steps(&loop_state) >= budget.max_inspects {
                        stop_reason = Some("inspect-budget-exhausted".to_string());
                        "planner inspect budget exhausted".to_string()
                    } else {
                        let output = run_planner_inspect_command(&self.workspace_root, command)?;
                        append_evidence_item(
                            &mut loop_state.evidence_items,
                            EvidenceItem {
                                source: format!("command: {command}"),
                                snippet: trim_for_planner(&output, 800),
                                rationale: decision.rationale.clone(),
                                rank: 0,
                            },
                            budget.max_evidence_items,
                        );
                        used_workspace_resources = true;
                        format!("inspected {command}")
                    }
                }
                PlannerAction::Refine { query, .. } => {
                    if let Some(gatherer) = context.gatherer.as_ref() {
                        event_sink.emit(TurnEvent::GathererCapability {
                            provider: gatherer_provider.clone(),
                            capability: format_gatherer_capability(&gatherer.capability()),
                        });
                        match gatherer.capability() {
                            GathererCapability::Available => {
                                let request = ContextGatherRequest::new(
                                    query.clone(),
                                    self.workspace_root.clone(),
                                    "planner-refine",
                                    EvidenceBudget::default(),
                                )
                                .with_planning(PlannerConfig::default().with_step_limit(1))
                                .with_prior_context(
                                    build_planner_prior_context(
                                        &context.interpretation,
                                        &context.recent_turns,
                                        &loop_state,
                                    ),
                                );
                                match gatherer.gather_context(&request).await {
                                    Ok(result) => {
                                        let bundle = result.evidence_bundle;
                                        if let Some(bundle) = bundle.as_ref() {
                                            event_sink.emit(TurnEvent::GathererSummary {
                                                provider: gatherer_provider.clone(),
                                                summary: bundle.summary.clone(),
                                                sources: evidence_sources(
                                                    &self.workspace_root,
                                                    bundle,
                                                ),
                                            });
                                            merge_evidence_items(
                                                &mut loop_state.evidence_items,
                                                bundle.items.clone(),
                                                budget.max_evidence_items,
                                            );
                                            loop_state.notes.extend(bundle.warnings.clone());
                                            used_workspace_resources = true;
                                            format!("refined search toward `{query}`")
                                        } else {
                                            "planner refine returned no evidence bundle".to_string()
                                        }
                                    }
                                    Err(err) => format!("planner refine failed: {err:#}"),
                                }
                            }
                            GathererCapability::Unsupported { reason }
                            | GathererCapability::HarnessRequired { reason } => {
                                format!("planner refine backend unavailable: {reason}")
                            }
                        }
                    } else {
                        "no gatherer backend is configured for refined planner search".to_string()
                    }
                }
                PlannerAction::Branch { branches, .. } => {
                    for branch in branches.iter().take(budget.max_branch_factor) {
                        if !loop_state.pending_branches.contains(branch) {
                            loop_state.pending_branches.push(branch.clone());
                        }
                    }
                    format!(
                        "queued {} planner branch(es)",
                        branches.len().min(budget.max_branch_factor)
                    )
                }
                PlannerAction::Stop { reason } => {
                    stop_reason = Some(reason.clone());
                    format!("planner requested synthesis: {reason}")
                }
            };

            loop_state.steps.push(PlannerStepRecord {
                sequence,
                action: decision.action.clone(),
                outcome: outcome.clone(),
            });

            if let PlannerAction::Stop { .. } = decision.action {
                break;
            }

            if stop_reason.is_some() {
                break;
            }
        }

        let completed = stop_reason.is_some();
        let stop_reason = stop_reason.unwrap_or_else(|| "planner-budget-exhausted".to_string());
        event_sink.emit(TurnEvent::PlannerSummary {
            strategy: "model-driven".to_string(),
            turns: loop_state.steps.len(),
            steps: loop_state.steps.len(),
            stop_reason: Some(stop_reason.clone()),
        });

        if !used_workspace_resources && planner_stopped_without_resource_use(&loop_state) {
            return Ok(None);
        }

        Ok(Some(build_planner_evidence_bundle(
            &context.prepared,
            prompt,
            &loop_state,
            completed,
            &stop_reason,
        )))
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
    PlannerThenSynthesize,
}

fn select_execution_plan(prompt: &str, planner_available: bool) -> PromptExecutionPlan {
    let intent = classify_turn(prompt);
    let path = if planner_available && intent.uses_planner() {
        PromptExecutionPath::PlannerThenSynthesize
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

    TurnIntent::Planned
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

fn normalize_turn_prompt(prompt: &str) -> String {
    prompt
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_ascii_lowercase()
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

fn format_planner_capability(capability: &PlannerCapability) -> String {
    match capability {
        PlannerCapability::Available => "available".to_string(),
        PlannerCapability::Unsupported { reason } => format!("unsupported: {reason}"),
    }
}

fn format_planner_strategy(strategy: &PlannerStrategyKind) -> &'static str {
    match strategy {
        PlannerStrategyKind::Heuristic => "heuristic",
        PlannerStrategyKind::ModelDriven => "model-driven",
    }
}

fn build_planner_prior_context(
    interpretation: &InterpretationContext,
    recent_turns: &[String],
    loop_state: &PlannerLoopState,
) -> Vec<String> {
    let mut prior = Vec::new();
    if !interpretation.is_empty() {
        prior.push(interpretation.render());
    }
    prior.extend(recent_turns.iter().cloned());
    prior.extend(
        loop_state
            .steps
            .iter()
            .map(|step| format!("step {}: {}", step.sequence, step.outcome)),
    );
    prior.extend(
        loop_state
            .pending_branches
            .iter()
            .map(|branch| format!("pending branch: {branch}")),
    );
    prior
}

fn read_steps(loop_state: &PlannerLoopState) -> usize {
    loop_state
        .steps
        .iter()
        .filter(|step| matches!(step.action, PlannerAction::Read { .. }))
        .count()
}

fn inspect_steps(loop_state: &PlannerLoopState) -> usize {
    loop_state
        .steps
        .iter()
        .filter(|step| matches!(step.action, PlannerAction::Inspect { .. }))
        .count()
}

fn merge_evidence_items(target: &mut Vec<EvidenceItem>, items: Vec<EvidenceItem>, limit: usize) {
    for item in items {
        append_evidence_item(target, item, limit);
    }
}

fn append_evidence_item(target: &mut Vec<EvidenceItem>, item: EvidenceItem, limit: usize) {
    let duplicate = target
        .iter()
        .any(|existing| existing.source == item.source && existing.snippet == item.snippet);
    if duplicate {
        return;
    }

    target.push(item);
    if target.len() > limit {
        target.truncate(limit);
    }
    for (index, item) in target.iter_mut().enumerate() {
        item.rank = index + 1;
    }
}

fn read_planner_file(workspace_root: &Path, path: &str) -> Result<(String, String)> {
    let resolved = resolve_planner_path(workspace_root, path)?;
    let contents = fs::read_to_string(&resolved)?;
    let relative = relative_workspace_path(workspace_root, &resolved);
    Ok((relative, trim_for_planner(&contents, 1_200)))
}

fn run_planner_inspect_command(workspace_root: &Path, command: &str) -> Result<String> {
    validate_inspect_command(command)?;
    let output = std::process::Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(workspace_root)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let rendered = if stderr.trim().is_empty() {
        stdout
    } else {
        format!("{stdout}\n{stderr}")
    };

    Ok(trim_for_planner(&rendered, 1_200))
}

fn validate_inspect_command(command: &str) -> Result<()> {
    let normalized = command.trim();
    if normalized.is_empty() {
        anyhow::bail!("planner inspect command must not be empty");
    }
    if normalized.contains("&&")
        || normalized.contains("||")
        || normalized.contains(';')
        || normalized.contains('>')
        || normalized.contains('<')
    {
        anyhow::bail!("planner inspect command must stay read-only and single-step");
    }

    let allowed_prefixes = [
        "git status",
        "git diff",
        "git log",
        "keel ",
        "rg ",
        "ls",
        "find ",
        "cat ",
        "sed -n",
        "head ",
        "tail ",
        "pwd",
    ];
    if allowed_prefixes
        .iter()
        .any(|prefix| normalized.starts_with(prefix))
    {
        Ok(())
    } else {
        anyhow::bail!("planner inspect command is outside the safe read-only allowlist")
    }
}

fn resolve_planner_path(workspace_root: &Path, requested: &str) -> Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        anyhow::bail!("absolute planner paths are not allowed");
    }

    let canonical_root = workspace_root.canonicalize()?;
    let resolved = canonical_root.join(requested_path);
    let canonical = resolved.canonicalize()?;
    if !canonical.starts_with(&canonical_root) {
        anyhow::bail!("planner path escapes workspace root");
    }
    Ok(canonical)
}

fn relative_workspace_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn trim_for_planner(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.trim().to_string();
    }

    let kept = input.chars().take(limit).collect::<String>();
    format!("{}...[truncated]", kept.trim_end())
}

fn planner_stopped_without_resource_use(loop_state: &PlannerLoopState) -> bool {
    matches!(
        loop_state.steps.last().map(|step| &step.action),
        Some(PlannerAction::Stop { .. })
    ) && loop_state.steps.iter().all(|step| {
        matches!(
            step.action,
            PlannerAction::Stop { .. } | PlannerAction::Branch { .. }
        )
    })
}

fn build_planner_evidence_bundle(
    prepared: &PreparedRuntimeLanes,
    prompt: &str,
    loop_state: &PlannerLoopState,
    completed: bool,
    stop_reason: &str,
) -> EvidenceBundle {
    let summary = format!(
        "Planner lane `{}` executed {} step(s) for `{}` and collected {} evidence item(s); stop reason: {}.",
        prepared.planner.model_id,
        loop_state.steps.len(),
        prompt,
        loop_state.evidence_items.len(),
        stop_reason
    );
    let planner = PlannerTraceMetadata {
        strategy: PlannerStrategyKind::ModelDriven,
        profile: Some(prepared.planner.model_id.clone()),
        completed,
        stop_reason: Some(stop_reason.to_string()),
        turn_count: loop_state.steps.len(),
        steps: loop_state
            .steps
            .iter()
            .map(|step| PlannerTraceStep {
                step_id: format!("planner-step-{}", step.sequence),
                sequence: step.sequence,
                parent_step_id: None,
                decisions: vec![crate::domain::ports::PlannerDecision {
                    action: step.action.label().to_string(),
                    query: planner_action_query(&step.action),
                    rationale: Some(step.outcome.clone()),
                    next_step_id: None,
                    turn_id: None,
                    stop_reason: matches!(step.action, PlannerAction::Stop { .. })
                        .then(|| stop_reason.to_string()),
                }],
            })
            .collect(),
        retained_artifacts: loop_state
            .evidence_items
            .iter()
            .take(5)
            .map(|item| RetainedEvidence {
                source: item.source.clone(),
                snippet: Some(item.snippet.clone()),
                rationale: Some(item.rationale.clone()),
            })
            .collect(),
    };
    let mut bundle =
        EvidenceBundle::new(summary, loop_state.evidence_items.clone()).with_planner(planner);
    if !loop_state.notes.is_empty() {
        bundle = bundle.with_warnings(loop_state.notes.clone());
    }
    bundle
}

fn planner_action_query(action: &PlannerAction) -> Option<String> {
    match action {
        PlannerAction::Search { query, .. } | PlannerAction::Refine { query, .. } => {
            Some(query.clone())
        }
        PlannerAction::Branch { branches, .. } => Some(branches.join(" | ")),
        PlannerAction::Read { path } => Some(path.clone()),
        PlannerAction::Inspect { command } => Some(command.clone()),
        PlannerAction::Stop { reason } => Some(reason.clone()),
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
        let planner = MechSuitService::build_lane(
            RuntimeLaneRole::Planner,
            "qwen-1.5b",
            sample_model_paths("planner"),
        );
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
            planner,
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

        assert_eq!(plan.intent, TurnIntent::Planned);
        assert_eq!(plan.path, super::PromptExecutionPath::PlannerThenSynthesize);
    }

    #[test]
    fn decomposition_worthy_prompts_use_gatherer_lane_when_available() {
        let plan =
            super::select_execution_plan("Trace the runtime lane architecture end-to-end", true);

        assert_eq!(plan.intent, TurnIntent::Planned);
        assert_eq!(plan.path, super::PromptExecutionPath::PlannerThenSynthesize);
    }

    #[test]
    fn repository_questions_use_gatherer_lane_when_available() {
        let plan = super::select_execution_plan("How does memory work in paddles?", true);

        assert_eq!(plan.intent, TurnIntent::Planned);
        assert_eq!(plan.path, super::PromptExecutionPath::PlannerThenSynthesize);
    }

    #[test]
    fn only_casual_and_explicit_actions_skip_the_planner() {
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
            super::PromptExecutionPath::PlannerThenSynthesize
        );
    }

    #[test]
    fn planned_prompts_without_a_planner_lane_stay_on_synthesizer() {
        assert_eq!(
            super::select_execution_plan("Trace the runtime lane architecture end-to-end", false)
                .path,
            super::PromptExecutionPath::SynthesizerOnly
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
