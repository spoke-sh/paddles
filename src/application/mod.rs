use crate::infrastructure::adapters::TransitContextResolver;
use crate::infrastructure::adapters::trace_recorders::TransitTraceRecorder;
use crate::infrastructure::adapters::transit_resolver::NoopContextResolver;
use crate::infrastructure::providers::ModelProvider;
pub use paddles_conversation::{ContextLocator, ConversationSession, TraceArtifactId};

use crate::domain::model::{
    ArtifactEnvelope, ArtifactKind, BootContext, CompactionDecision, CompactionPlan,
    ConversationThreadRef, ConversationTranscript, ConversationTranscriptUpdate,
    ForensicArtifactCapture, ForensicTraceSink, MultiplexEventSink, TaskTraceId, ThreadCandidate,
    ThreadDecision, ThreadDecisionKind, ThreadMergeMode, ThreadMergeRecord, TraceBranch,
    TraceBranchId, TraceBranchStatus, TraceCheckpointId, TraceCheckpointKind,
    TraceCompletionCheckpoint, TraceLineage, TraceModelExchangeArtifact, TraceModelExchangePhase,
    TraceRecord, TraceRecordId, TraceRecordKind, TraceSelectionArtifact, TraceSelectionKind,
    TraceTaskRoot, TraceToolCall, TraceTurnStarted, TranscriptUpdateSink, TurnEvent, TurnEventSink,
    TurnIntent, TurnTraceId,
};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, ContextResolver, EvidenceBudget, EvidenceBundle,
    EvidenceItem, GathererCapability, InitialAction, InitialActionDecision, InterpretationContext,
    InterpretationRequest, ModelPaths, ModelRegistry, NoopTraceRecorder, OperatorMemory,
    PlannerAction, PlannerBudget, PlannerCapability, PlannerConfig, PlannerLoopState,
    PlannerRequest, PlannerStepRecord, PlannerStrategyKind, PlannerTraceMetadata, PlannerTraceStep,
    RecursivePlanner, RecursivePlannerDecision, RetainedEvidence, RetrievalMode, RetrievalStrategy,
    SynthesizerEngine, ThreadDecisionRequest, TraceRecorder, WorkspaceAction,
};
use anyhow::Result;
use clap::ValueEnum;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

/// Factory that constructs a synthesizer engine for a given model ID.
pub type SynthesizerFactory =
    dyn Fn(&Path, &PreparedModelLane) -> Result<Arc<dyn SynthesizerEngine>> + Send + Sync;

/// Factory that constructs a recursive planner for a given model ID.
pub type PlannerFactory =
    dyn Fn(&Path, &PreparedModelLane) -> Result<Arc<dyn RecursivePlanner>> + Send + Sync;

/// Factory that constructs an optional gatherer from runtime configuration.
///
/// Arguments: `(config, workspace_root, verbose, gatherer_model_paths)`.
/// The application resolves model paths from the registry before calling.
pub type GathererFactory = dyn Fn(
        &RuntimeLaneConfig,
        &Path,
        u8,
        Option<ModelPaths>,
    ) -> Result<Option<(PreparedGathererLane, Arc<dyn ContextGatherer>)>>
    + Send
    + Sync;

/// Application service for managing the mech suit lifecycle.
pub struct MechSuitService {
    workspace_root: PathBuf,
    registry: Arc<dyn ModelRegistry>,
    operator_memory: Arc<dyn OperatorMemory>,
    synthesizer_factory: Box<SynthesizerFactory>,
    planner_factory: Box<PlannerFactory>,
    gatherer_factory: Box<GathererFactory>,
    runtime: RwLock<Option<ActiveRuntimeState>>,
    verbose: AtomicU8,
    event_sink: Arc<dyn TurnEventSink>,
    event_observers: Mutex<Vec<Arc<dyn TurnEventSink>>>,
    transcript_observers: Mutex<Vec<Arc<dyn TranscriptUpdateSink>>>,
    trace_recorder: Arc<dyn TraceRecorder>,
    trace_counter: AtomicU64,
    sessions: Mutex<HashMap<String, ConversationSession>>,
    shared_session_id: Mutex<Option<String>>,
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
    #[value(alias = "sift-autonomous")]
    SiftDirect,
    Context1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLaneConfig {
    synthesizer_provider: ModelProvider,
    planner_model_id: Option<String>,
    planner_provider: Option<ModelProvider>,
    synthesizer_model_id: String,
    gatherer_model_id: Option<String>,
    gatherer_provider: GathererProvider,
    context1_harness_ready: bool,
}

impl RuntimeLaneConfig {
    pub fn new(synthesizer_model_id: impl Into<String>, gatherer_model_id: Option<String>) -> Self {
        Self {
            synthesizer_provider: ModelProvider::Sift,
            planner_model_id: None,
            planner_provider: None,
            synthesizer_model_id: synthesizer_model_id.into(),
            gatherer_model_id,
            gatherer_provider: GathererProvider::SiftDirect,
            context1_harness_ready: false,
        }
    }

    pub fn with_planner_model_id(mut self, planner_model_id: Option<String>) -> Self {
        self.planner_model_id = planner_model_id;
        self
    }

    pub fn with_synthesizer_provider(mut self, provider: ModelProvider) -> Self {
        self.synthesizer_provider = provider;
        self
    }

    pub fn with_planner_provider(mut self, provider: Option<ModelProvider>) -> Self {
        self.planner_provider = provider;
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

    pub fn synthesizer_provider(&self) -> ModelProvider {
        self.synthesizer_provider
    }

    pub fn planner_model_id(&self) -> Option<&str> {
        self.planner_model_id.as_deref()
    }

    pub fn planner_provider(&self) -> ModelProvider {
        self.planner_provider.unwrap_or(self.synthesizer_provider)
    }

    pub fn planner_provider_override(&self) -> Option<ModelProvider> {
        self.planner_provider
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
    pub provider: ModelProvider,
    pub model_id: String,
    pub paths: Option<ModelPaths>,
}

impl PreparedModelLane {
    pub fn qualified_model_label(&self) -> String {
        self.provider.qualified_model_label(&self.model_id)
    }
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
    synthesizer_engine: Arc<dyn SynthesizerEngine>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
}

struct PlannerLoopContext {
    prepared: PreparedRuntimeLanes,
    planner_engine: Arc<dyn RecursivePlanner>,
    synthesizer_engine: Arc<dyn SynthesizerEngine>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
    resolver: Arc<dyn ContextResolver>,
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

#[derive(Clone)]
struct StructuredTurnTrace {
    downstream: Arc<dyn TurnEventSink>,
    recorder: Arc<dyn TraceRecorder>,
    session: ConversationSession,
    turn_id: TurnTraceId,
    active_thread: ConversationThreadRef,
    last_synthesis: Arc<Mutex<Option<SynthesisTraceState>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SynthesisTraceState {
    grounded: bool,
    citations: Vec<String>,
    insufficient_evidence: bool,
}

impl StructuredTurnTrace {
    fn new(
        downstream: Arc<dyn TurnEventSink>,
        recorder: Arc<dyn TraceRecorder>,
        session: ConversationSession,
        turn_id: TurnTraceId,
        active_thread: ConversationThreadRef,
    ) -> Self {
        Self {
            downstream,
            recorder,
            session,
            turn_id,
            active_thread,
            last_synthesis: Arc::new(Mutex::new(None)),
        }
    }

    fn as_event_sink(self: &Arc<Self>) -> Arc<dyn TurnEventSink> {
        let sink: Arc<dyn TurnEventSink> = self.clone();
        sink
    }

    fn default_branch_id(&self) -> Option<TraceBranchId> {
        self.active_thread.branch_id()
    }

    fn record_turn_start(
        &self,
        prompt: &str,
        interpretation: &InterpretationContext,
        prepared: &PreparedRuntimeLanes,
    ) {
        let prompt_artifact = self.text_artifact(
            ArtifactKind::Prompt,
            format!("user prompt `{}`", trim_for_planner(prompt, 80)),
            prompt,
            800,
        );
        let interpretation_artifact = (!interpretation.is_empty()).then(|| {
            self.text_artifact(
                ArtifactKind::Interpretation,
                interpretation.summary.clone(),
                interpretation.render(),
                1_200,
            )
            .with_label("sources", interpretation.sources().join(", "))
        });
        let record_task_root = {
            let state = self.session.state();
            let mut state = state.lock().expect("conversation session lock");
            if state.root_started {
                false
            } else {
                state.root_started = true;
                true
            }
        };

        if record_task_root {
            self.record_kind(
                None,
                TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                    prompt: prompt_artifact,
                    interpretation: interpretation_artifact,
                    planner_model: prepared.planner.model_id.clone(),
                    synthesizer_model: prepared.synthesizer.model_id.clone(),
                }),
            );
            return;
        }

        self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::TurnStarted(TraceTurnStarted {
                prompt: prompt_artifact,
                interpretation: interpretation_artifact,
                planner_model: prepared.planner.model_id.clone(),
                synthesizer_model: prepared.synthesizer.model_id.clone(),
                thread: self.active_thread.clone(),
            }),
        );
    }

    fn record_planner_action(
        &self,
        action: &str,
        rationale: &str,
        branch_id: Option<TraceBranchId>,
    ) {
        self.record_kind(
            branch_id.or_else(|| self.default_branch_id()),
            TraceRecordKind::PlannerAction {
                action: action.to_string(),
                rationale: rationale.to_string(),
            },
        );
    }

    fn declare_branch(
        &self,
        branch_id: TraceBranchId,
        label: &str,
        rationale: Option<&str>,
        parent_branch_id: Option<TraceBranchId>,
    ) -> TraceBranch {
        let branch = {
            let state = self.session.state();
            let state = state.lock().expect("conversation session lock");
            TraceBranch {
                branch_id,
                label: label.to_string(),
                status: TraceBranchStatus::Pending,
                rationale: rationale.map(str::to_string),
                parent_branch_id,
                created_from_record_id: self.parent_record_id_for(
                    state.root_last_record_id.clone(),
                    &state.branch_last_record_ids,
                    self.default_branch_id().as_ref(),
                ),
            }
        };
        self.record_kind(
            Some(branch.branch_id.clone()),
            TraceRecordKind::PlannerBranchDeclared(branch.clone()),
        );
        branch
    }

    fn record_selection_artifact(
        &self,
        kind: TraceSelectionKind,
        summary: impl Into<String>,
        sources: Vec<String>,
        content: impl Into<String>,
        artifact_kind: ArtifactKind,
    ) {
        let summary = summary.into();
        let artifact = self.text_artifact(artifact_kind, summary.clone(), content.into(), 1_200);
        self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::SelectionArtifact(TraceSelectionArtifact {
                selection_id: artifact.artifact_id.clone(),
                kind,
                summary,
                artifact,
                selected_from: sources,
            }),
        );
    }

    fn record_thread_candidate(&self, candidate: &ThreadCandidate) {
        self.record_kind(
            candidate.active_thread.branch_id(),
            TraceRecordKind::ThreadCandidateCaptured(candidate.clone()),
        );
    }

    fn record_thread_decision(
        &self,
        decision: &ThreadDecision,
        source_thread: &ConversationThreadRef,
    ) {
        self.record_kind(
            source_thread.branch_id(),
            TraceRecordKind::ThreadDecisionSelected(decision.clone()),
        );
    }

    fn record_thread_merge(
        &self,
        decision: &ThreadDecision,
        source_thread: &ConversationThreadRef,
        target_thread: &ConversationThreadRef,
    ) {
        let summary_artifact = decision.merge_summary.as_ref().map(|summary| {
            self.text_artifact(
                ArtifactKind::Selection,
                format!(
                    "thread {} outcome",
                    decision
                        .merge_mode
                        .unwrap_or(ThreadMergeMode::Summary)
                        .label()
                ),
                summary,
                800,
            )
        });
        self.record_kind(
            source_thread.branch_id(),
            TraceRecordKind::ThreadMerged(ThreadMergeRecord {
                decision: decision.clone(),
                source_thread: source_thread.clone(),
                target_thread: target_thread.clone(),
                summary_artifact,
            }),
        );
    }

    fn remember_synthesis(
        &self,
        grounded: bool,
        citations: Vec<String>,
        insufficient_evidence: bool,
    ) {
        let mut state = self.last_synthesis.lock().expect("synthesis trace lock");
        *state = Some(SynthesisTraceState {
            grounded,
            citations,
            insufficient_evidence,
        });
    }

    fn record_completion(&self, reply: &str) {
        let synthesis = self
            .last_synthesis
            .lock()
            .expect("synthesis trace lock")
            .clone()
            .unwrap_or(SynthesisTraceState {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            });
        let response = self
            .text_artifact(
                ArtifactKind::ModelOutput,
                if synthesis.insufficient_evidence {
                    "insufficient evidence response".to_string()
                } else {
                    "assistant response".to_string()
                },
                reply,
                1_200,
            )
            .with_label("citations", synthesis.citations.join(", "));
        self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                checkpoint_id: self.next_checkpoint_id(),
                kind: if synthesis.insufficient_evidence {
                    TraceCheckpointKind::TurnFailed
                } else {
                    TraceCheckpointKind::TurnCompleted
                },
                summary: if synthesis.insufficient_evidence {
                    "turn ended with insufficient evidence".to_string()
                } else {
                    "turn completed".to_string()
                },
                response: Some(response),
                citations: synthesis.citations,
                grounded: synthesis.grounded,
            }),
        );
    }

    fn parent_record_id_for(
        &self,
        root_last_record_id: Option<TraceRecordId>,
        branch_last_record_ids: &HashMap<TraceBranchId, TraceRecordId>,
        branch_id: Option<&TraceBranchId>,
    ) -> Option<TraceRecordId> {
        branch_id
            .and_then(|branch| branch_last_record_ids.get(branch).cloned())
            .or(root_last_record_id)
    }

    fn record_kind(&self, branch_id: Option<TraceBranchId>, kind: TraceRecordKind) {
        let record = {
            let session_state = self.session.state();
            let mut state = session_state.lock().expect("conversation session lock");
            let sequence = state.next_record_sequence;
            let record_id =
                TraceRecordId::new(format!("{}.record-{:04}", self.turn_id.as_str(), sequence))
                    .expect("generated record id");
            state.next_record_sequence += 1;
            let parent_record_id = self.parent_record_id_for(
                state.root_last_record_id.clone(),
                &state.branch_last_record_ids,
                branch_id.as_ref(),
            );
            let lineage = TraceLineage {
                task_id: state.task_id.clone(),
                turn_id: self.turn_id.clone(),
                branch_id: branch_id.clone(),
                parent_record_id,
            };
            if let Some(branch_id) = branch_id {
                state
                    .branch_last_record_ids
                    .insert(branch_id, record_id.clone());
            } else {
                state.root_last_record_id = Some(record_id.clone());
            }

            TraceRecord {
                record_id,
                sequence,
                lineage,
                kind,
            }
        };
        self.record_or_warn(record);
    }

    fn text_artifact(
        &self,
        kind: ArtifactKind,
        summary: impl Into<String>,
        content: impl Into<String>,
        inline_limit: usize,
    ) -> ArtifactEnvelope {
        let artifact_id = self.session.next_artifact_id(&self.turn_id);
        ArtifactEnvelope::text(artifact_id, kind, summary, content, inline_limit)
    }

    fn exact_artifact(
        &self,
        kind: ArtifactKind,
        summary: impl Into<String>,
        content: impl Into<String>,
        mime_type: impl Into<String>,
    ) -> ArtifactEnvelope {
        let artifact_id = self.session.next_artifact_id(&self.turn_id);
        ArtifactEnvelope::text(artifact_id, kind, summary, content, usize::MAX)
            .with_mime_type(mime_type)
    }

    fn next_checkpoint_id(&self) -> TraceCheckpointId {
        TraceCheckpointId::new(format!("{}.checkpoint", self.turn_id.as_str()))
            .expect("generated checkpoint id")
    }

    fn record_or_warn(&self, record: TraceRecord) {
        if let Err(err) = self.recorder.record(record) {
            let should_emit = {
                let state = self.session.state();
                let mut state = state.lock().expect("conversation session lock");
                if state.recorder_warning_emitted {
                    false
                } else {
                    state.recorder_warning_emitted = true;
                    true
                }
            };
            if should_emit {
                self.downstream.emit(TurnEvent::Fallback {
                    stage: "trace-recorder".to_string(),
                    reason: format!("trace recording failed: {err:#}"),
                });
            }
        }
    }
}

impl TurnEventSink for StructuredTurnTrace {
    fn emit(&self, event: TurnEvent) {
        self.downstream.emit(event.clone());
        match event {
            TurnEvent::ToolCalled {
                call_id,
                tool_name,
                invocation,
            } => {
                let artifact = self.text_artifact(
                    ArtifactKind::ToolInvocation,
                    format!("tool request `{tool_name}`"),
                    invocation,
                    800,
                );
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::ToolCallRequested(TraceToolCall {
                        call_id,
                        tool_name,
                        payload: artifact,
                        success: None,
                    }),
                );
            }
            TurnEvent::ToolFinished {
                call_id,
                tool_name,
                summary,
            } => {
                let success = !summary.to_ascii_lowercase().contains("failed:");
                let artifact = self.text_artifact(
                    ArtifactKind::ToolOutput,
                    format!("tool result `{tool_name}`"),
                    summary,
                    1_000,
                );
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::ToolCallCompleted(TraceToolCall {
                        call_id,
                        tool_name,
                        payload: artifact,
                        success: Some(success),
                    }),
                );
            }
            TurnEvent::SynthesisReady {
                grounded,
                citations,
                insufficient_evidence,
            } => {
                self.remember_synthesis(grounded, citations, insufficient_evidence);
            }
            _ => {}
        }
    }

    fn forensic_trace_sink(&self) -> Option<&dyn ForensicTraceSink> {
        Some(self)
    }
}

impl ForensicTraceSink for StructuredTurnTrace {
    fn record_forensic_artifact(
        &self,
        capture: ForensicArtifactCapture,
    ) -> Option<TraceArtifactId> {
        let mut artifact = self
            .exact_artifact(
                match capture.phase {
                    TraceModelExchangePhase::AssembledContext => ArtifactKind::Prompt,
                    TraceModelExchangePhase::ProviderRequest => ArtifactKind::ToolInvocation,
                    TraceModelExchangePhase::RawProviderResponse => ArtifactKind::ModelOutput,
                    TraceModelExchangePhase::RenderedResponse => ArtifactKind::Checkpoint,
                },
                capture.summary,
                capture.content,
                capture.mime_type,
            )
            .with_label("lane", capture.lane.label())
            .with_label("category", capture.category.label())
            .with_label("phase", capture.phase.label())
            .with_label("provider", capture.provider.clone())
            .with_label("model", capture.model.clone());
        for (key, value) in capture.labels {
            artifact = artifact.with_label(key, value);
        }
        let artifact_id = artifact.artifact_id.clone();
        self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                lane: capture.lane,
                category: capture.category,
                phase: capture.phase,
                provider: capture.provider,
                model: capture.model,
                parent_artifact_id: capture.parent_artifact_id,
                artifact,
            }),
        );
        Some(artifact_id)
    }
}

fn render_turn_event(event: &TurnEvent) -> String {
    match event {
        TurnEvent::IntentClassified { intent } => {
            format!("• Classified turn\n  └ {}", intent.label())
        }
        TurnEvent::InterpretationContext { context } => {
            let mut lines = vec![format!(
                "• Assembled interpretation context [{} docs, {} hints, {} procedures]",
                context.documents.len(),
                context.tool_hints.len(),
                context.decision_framework.procedures.len()
            )];
            lines.push(format!("  └ {}", trim_event_detail(&context.summary, 3)));
            if !context.documents.is_empty() {
                lines.push(format!(
                    "    Sources: {}",
                    trim_event_detail(&context.sources().join(", "), 2)
                ));
            }
            lines.join("\n")
        }
        TurnEvent::GuidanceGraphExpanded {
            depth,
            document_count,
            sources,
        } => format!(
            "• Expanded guidance graph (depth {})\n  └ Discovered {} docs: {}",
            depth,
            document_count,
            trim_event_detail(&sources.join(", "), 2)
        ),
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
            "• Planner step {sequence}: {}\n  └ Rationale: {}",
            trim_event_detail(action, 1),
            trim_event_detail(rationale, 2)
        ),
        TurnEvent::ThreadCandidateCaptured {
            candidate_id,
            active_thread,
            prompt,
        } => format!(
            "• Captured steering prompt\n  └ {candidate_id} on {active_thread}: {}",
            trim_event_detail(prompt, 2)
        ),
        TurnEvent::ThreadDecisionApplied {
            candidate_id,
            decision,
            target_thread,
            rationale,
        } => format!(
            "• Applied thread decision\n  └ {candidate_id}: {decision} -> {target_thread}\n    Rationale: {}",
            trim_event_detail(rationale, 2)
        ),
        TurnEvent::ThreadMerged {
            source_thread,
            target_thread,
            mode,
            summary,
        } => format!(
            "• Merged thread\n  └ {} -> {} via {}\n    {}",
            source_thread,
            target_thread,
            mode,
            trim_event_detail(
                summary.as_deref().unwrap_or("No merge summary recorded."),
                2
            )
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
            mode,
            turns,
            steps,
            stop_reason,
            active_branch_id,
            branch_count,
            frontier_count,
            node_count,
            edge_count,
            retained_artifact_count,
        } => {
            let opt = |v: &Option<usize>| {
                v.map(|n| n.to_string())
                    .unwrap_or_else(|| "n/a".to_string())
            };
            format!(
                "• Reviewed planner trace\n  └ strategy={strategy}, mode={mode}, turns={turns}, steps={steps}, stop={}, active={}, branches={}, frontier={}, nodes={}, edges={}, retained={}",
                stop_reason.as_deref().unwrap_or("none"),
                active_branch_id.as_deref().unwrap_or("none"),
                opt(branch_count),
                opt(frontier_count),
                opt(node_count),
                opt(edge_count),
                opt(retained_artifact_count),
            )
        }
        TurnEvent::RefinementApplied {
            reason,
            before_summary,
            after_summary,
        } => format!(
            "• Applied interpretation refinement\n  └ {reason}\n  └ before: {}\n  └ after: {}",
            trim_event_detail(before_summary, 1),
            trim_event_detail(after_summary, 1)
        ),
        TurnEvent::ContextAssembly {
            label,
            hits,
            retained_artifacts,
            pruned_artifacts,
        } => format!(
            "• Assembled workspace context ({label})\n  └ {hits} hit(s), retained {retained_artifacts}, pruned {pruned_artifacts}"
        ),
        TurnEvent::ContextPressure { pressure } => {
            let factors: Vec<_> = pressure.factors.iter().map(|f| f.label()).collect();
            format!(
                "• Context pressure: {}\n  └ {} truncation(s), factors: [{}]",
                pressure.level.label(),
                pressure.truncation_count,
                factors.join(", ")
            )
        }
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
        TurnEvent::PlannerStepProgress {
            step_number,
            step_limit,
            action,
            query,
            evidence_count,
        } => {
            let q = query
                .as_deref()
                .map(|q| format!(" — {}", trim_event_detail(q, 1)))
                .unwrap_or_default();
            format!("• Step {step_number}/{step_limit}: {action}{q} [{evidence_count} evidence]")
        }
        TurnEvent::GathererSearchProgress {
            phase,
            elapsed_seconds,
            eta_seconds,
            strategy,
            detail,
        } => {
            let eta = eta_seconds
                .map(|eta| format!(" eta {eta}s"))
                .unwrap_or_else(|| " eta unknown".to_string());
            let strategy = strategy
                .as_deref()
                .map(|value| format!(" strategy={value}"))
                .unwrap_or_default();
            let suffix = detail
                .as_deref()
                .map(|d| format!(" | {d}"))
                .unwrap_or_default();
            format!("• Searching ({phase})\n  └ elapsed {elapsed_seconds}s{eta}{strategy}{suffix}")
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
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        registry: Arc<dyn ModelRegistry>,
        operator_memory: Arc<dyn OperatorMemory>,
        synthesizer_factory: Box<SynthesizerFactory>,
        planner_factory: Box<PlannerFactory>,
        gatherer_factory: Box<GathererFactory>,
    ) -> Self {
        Self::with_trace_recorder(
            workspace_root,
            registry,
            operator_memory,
            synthesizer_factory,
            planner_factory,
            gatherer_factory,
            Arc::new(NoopTraceRecorder),
        )
    }

    pub fn with_trace_recorder(
        workspace_root: impl Into<PathBuf>,
        registry: Arc<dyn ModelRegistry>,
        operator_memory: Arc<dyn OperatorMemory>,
        synthesizer_factory: Box<SynthesizerFactory>,
        planner_factory: Box<PlannerFactory>,
        gatherer_factory: Box<GathererFactory>,
        trace_recorder: Arc<dyn TraceRecorder>,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            registry,
            operator_memory,
            synthesizer_factory,
            planner_factory,
            gatherer_factory,
            runtime: RwLock::new(None),
            verbose: AtomicU8::new(0),
            event_sink: Arc::new(ConsoleTurnEventSink::default()),
            event_observers: Mutex::new(Vec::new()),
            transcript_observers: Mutex::new(Vec::new()),
            trace_recorder,
            trace_counter: AtomicU64::new(1),
            sessions: Mutex::new(HashMap::new()),
            shared_session_id: Mutex::new(None),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    /// Register an additional event observer that receives all TurnEvents
    /// from every turn, regardless of which interface submitted it.
    pub fn register_event_observer(&self, observer: Arc<dyn TurnEventSink>) {
        self.event_observers
            .lock()
            .expect("event observers lock")
            .push(observer);
    }

    pub fn register_transcript_observer(&self, observer: Arc<dyn TranscriptUpdateSink>) {
        self.transcript_observers
            .lock()
            .expect("transcript observers lock")
            .push(observer);
    }

    fn emit_transcript_update(&self, task_id: &TaskTraceId) {
        let update = ConversationTranscriptUpdate {
            task_id: task_id.clone(),
        };
        let observers = self
            .transcript_observers
            .lock()
            .expect("transcript observers lock")
            .clone();
        for observer in observers {
            observer.emit(update.clone());
        }
    }

    pub fn replay_all_traces(&self) -> Result<Vec<crate::domain::model::TraceReplay>> {
        let mut task_ids = self.trace_recorder.task_ids();
        task_ids.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        task_ids
            .iter()
            .map(|id| self.trace_recorder.replay(id))
            .collect()
    }

    fn wrap_sink_with_observers(&self, sink: Arc<dyn TurnEventSink>) -> Arc<dyn TurnEventSink> {
        let observers = self
            .event_observers
            .lock()
            .expect("event observers lock")
            .clone();
        if observers.is_empty() {
            return sink;
        }
        let mut sinks = vec![sink];
        sinks.extend(observers);
        Arc::new(MultiplexEventSink::new(sinks))
    }

    fn allocate_task_id(&self) -> TaskTraceId {
        let sequence = self.trace_counter.fetch_add(1, Ordering::Relaxed);
        TaskTraceId::new(format!("task-{sequence:06}")).expect("generated task trace id")
    }

    fn register_session(&self, session: ConversationSession) -> ConversationSession {
        self.sessions
            .lock()
            .expect("conversation sessions lock")
            .insert(session.task_id().as_str().to_string(), session.clone());
        session
    }

    pub fn create_conversation_session(&self) -> ConversationSession {
        self.register_session(ConversationSession::new(self.allocate_task_id()))
    }

    pub fn shared_conversation_session(&self) -> ConversationSession {
        if let Some(session_id) = self
            .shared_session_id
            .lock()
            .expect("shared session lock")
            .clone()
            && let Some(session) = self
                .sessions
                .lock()
                .expect("conversation sessions lock")
                .get(&session_id)
                .cloned()
        {
            return session;
        }

        let session = self.create_conversation_session();
        *self.shared_session_id.lock().expect("shared session lock") =
            Some(session.task_id().as_str().to_string());
        session
    }

    pub fn conversation_session(&self, task_id: &TaskTraceId) -> Option<ConversationSession> {
        self.sessions
            .lock()
            .expect("conversation sessions lock")
            .get(task_id.as_str())
            .cloned()
    }

    pub fn replay_conversation_transcript(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationTranscript> {
        match self.trace_recorder.replay(task_id) {
            Ok(replay) => Ok(ConversationTranscript::from_trace_replay(&replay)),
            Err(err) => {
                let known_session = self
                    .sessions
                    .lock()
                    .expect("conversation sessions lock")
                    .contains_key(task_id.as_str());
                if known_session {
                    Ok(ConversationTranscript {
                        task_id: task_id.clone(),
                        entries: Vec::new(),
                    })
                } else {
                    Err(err)
                }
            }
        }
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
        provider: ModelProvider,
        model_id: impl Into<String>,
        paths: Option<ModelPaths>,
    ) -> PreparedModelLane {
        PreparedModelLane {
            role,
            provider,
            model_id: model_id.into(),
            paths,
        }
    }

    /// Prepare the configured runtime lanes for inference.
    pub async fn prepare_runtime_lanes(
        &self,
        config: &RuntimeLaneConfig,
    ) -> Result<PreparedRuntimeLanes> {
        let synthesizer_paths = if config.synthesizer_provider() == ModelProvider::Sift {
            Some(
                self.registry
                    .get_model_paths(config.synthesizer_model_id())
                    .await?,
            )
        } else {
            None
        };
        let planner_model_id = config
            .planner_model_id()
            .unwrap_or(config.synthesizer_model_id())
            .to_string();
        let planner_provider = config.planner_provider();
        let planner_paths = if planner_provider != ModelProvider::Sift {
            None
        } else if planner_model_id == config.synthesizer_model_id() {
            synthesizer_paths.clone()
        } else {
            Some(self.registry.get_model_paths(&planner_model_id).await?)
        };
        let planner = Self::build_lane(
            RuntimeLaneRole::Planner,
            planner_provider,
            &planner_model_id,
            planner_paths,
        );
        let synthesizer = Self::build_lane(
            RuntimeLaneRole::Synthesizer,
            config.synthesizer_provider(),
            config.synthesizer_model_id(),
            synthesizer_paths,
        );

        let gatherer_model_paths = match config.gatherer_provider() {
            GathererProvider::Local => match config.gatherer_model_id() {
                Some(model_id) => Some(self.registry.get_model_paths(model_id).await?),
                None => None,
            },
            _ => None,
        };
        let (prepared_gatherer, gatherer) = match (self.gatherer_factory)(
            config,
            &self.workspace_root,
            self.verbose.load(Ordering::Relaxed),
            gatherer_model_paths,
        )? {
            Some((lane, adapter)) => (Some(lane), Some(adapter)),
            None => (None, None),
        };

        let prepared = PreparedRuntimeLanes {
            planner,
            synthesizer,
            gatherer: prepared_gatherer,
        };

        let verbose = self.verbose.load(Ordering::Relaxed);
        let engine = (self.synthesizer_factory)(&self.workspace_root, &prepared.synthesizer)?;
        engine.set_verbose(verbose);
        let planner_engine = (self.planner_factory)(&self.workspace_root, &prepared.planner)?;

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
        let session = self.create_conversation_session();
        self.process_prompt_in_session_with_sink(prompt, session, Arc::clone(&self.event_sink))
            .await
    }

    pub async fn process_prompt_with_sink(
        &self,
        prompt: &str,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let session = self.create_conversation_session();
        self.process_prompt_in_session_with_sink(prompt, session, event_sink)
            .await
    }

    pub async fn process_prompt_in_session_with_sink(
        &self,
        prompt: &str,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let event_sink = self.wrap_sink_with_observers(event_sink);
        let runtime_guard = self.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
        let prepared = runtime.prepared.clone();
        let planner_engine = Arc::clone(&runtime.planner_engine);
        let synthesizer_engine = Arc::clone(&runtime.synthesizer_engine);
        let gatherer = runtime.gatherer.clone();
        drop(runtime_guard);

        let interpretation = self
            .derive_interpretation_context(prompt, planner_engine.as_ref(), event_sink.clone())
            .await;
        let turn_id = session.allocate_turn_id();
        let active_thread = session.active_thread().thread_ref;
        let trace = Arc::new(StructuredTurnTrace::new(
            event_sink,
            Arc::clone(&self.trace_recorder),
            session.clone(),
            turn_id,
            active_thread.clone(),
        ));
        trace.record_turn_start(prompt, &interpretation, &prepared);
        self.emit_transcript_update(&session.task_id());
        trace.emit(TurnEvent::InterpretationContext {
            context: interpretation.clone(),
        });

        let planner_capability = planner_engine.capability();
        trace.emit(TurnEvent::PlannerCapability {
            provider: prepared.planner.model_id.clone(),
            capability: format_planner_capability(&planner_capability),
        });

        let recent_turns = synthesizer_engine.recent_turn_summaries()?;
        let request = PlannerRequest::new(
            prompt,
            self.workspace_root.clone(),
            interpretation.clone(),
            PlannerBudget::default(),
        )
        .with_recent_turns(recent_turns.clone());

        let execution_plan = match planner_capability {
            PlannerCapability::Available => {
                let mut decision = planner_engine
                    .select_initial_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                    .await?;
                if let Some(bootstrapped) = self
                    .bootstrap_known_edit_initial_action(
                        prompt,
                        &interpretation,
                        &recent_turns,
                        gatherer.as_ref(),
                        &decision,
                    )
                    .await?
                {
                    let candidate_summary = if bootstrapped.edit.candidate_files.is_empty() {
                        "no viable candidates discovered".to_string()
                    } else {
                        format!(
                            "candidate files: {}",
                            bootstrapped.edit.candidate_files.join(", ")
                        )
                    };
                    trace.emit(TurnEvent::Fallback {
                        stage: "known-edit-bootstrap".to_string(),
                        reason: format!(
                            "known edit turn bypassed initial `{}` and forced `{}`; {}",
                            decision.action.summary(),
                            bootstrapped.action.summary(),
                            candidate_summary
                        ),
                    });
                    decision = bootstrapped;
                }
                trace.emit(TurnEvent::PlannerActionSelected {
                    sequence: 1,
                    action: decision.action.summary(),
                    rationale: decision.rationale.clone(),
                });
                trace.record_planner_action(&decision.action.summary(), &decision.rationale, None);
                execution_plan_from_initial_action(&prepared, decision)
            }
            PlannerCapability::Unsupported { reason } => {
                trace.emit(TurnEvent::Fallback {
                    stage: "planner".to_string(),
                    reason: format!("planner unavailable before first action selection: {reason}"),
                });
                fallback_execution_plan(&prepared)
            }
        };

        trace.emit(TurnEvent::IntentClassified {
            intent: execution_plan.intent.clone(),
        });
        trace.emit(TurnEvent::RouteSelected {
            summary: execution_plan.route_summary.clone(),
        });

        let gathered_evidence = match execution_plan.path {
            PromptExecutionPath::PlannerThenSynthesize => {
                let recent_turns = synthesizer_engine.recent_turn_summaries()?;

                // Resolve context artifacts using a transit-backed resolver if available.
                let resolver: Arc<dyn ContextResolver> = if let Some(transit) = self
                    .trace_recorder
                    .as_any()
                    .downcast_ref::<TransitTraceRecorder>()
                {
                    Arc::new(TransitContextResolver::new(Arc::new(transit.clone())))
                } else {
                    Arc::new(NoopContextResolver)
                };

                self.execute_recursive_planner_loop(
                    prompt,
                    PlannerLoopContext {
                        prepared: prepared.clone(),
                        planner_engine,
                        synthesizer_engine: Arc::clone(&synthesizer_engine),
                        gatherer,
                        resolver,
                        interpretation: interpretation.clone(),
                        recent_turns,
                    },
                    execution_plan.initial_planner_decision.clone(),
                    Arc::clone(&trace),
                )
                .await?
            }
            PromptExecutionPath::SynthesizerOnly => None,
        };

        let prompt = prompt.to_string();
        let intent = execution_plan.intent;
        let engine = synthesizer_engine;
        let event_sink = trace.as_event_sink();
        let session_for_reply = session.clone();
        let thread_for_reply = active_thread;
        let prompt_for_model = prompt.clone();
        let reply = tokio::task::spawn_blocking(move || {
            engine.respond_for_turn(
                &prompt_for_model,
                intent,
                gathered_evidence.as_ref(),
                event_sink,
            )
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))??;

        trace.record_completion(&reply);
        self.emit_transcript_update(&session.task_id());
        session_for_reply.note_thread_reply(&thread_for_reply, &prompt, &reply);
        Ok(reply)
    }

    async fn bootstrap_known_edit_initial_action(
        &self,
        prompt: &str,
        interpretation: &InterpretationContext,
        recent_turns: &[String],
        gatherer: Option<&Arc<dyn ContextGatherer>>,
        decision: &InitialActionDecision,
    ) -> Result<Option<InitialActionDecision>> {
        if !decision.edit.known_edit
            && !mutation_turn_requires_execution_pressure(prompt, interpretation)
        {
            return Ok(None);
        }

        let candidates = known_edit_bootstrap_candidates(
            &self.workspace_root,
            &decision.edit.candidate_files,
            prompt,
            3,
        );
        if candidates.is_empty() {
            return Ok(None);
        }

        let ranked_candidates = if let Some(gatherer) = gatherer {
            rerank_known_edit_candidates_with_vector_lookup(
                gatherer,
                &self.workspace_root,
                prompt,
                interpretation,
                recent_turns,
                &candidates,
            )
            .await
        } else {
            candidates.clone()
        };

        let best_path = ranked_candidates
            .first()
            .cloned()
            .unwrap_or_else(|| candidates[0].clone());
        Ok(Some(InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Read {
                    path: best_path.clone(),
                },
            },
            rationale: format!(
                "known edit turn; action produces information, so read `{best_path}` before broader planning"
            ),
            edit: crate::domain::ports::InitialEditInstruction {
                known_edit: decision.edit.known_edit
                    || mutation_turn_requires_execution_pressure(prompt, interpretation),
                candidate_files: ranked_candidates,
            },
        }))
    }

    pub async fn process_thread_candidate_in_session_with_sink(
        &self,
        candidate: ThreadCandidate,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let event_sink = self.wrap_sink_with_observers(event_sink);
        let runtime_guard = self.runtime.read().await;
        let runtime = runtime_guard
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Runtime lanes not initialized"))?;
        let planner_engine = Arc::clone(&runtime.planner_engine);
        let synthesizer_engine = Arc::clone(&runtime.synthesizer_engine);
        drop(runtime_guard);

        let interpretation = self
            .derive_interpretation_context(
                &candidate.prompt,
                planner_engine.as_ref(),
                event_sink.clone(),
            )
            .await;
        let source_thread = candidate.active_thread.clone();
        let turn_id = session.allocate_turn_id();
        let trace = Arc::new(StructuredTurnTrace::new(
            event_sink,
            Arc::clone(&self.trace_recorder),
            session.clone(),
            turn_id,
            source_thread.clone(),
        ));
        trace.emit(TurnEvent::ThreadCandidateCaptured {
            candidate_id: candidate.candidate_id.as_str().to_string(),
            active_thread: candidate.active_thread.stable_id(),
            prompt: candidate.prompt.clone(),
        });
        trace.record_thread_candidate(&candidate);

        let recent_turns = synthesizer_engine.recent_turn_summaries()?;
        let active_thread = session.active_thread();
        let thread_request = ThreadDecisionRequest::new(
            self.workspace_root.clone(),
            interpretation,
            active_thread.clone(),
            candidate.clone(),
        )
        .with_recent_turns(recent_turns)
        .with_known_threads(session.known_threads())
        .with_recent_thread_summary(session.recent_thread_summary(&active_thread.thread_ref));

        let decision = planner_engine
            .select_thread_decision(&thread_request, trace.clone() as Arc<dyn TurnEventSink>)
            .await?;
        trace.emit(TurnEvent::ThreadDecisionApplied {
            candidate_id: candidate.candidate_id.as_str().to_string(),
            decision: decision.kind.label().to_string(),
            target_thread: decision.target_thread.stable_id(),
            rationale: decision.rationale.clone(),
        });
        trace.record_thread_decision(&decision, &source_thread);

        let branch_id = if matches!(decision.kind, ThreadDecisionKind::OpenChildThread) {
            let branch_id = session.next_branch_id();
            trace.declare_branch(
                branch_id.clone(),
                decision
                    .new_thread_label
                    .as_deref()
                    .unwrap_or(candidate.prompt.as_str()),
                Some(decision.rationale.as_str()),
                source_thread.branch_id(),
            );
            Some(branch_id)
        } else {
            None
        };

        if matches!(decision.kind, ThreadDecisionKind::MergeIntoTarget) {
            trace.emit(TurnEvent::ThreadMerged {
                source_thread: source_thread.stable_id(),
                target_thread: decision.target_thread.stable_id(),
                mode: decision
                    .merge_mode
                    .unwrap_or(ThreadMergeMode::Summary)
                    .label()
                    .to_string(),
                summary: decision.merge_summary.clone(),
            });
            trace.record_thread_merge(&decision, &source_thread, &decision.target_thread);
        }

        session.apply_thread_decision(&decision, branch_id, &candidate.prompt);
        self.process_prompt_in_session_with_sink(
            &candidate.prompt,
            session,
            trace.downstream.clone(),
        )
        .await
    }

    async fn derive_interpretation_context(
        &self,
        prompt: &str,
        planner: &dyn RecursivePlanner,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> InterpretationContext {
        let request = InterpretationRequest::new(
            prompt,
            self.workspace_root.clone(),
            self.operator_memory
                .operator_memory_documents(&self.workspace_root),
        );
        match planner
            .derive_interpretation_context(&request, event_sink)
            .await
        {
            Ok(context) => context,
            Err(err) => {
                if self.verbose.load(Ordering::Relaxed) >= 1 {
                    println!(
                        "[WARN] Falling back to AGENTS-only interpretation context after model-driven derivation failed: {err:#}"
                    );
                }
                self.operator_memory
                    .build_interpretation_context(prompt, &self.workspace_root)
            }
        }
    }

    async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        initial_decision: Option<RecursivePlannerDecision>,
        trace: Arc<StructuredTurnTrace>,
    ) -> Result<Option<EvidenceBundle>> {
        let mut context = context;
        let budget = planner_budget_for_turn(prompt, &context.interpretation);
        let mut loop_state = PlannerLoopState::default();
        let mut used_workspace_resources = false;
        let mut stop_reason = None;
        let mut pending_initial_decision = initial_decision;
        let mut steps_without_new_evidence = 0usize;
        let gatherer_provider = context
            .prepared
            .gatherer
            .as_ref()
            .map(|lane| lane.label.clone())
            .unwrap_or_else(|| "workspace".to_string());

        for sequence in 1..=budget.max_steps {
            let evidence_count_before = loop_state.evidence_items.len();
            let mut decision = if let Some(decision) = pending_initial_decision.take() {
                decision
            } else {
                let request = PlannerRequest::new(
                    prompt,
                    self.workspace_root.clone(),
                    context.interpretation.clone(),
                    budget.clone(),
                )
                .with_recent_turns(context.recent_turns.clone())
                .with_loop_state(loop_state.clone())
                .with_resolver(context.resolver.clone());
                let decision = context
                    .planner_engine
                    .select_next_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                    .await?;
                trace.emit(TurnEvent::PlannerActionSelected {
                    sequence,
                    action: decision.action.summary(),
                    rationale: decision.rationale.clone(),
                });
                trace.record_planner_action(&decision.action.summary(), &decision.rationale, None);
                decision
            };

            if let Some(forced_decision) = coerce_execution_pressure_action(
                prompt,
                &context.interpretation,
                &loop_state,
                &decision,
                &self.workspace_root,
            ) {
                let reason = format!(
                    "execution pressure redirected `{}` to `{}` because acting on a likely file is more informative than more retrieval",
                    decision.action.summary(),
                    forced_decision.action.summary()
                );
                trace.emit(TurnEvent::Fallback {
                    stage: "execution-pressure".to_string(),
                    reason: reason.clone(),
                });
                loop_state.notes.push(reason);
                decision = forced_decision;
            }

            trace.emit(TurnEvent::PlannerStepProgress {
                step_number: sequence,
                step_limit: budget.max_steps,
                action: decision.action.summary(),
                query: decision.action.target_query(),
                evidence_count: loop_state.evidence_items.len(),
            });

            let outcome = match &decision.action {
                PlannerAction::Workspace { action } => match action {
                    WorkspaceAction::Search {
                        query,
                        mode,
                        strategy,
                        intent,
                    } => {
                        if search_steps(&loop_state) >= budget.max_searches {
                            stop_reason = Some("search-budget-exhausted".to_string());
                            "planner search budget exhausted".to_string()
                        } else if mutation_turn_requires_execution_pressure(
                            prompt,
                            &context.interpretation,
                        ) && search_steps(&loop_state) >= 1
                            && !has_file_targeting_step(&loop_state)
                        {
                            let message = "execution pressure requires reading or editing a likely file after the initial search instead of continuing broad retrieval".to_string();
                            loop_state.notes.push(message.clone());
                            message
                        } else if let Some(gatherer) = context.gatherer.as_ref() {
                            trace.emit(TurnEvent::GathererCapability {
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
                                    .with_planning(
                                        PlannerConfig::default()
                                            .with_mode(*mode)
                                            .with_retrieval_strategy(*strategy)
                                            .with_step_limit(1),
                                    )
                                    .with_prior_context(
                                        build_planner_prior_context(
                                            &context.interpretation,
                                            &context.recent_turns,
                                            &loop_state,
                                            Some(context.resolver.clone()),
                                        )
                                        .await,
                                    );
                                    gatherer.set_event_sink(Some(
                                        trace.clone() as Arc<dyn TurnEventSink>
                                    ));
                                    match gatherer.gather_context(&request).await {
                                        Ok(result) => {
                                            let bundle = result.evidence_bundle;
                                            if let Some(bundle) = bundle.as_ref() {
                                                trace.emit(TurnEvent::GathererSummary {
                                                    provider: gatherer_provider.clone(),
                                                    summary: bundle.summary.clone(),
                                                    sources: evidence_sources(
                                                        &self.workspace_root,
                                                        bundle,
                                                    ),
                                                });
                                                trace.record_selection_artifact(
                                                    TraceSelectionKind::Evidence,
                                                    bundle.summary.clone(),
                                                    evidence_sources(&self.workspace_root, bundle),
                                                    render_evidence_bundle_artifact(bundle),
                                                    ArtifactKind::EvidenceBundle,
                                                );
                                                if let Some(planner) = bundle.planner.as_ref() {
                                                    trace.emit(planner_summary_event(planner));
                                                    trace.record_selection_artifact(
                                                        TraceSelectionKind::PlannerTrace,
                                                        "planner trace".to_string(),
                                                        planner_retained_sources(planner),
                                                        render_planner_trace_artifact(planner),
                                                        ArtifactKind::PlannerTrace,
                                                    );
                                                    loop_state.latest_gatherer_trace =
                                                        Some(planner.clone());
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
                                                "planner search returned no evidence bundle"
                                                    .to_string()
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
                    WorkspaceAction::Inspect { command } => {
                        if inspect_steps(&loop_state) >= budget.max_inspects {
                            stop_reason = Some("inspect-budget-exhausted".to_string());
                            "planner inspect budget exhausted".to_string()
                        } else {
                            match run_planner_inspect_command(&self.workspace_root, command) {
                                Ok(output) => {
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
                                Err(err) => {
                                    format!("inspect failed: {err:#}")
                                }
                            }
                        }
                    }
                    WorkspaceAction::Read { .. }
                    | WorkspaceAction::ListFiles { .. }
                    | WorkspaceAction::Shell { .. }
                    | WorkspaceAction::Diff { .. }
                    | WorkspaceAction::WriteFile { .. }
                    | WorkspaceAction::ReplaceInFile { .. }
                    | WorkspaceAction::ApplyPatch { .. } => {
                        if matches!(action, WorkspaceAction::Read { .. })
                            && read_steps(&loop_state) >= budget.max_reads
                        {
                            stop_reason = Some("read-budget-exhausted".to_string());
                            "planner read budget exhausted".to_string()
                        } else {
                            let call_id = format!("planner-tool-{sequence}");
                            trace.emit(TurnEvent::ToolCalled {
                                call_id: call_id.clone(),
                                tool_name: action.label().to_string(),
                                invocation: action.describe(),
                            });
                            match context.synthesizer_engine.execute_workspace_action(action) {
                                Ok(result) => {
                                    trace.emit(TurnEvent::ToolFinished {
                                        call_id,
                                        tool_name: result.name.to_string(),
                                        summary: result.summary.clone(),
                                    });
                                    append_evidence_item(
                                        &mut loop_state.evidence_items,
                                        EvidenceItem {
                                            source: workspace_action_evidence_source(action),
                                            snippet: trim_for_planner(&result.summary, 1_200),
                                            rationale: decision.rationale.clone(),
                                            rank: 0,
                                        },
                                        budget.max_evidence_items,
                                    );
                                    used_workspace_resources = true;
                                    result.summary
                                }
                                Err(err) => {
                                    let summary =
                                        format!("Tool `{}` failed: {err:#}", action.label());
                                    trace.emit(TurnEvent::ToolFinished {
                                        call_id,
                                        tool_name: action.label().to_string(),
                                        summary: summary.clone(),
                                    });
                                    append_evidence_item(
                                        &mut loop_state.evidence_items,
                                        EvidenceItem {
                                            source: workspace_action_evidence_source(action),
                                            snippet: trim_for_planner(&summary, 1_200),
                                            rationale: decision.rationale.clone(),
                                            rank: 0,
                                        },
                                        budget.max_evidence_items,
                                    );
                                    used_workspace_resources = true;
                                    stop_reason.get_or_insert_with(|| {
                                        "workspace-action-failed".to_string()
                                    });
                                    summary
                                }
                            }
                        }
                    }
                },
                PlannerAction::Refine {
                    query,
                    mode,
                    strategy,
                    ..
                } => {
                    if search_steps(&loop_state) >= budget.max_searches {
                        stop_reason = Some("search-budget-exhausted".to_string());
                        "planner refine budget exhausted".to_string()
                    } else if mutation_turn_requires_execution_pressure(
                        prompt,
                        &context.interpretation,
                    ) && !has_file_targeting_step(&loop_state)
                    {
                        let message = "execution pressure blocks refine until the planner reads or edits a likely target file".to_string();
                        loop_state.notes.push(message.clone());
                        message
                    } else if let Some(gatherer) = context.gatherer.as_ref() {
                        trace.emit(TurnEvent::GathererCapability {
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
                                .with_planning(
                                    PlannerConfig::default()
                                        .with_mode(*mode)
                                        .with_retrieval_strategy(*strategy)
                                        .with_step_limit(1),
                                )
                                .with_prior_context(
                                    build_planner_prior_context(
                                        &context.interpretation,
                                        &context.recent_turns,
                                        &loop_state,
                                        Some(context.resolver.clone()),
                                    )
                                    .await,
                                );
                                gatherer
                                    .set_event_sink(Some(trace.clone() as Arc<dyn TurnEventSink>));
                                match gatherer.gather_context(&request).await {
                                    Ok(result) => {
                                        let bundle = result.evidence_bundle;
                                        if let Some(bundle) = bundle.as_ref() {
                                            trace.emit(TurnEvent::GathererSummary {
                                                provider: gatherer_provider.clone(),
                                                summary: bundle.summary.clone(),
                                                sources: evidence_sources(
                                                    &self.workspace_root,
                                                    bundle,
                                                ),
                                            });
                                            trace.record_selection_artifact(
                                                TraceSelectionKind::Evidence,
                                                bundle.summary.clone(),
                                                evidence_sources(&self.workspace_root, bundle),
                                                render_evidence_bundle_artifact(bundle),
                                                ArtifactKind::EvidenceBundle,
                                            );
                                            if let Some(planner) = bundle.planner.as_ref() {
                                                trace.emit(planner_summary_event(planner));
                                                trace.record_selection_artifact(
                                                    TraceSelectionKind::PlannerTrace,
                                                    "planner trace".to_string(),
                                                    planner_retained_sources(planner),
                                                    render_planner_trace_artifact(planner),
                                                    ArtifactKind::PlannerTrace,
                                                );
                                                loop_state.latest_gatherer_trace =
                                                    Some(planner.clone());
                                            }
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
                        let exists = loop_state
                            .pending_branches
                            .iter()
                            .any(|pending| pending.label == *branch);
                        if !exists {
                            let branch_id = trace.session.next_branch_id();
                            let branch_trace = trace.declare_branch(
                                branch_id,
                                branch,
                                Some(decision.rationale.as_str()),
                                None,
                            );
                            loop_state.pending_branches.push(branch_trace);
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
                step_id: format!("planner-step-{sequence}"),
                sequence,
                branch_id: None,
                action: decision.action.clone(),
                outcome: outcome.clone(),
            });

            let evidence_count_after = loop_state.evidence_items.len();
            if evidence_count_after > evidence_count_before {
                steps_without_new_evidence = 0;
            } else {
                steps_without_new_evidence += 1;
            }

            if stop_reason.is_none() && !matches!(decision.action, PlannerAction::Stop { .. }) {
                let refinement_reason = self.mid_loop_refinement_reason(
                    sequence,
                    &loop_state,
                    steps_without_new_evidence,
                );
                if let Some(refinement_reason) = refinement_reason
                    && let Some(updated_context) = self
                        .derive_mid_loop_interpretation_context(
                            prompt,
                            &context,
                            &loop_state,
                            &evidence_count_before,
                            &trace,
                        )
                        .await
                    && updated_context != context.interpretation
                {
                    let before_summary = context.interpretation.summary.clone();
                    let after_summary = updated_context.summary.clone();
                    loop_state.refinement_count += 1;
                    loop_state.last_refinement_step = Some(sequence);
                    let refinement_signature =
                        Self::mid_loop_refinement_signature(&updated_context);
                    if !self.mid_loop_refinement_signature_is_stable(
                        &loop_state.refinement_policy,
                        &loop_state.refinement_signatures,
                        &refinement_signature,
                    ) {
                        trace.emit(TurnEvent::Fallback {
                            stage: "refinement-guard".to_string(),
                            reason:
                                "oscillation guard prevented refinement to recently seen interpretation signature"
                                    .to_string(),
                        });
                        continue;
                    }
                    loop_state
                        .refinement_signatures
                        .push(refinement_signature.clone());
                    if loop_state.refinement_policy.signature_history_limit > 0 {
                        let limit = loop_state.refinement_policy.signature_history_limit;
                        if loop_state.refinement_signatures.len() > limit {
                            let overflow =
                                loop_state.refinement_signatures.len().saturating_sub(limit);
                            loop_state.refinement_signatures.drain(0..overflow);
                        }
                    }
                    context.interpretation = updated_context;
                    trace.emit(TurnEvent::RefinementApplied {
                        reason: refinement_reason,
                        before_summary,
                        after_summary,
                    });
                }
            }

            if let PlannerAction::Stop { .. } = decision.action {
                break;
            }

            if stop_reason.is_some() {
                break;
            }
        }

        let completed = stop_reason.is_some();
        let stop_reason = stop_reason.unwrap_or_else(|| "planner-budget-exhausted".to_string());
        trace.emit(TurnEvent::PlannerSummary {
            strategy: "model-driven".to_string(),
            mode: loop_state
                .latest_gatherer_trace
                .as_ref()
                .map(|planner| planner.mode.label().to_string())
                .unwrap_or_else(|| RetrievalMode::Linear.label().to_string()),
            turns: loop_state.steps.len(),
            steps: loop_state.steps.len(),
            stop_reason: Some(stop_reason.clone()),
            active_branch_id: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .and_then(|graph| graph.active_branch_id.clone()),
            branch_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.branches.len()),
            frontier_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.frontier.len()),
            node_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.nodes.len()),
            edge_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .and_then(|planner| planner.graph_episode.as_ref())
                .map(|graph| graph.edges.len()),
            retained_artifact_count: loop_state
                .latest_gatherer_trace
                .as_ref()
                .map(|planner| planner.retained_artifacts.len()),
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

    fn mid_loop_refinement_reason(
        &self,
        sequence: usize,
        loop_state: &PlannerLoopState,
        steps_without_new_evidence: usize,
    ) -> Option<String> {
        let policy = &loop_state.refinement_policy;
        if !policy.enabled {
            return None;
        }

        if loop_state.refinement_count >= policy.max_refinements_per_turn {
            return None;
        }

        if let Some(last_refinement_step) = loop_state.last_refinement_step {
            let steps_since_last_refinement = sequence.saturating_sub(last_refinement_step);
            if steps_since_last_refinement <= policy.cooldown_steps {
                return None;
            }
        }

        if matches!(
            policy.trigger.source,
            crate::domain::ports::RefinementTriggerSource::Manual
        ) {
            return None;
        }

        if loop_state.evidence_items.len() < policy.trigger.min_evidence_items {
            return None;
        }

        if steps_without_new_evidence >= policy.trigger.min_steps_without_new_evidence {
            Some(format!(
                "evidence-pressure ({} evidence items after {} quiet steps)",
                loop_state.evidence_items.len(),
                steps_without_new_evidence
            ))
        } else {
            None
        }
    }

    fn mid_loop_refinement_signature(context: &InterpretationContext) -> String {
        let signature_summary: String = context.summary.chars().take(240).collect();
        format!(
            "{}::docs={}",
            signature_summary.replace('\n', " "),
            context.documents.len()
        )
    }

    fn mid_loop_refinement_signature_is_stable(
        &self,
        policy: &crate::domain::ports::RefinementPolicy,
        signatures: &[String],
        signature: &str,
    ) -> bool {
        if policy.oscillation_signature_window == 0 {
            return true;
        }

        let window = policy
            .oscillation_signature_window
            .min(policy.signature_history_limit.max(1));
        let prior_recurrence = signatures
            .iter()
            .rev()
            .take(window)
            .filter(|entry| entry == &signature)
            .count();
        prior_recurrence == 0
    }

    async fn derive_mid_loop_interpretation_context(
        &self,
        prompt: &str,
        context: &PlannerLoopContext,
        loop_state: &PlannerLoopState,
        _evidence_count_before: &usize,
        trace: &Arc<StructuredTurnTrace>,
    ) -> Option<InterpretationContext> {
        let mut documents = self
            .operator_memory
            .operator_memory_documents(&self.workspace_root);
        documents.push(crate::domain::ports::OperatorMemoryDocument {
            path: self.workspace_root.join(".paddles/refinement-context.md"),
            source: "planner-loop-context".to_string(),
            contents: format!(
                "Interpretation context before refinement:\n{}",
                context.interpretation.render()
            ),
        });

        let evidence_snapshot = loop_state
            .evidence_items
            .iter()
            .rev()
            .take(3)
            .map(|evidence| format!("{}: {}", evidence.source, evidence.snippet))
            .collect::<Vec<_>>()
            .join("\n");
        if !evidence_snapshot.is_empty() {
            documents.push(crate::domain::ports::OperatorMemoryDocument {
                path: self.workspace_root.join(".paddles/refinement-evidence.md"),
                source: "planner-loop-evidence".to_string(),
                contents: format!("Recent evidence:\n{evidence_snapshot}"),
            });
        }

        let request = InterpretationRequest::new(prompt, self.workspace_root.clone(), documents);
        match context
            .planner_engine
            .derive_interpretation_context(&request, trace.clone() as Arc<dyn TurnEventSink>)
            .await
        {
            Ok(updated_context) => Some(updated_context),
            Err(err) => {
                if self.verbose.load(Ordering::Relaxed) >= 1 {
                    println!(
                        "[WARN] mid-loop refinement failed, keeping prior interpretation context: {err:#}"
                    );
                }
                None
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PromptExecutionPlan {
    intent: TurnIntent,
    path: PromptExecutionPath,
    route_summary: String,
    initial_planner_decision: Option<RecursivePlannerDecision>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptExecutionPath {
    SynthesizerOnly,
    PlannerThenSynthesize,
}

fn fallback_execution_plan(prepared: &PreparedRuntimeLanes) -> PromptExecutionPlan {
    PromptExecutionPlan {
        intent: TurnIntent::DirectResponse,
        path: PromptExecutionPath::SynthesizerOnly,
        route_summary: format!(
            "planner lane '{}' is unavailable, so the turn will fall back to synthesizer lane '{}' for a direct response",
            prepared.planner.model_id, prepared.synthesizer.model_id
        ),
        initial_planner_decision: None,
    }
}

fn execution_plan_from_initial_action(
    prepared: &PreparedRuntimeLanes,
    decision: InitialActionDecision,
) -> PromptExecutionPlan {
    let InitialActionDecision {
        action,
        rationale,
        edit: _,
    } = decision;
    match action {
        InitialAction::Answer => PromptExecutionPlan {
            intent: TurnIntent::DirectResponse,
            path: PromptExecutionPath::SynthesizerOnly,
            route_summary: format!(
                "model selected a direct response on synthesizer lane '{}'",
                prepared.synthesizer.model_id
            ),
            initial_planner_decision: None,
        },
        InitialAction::Stop { reason } => PromptExecutionPlan {
            intent: TurnIntent::DirectResponse,
            path: PromptExecutionPath::SynthesizerOnly,
            route_summary: format!(
                "model selected stop before recursive resource use ({reason}); synthesizer lane '{}' will answer directly",
                prepared.synthesizer.model_id
            ),
            initial_planner_decision: None,
        },
        resource_action => {
            let planner_action = resource_action
                .as_planner_action()
                .expect("resource action must map to planner action");
            let route_summary = if let Some(gatherer_lane) = &prepared.gatherer {
                format!(
                    "model selected initial planner action {}; turn will use planner lane '{}' with gatherer backend '{}' ({:?}) before synthesizer lane '{}'",
                    planner_action.summary(),
                    prepared.planner.model_id,
                    gatherer_lane.label,
                    gatherer_lane.provider,
                    prepared.synthesizer.model_id
                )
            } else {
                format!(
                    "model selected initial planner action {}; turn will use planner lane '{}' and synthesizer lane '{}' with no dedicated gatherer backend configured",
                    planner_action.summary(),
                    prepared.planner.model_id,
                    prepared.synthesizer.model_id
                )
            };

            PromptExecutionPlan {
                intent: TurnIntent::Planned,
                path: PromptExecutionPath::PlannerThenSynthesize,
                route_summary,
                initial_planner_decision: Some(RecursivePlannerDecision {
                    action: planner_action,
                    rationale,
                }),
            }
        }
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

fn planner_summary_event(planner: &PlannerTraceMetadata) -> TurnEvent {
    TurnEvent::PlannerSummary {
        strategy: format_planner_strategy(&planner.strategy).to_string(),
        mode: planner.mode.label().to_string(),
        turns: planner.turn_count,
        steps: planner.steps.len(),
        stop_reason: planner.stop_reason.clone(),
        active_branch_id: planner
            .graph_episode
            .as_ref()
            .and_then(|graph| graph.active_branch_id.clone()),
        branch_count: planner
            .graph_episode
            .as_ref()
            .map(|graph| graph.branches.len()),
        frontier_count: planner
            .graph_episode
            .as_ref()
            .map(|graph| graph.frontier.len()),
        node_count: planner
            .graph_episode
            .as_ref()
            .map(|graph| graph.nodes.len()),
        edge_count: planner
            .graph_episode
            .as_ref()
            .map(|graph| graph.edges.len()),
        retained_artifact_count: Some(planner.retained_artifacts.len()),
    }
}

async fn build_planner_prior_context(
    interpretation: &InterpretationContext,
    recent_turns: &[String],
    loop_state: &PlannerLoopState,
    resolver: Option<Arc<dyn ContextResolver>>,
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
            .map(|branch| format!("pending branch: {}", branch.summary())),
    );

    // Pull context on demand for retained evidence from autonomous gatherers.
    for artifact in &loop_state.evidence_items {
        prior.push(format!(
            "evidence ({}): {}",
            artifact.source, artifact.snippet
        ));
    }

    if let Some(trace) = &loop_state.latest_gatherer_trace {
        for artifact in &trace.retained_artifacts {
            if let Some(snippet) = &artifact.snippet {
                prior.push(format!(
                    "retained evidence ({}): {}",
                    artifact.source, snippet
                ));
            } else if let (Some(resolver), Some(locator)) = (&resolver, &artifact.locator) {
                // On-demand resolution of truncated artifacts.
                if let Ok(content) = resolver.resolve(locator).await {
                    prior.push(format!(
                        "retained evidence (resolved from {}): {}",
                        artifact.source, content
                    ));
                } else {
                    prior.push(format!(
                        "retained evidence ({}): [locator resolution failed]",
                        artifact.source
                    ));
                }
            } else {
                prior.push(format!(
                    "retained evidence ({}): [truncated, no locator available]",
                    artifact.source
                ));
            }
        }
    }

    prior
}

fn planner_budget_for_turn(prompt: &str, interpretation: &InterpretationContext) -> PlannerBudget {
    if mutation_turn_requires_execution_pressure(prompt, interpretation) {
        PlannerBudget {
            max_steps: 8,
            max_reads: 4,
            max_inspects: 2,
            max_searches: 1,
            ..PlannerBudget::default()
        }
    } else {
        PlannerBudget::default()
    }
}

fn mutation_turn_requires_execution_pressure(
    prompt: &str,
    interpretation: &InterpretationContext,
) -> bool {
    let prompt_lower = prompt.to_ascii_lowercase();
    let prompt_signals = [
        "edit ",
        "change ",
        "update ",
        "modify ",
        "implement ",
        "fix ",
        "refactor ",
        "rename ",
        "make ",
        "render ",
        "show ",
        "add ",
        "remove ",
    ];
    if prompt_signals
        .iter()
        .any(|signal| prompt_lower.contains(signal))
    {
        return true;
    }

    interpretation.tool_hints.iter().any(|hint| {
        matches!(
            hint.action,
            WorkspaceAction::WriteFile { .. }
                | WorkspaceAction::ReplaceInFile { .. }
                | WorkspaceAction::ApplyPatch { .. }
        )
    })
}

fn known_edit_bootstrap_candidates(
    workspace_root: &Path,
    hinted_paths: &[String],
    prompt: &str,
    limit: usize,
) -> Vec<String> {
    const MIN_EDIT_TARGET_SCORE: i32 = 60;
    let normalized_hints = hinted_paths
        .iter()
        .filter_map(|path| normalize_known_edit_candidate(workspace_root, path))
        .collect::<Vec<_>>();
    let tokens = known_edit_search_tokens(prompt, &normalized_hints);
    let mut scored = HashMap::<String, i32>::new();

    for (index, path) in normalized_hints.iter().enumerate() {
        scored.insert(
            path.clone(),
            400 + score_execution_pressure_path(path) - index as i32 * 10,
        );
    }

    visit_known_edit_candidates(
        workspace_root,
        workspace_root,
        &tokens,
        &normalized_hints,
        &mut scored,
        &mut 0,
    );

    let mut ranked = scored.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|(path_a, score_a), (path_b, score_b)| {
        score_b.cmp(score_a).then_with(|| path_a.cmp(path_b))
    });
    ranked
        .into_iter()
        .filter(|(_, score)| *score >= MIN_EDIT_TARGET_SCORE)
        .take(limit)
        .map(|(path, _)| path)
        .collect()
}

async fn rerank_known_edit_candidates_with_vector_lookup(
    gatherer: &Arc<dyn ContextGatherer>,
    workspace_root: &Path,
    prompt: &str,
    interpretation: &InterpretationContext,
    recent_turns: &[String],
    candidates: &[String],
) -> Vec<String> {
    if !matches!(gatherer.capability(), GathererCapability::Available) {
        return candidates.to_vec();
    }

    let mut prior_context = Vec::new();
    prior_context.push(format!(
        "known-edit candidate files under review: {}",
        candidates.join(", ")
    ));
    if !interpretation.is_empty() {
        prior_context.push(interpretation.render());
    }
    prior_context.extend(recent_turns.iter().take(2).cloned());

    let request = ContextGatherRequest::new(
        prompt,
        workspace_root.to_path_buf(),
        "known-edit-bootstrap",
        EvidenceBudget::default(),
    )
    .with_planning(
        PlannerConfig::default()
            .with_mode(RetrievalMode::Linear)
            .with_retrieval_strategy(RetrievalStrategy::Vector)
            .with_step_limit(1),
    )
    .with_prior_context(prior_context);

    let Ok(result) = gatherer.gather_context(&request).await else {
        return candidates.to_vec();
    };

    let Some(bundle) = result.evidence_bundle else {
        return candidates.to_vec();
    };

    let mut scored = HashMap::<String, i32>::new();
    for (index, path) in candidates.iter().enumerate() {
        scored.insert(path.clone(), 300 - index as i32 * 10);
    }

    for item in &bundle.items {
        let Some(path) = normalize_execution_pressure_source(&item.source, workspace_root) else {
            continue;
        };
        if let Some(score) = scored.get_mut(&path) {
            *score += 90 + evidence_rank_bonus(item.rank);
        }
    }

    if let Some(trace) = &bundle.planner {
        for (index, artifact) in trace.retained_artifacts.iter().enumerate() {
            let Some(path) = normalize_execution_pressure_source(&artifact.source, workspace_root)
            else {
                continue;
            };
            if let Some(score) = scored.get_mut(&path) {
                *score += 60 - index as i32 * 5;
            }
        }
    }

    let mut ranked = scored.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|(path_a, score_a), (path_b, score_b)| {
        score_b.cmp(score_a).then_with(|| path_a.cmp(path_b))
    });
    ranked.into_iter().map(|(path, _)| path).collect()
}

fn search_steps(loop_state: &PlannerLoopState) -> usize {
    loop_state
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step.action,
                PlannerAction::Workspace {
                    action: WorkspaceAction::Search { .. }
                } | PlannerAction::Refine { .. }
            )
        })
        .count()
}

fn has_file_targeting_step(loop_state: &PlannerLoopState) -> bool {
    loop_state.steps.iter().any(|step| {
        matches!(
            step.action,
            PlannerAction::Workspace {
                action: WorkspaceAction::Read { .. }
                    | WorkspaceAction::ListFiles { .. }
                    | WorkspaceAction::Diff { .. }
                    | WorkspaceAction::WriteFile { .. }
                    | WorkspaceAction::ReplaceInFile { .. }
                    | WorkspaceAction::ApplyPatch { .. }
            }
        )
    })
}

fn coerce_execution_pressure_action(
    prompt: &str,
    interpretation: &InterpretationContext,
    loop_state: &PlannerLoopState,
    decision: &RecursivePlannerDecision,
    workspace_root: &Path,
) -> Option<RecursivePlannerDecision> {
    if !mutation_turn_requires_execution_pressure(prompt, interpretation)
        || has_file_targeting_step(loop_state)
    {
        return None;
    }

    if decision_targets_file(&decision.action) {
        return None;
    }

    let path = likely_execution_pressure_targets(loop_state, workspace_root, 3)
        .into_iter()
        .next()
        .or_else(|| {
            known_edit_bootstrap_candidates(workspace_root, &[], prompt, 3)
                .into_iter()
                .next()
        })?;
    Some(RecursivePlannerDecision {
        action: PlannerAction::Workspace {
            action: WorkspaceAction::Read { path: path.clone() },
        },
        rationale: format!(
            "action produces information; read likely target file `{path}` before further non-file actions"
        ),
    })
}

fn decision_targets_file(action: &PlannerAction) -> bool {
    matches!(
        action,
        PlannerAction::Workspace {
            action: WorkspaceAction::Read { .. }
                | WorkspaceAction::ListFiles { .. }
                | WorkspaceAction::Diff { .. }
                | WorkspaceAction::WriteFile { .. }
                | WorkspaceAction::ReplaceInFile { .. }
                | WorkspaceAction::ApplyPatch { .. }
        }
    )
}

fn likely_execution_pressure_targets(
    loop_state: &PlannerLoopState,
    workspace_root: &Path,
    limit: usize,
) -> Vec<String> {
    let mut scored = HashMap::<String, i32>::new();

    for item in &loop_state.evidence_items {
        let Some(path) = normalize_execution_pressure_source(&item.source, workspace_root) else {
            continue;
        };
        let score = score_execution_pressure_path(&path) + evidence_rank_bonus(item.rank);
        match scored.entry(path) {
            Entry::Occupied(mut entry) => {
                if score > *entry.get() {
                    entry.insert(score);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(score);
            }
        }
    }

    if let Some(trace) = &loop_state.latest_gatherer_trace {
        for (index, artifact) in trace.retained_artifacts.iter().enumerate() {
            let Some(path) = normalize_execution_pressure_source(&artifact.source, workspace_root)
            else {
                continue;
            };
            let score = score_execution_pressure_path(&path) + (40 - index as i32 * 5);
            match scored.entry(path) {
                Entry::Occupied(mut entry) => {
                    if score > *entry.get() {
                        entry.insert(score);
                    }
                }
                Entry::Vacant(entry) => {
                    entry.insert(score);
                }
            }
        }
    }

    let mut ranked = scored.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|(path_a, score_a), (path_b, score_b)| {
        score_b.cmp(score_a).then_with(|| path_a.cmp(path_b))
    });
    ranked
        .into_iter()
        .take(limit)
        .map(|(path, _)| path)
        .collect()
}

fn normalize_known_edit_candidate(workspace_root: &Path, path: &str) -> Option<String> {
    let normalized = normalize_execution_pressure_source(path, workspace_root)?;
    workspace_root
        .join(&normalized)
        .is_file()
        .then_some(normalized)
}

fn visit_known_edit_candidates(
    dir: &Path,
    workspace_root: &Path,
    tokens: &[String],
    normalized_hints: &[String],
    scored: &mut HashMap<String, i32>,
    visited_files: &mut usize,
) {
    if *visited_files >= 512 {
        return;
    }

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        if *visited_files >= 512 {
            break;
        }

        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            if matches!(name.as_str(), ".git" | "target" | ".direnv") {
                continue;
            }
            visit_known_edit_candidates(
                &path,
                workspace_root,
                tokens,
                normalized_hints,
                scored,
                visited_files,
            );
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        *visited_files += 1;
        let rel = path
            .strip_prefix(workspace_root)
            .ok()
            .map(|relative| relative.to_string_lossy().replace('\\', "/"));
        let Some(rel) = rel else {
            continue;
        };
        if !is_plausible_workspace_file(&rel) {
            continue;
        }

        let score = known_edit_candidate_score(&rel, tokens, normalized_hints);
        match scored.entry(rel) {
            Entry::Occupied(mut entry) => {
                if score > *entry.get() {
                    entry.insert(score);
                }
            }
            Entry::Vacant(entry) => {
                entry.insert(score);
            }
        }
    }
}

fn known_edit_candidate_score(path: &str, tokens: &[String], normalized_hints: &[String]) -> i32 {
    let mut score = score_execution_pressure_path(path);
    let path_lower = path.to_ascii_lowercase();

    for (index, hint) in normalized_hints.iter().enumerate() {
        if path == hint {
            score += 250 - index as i32 * 10;
            continue;
        }

        let hint_lower = hint.to_ascii_lowercase();
        if path_lower.contains(&hint_lower) || hint_lower.contains(&path_lower) {
            score += 80 - index as i32 * 5;
        }
    }

    for token in tokens {
        if path_lower.contains(token) {
            score += 25;
        }
    }

    score
}

fn known_edit_search_tokens(prompt: &str, hinted_paths: &[String]) -> Vec<String> {
    let mut tokens = Vec::new();
    for source in std::iter::once(prompt).chain(hinted_paths.iter().map(String::as_str)) {
        for token in source
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .map(|token| token.trim().to_ascii_lowercase())
            .filter(|token| token.len() >= 3)
        {
            if matches!(
                token.as_str(),
                "the"
                    | "and"
                    | "for"
                    | "with"
                    | "from"
                    | "that"
                    | "this"
                    | "turn"
                    | "file"
                    | "files"
                    | "edit"
                    | "code"
                    | "best"
                    | "then"
                    | "just"
                    | "start"
                    | "next"
            ) {
                continue;
            }
            if !tokens.contains(&token) {
                tokens.push(token);
            }
        }
    }
    tokens
}

fn normalize_execution_pressure_source(source: &str, workspace_root: &Path) -> Option<String> {
    if source.trim().is_empty() || source.starts_with("command: ") {
        return None;
    }

    let path = Path::new(source);
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root).ok()?.to_path_buf()
    } else {
        PathBuf::from(source)
    };

    if relative
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return None;
    }

    let path_text = relative.to_string_lossy().replace('\\', "/");
    is_plausible_workspace_file(&path_text).then_some(path_text)
}

fn is_plausible_workspace_file(path: &str) -> bool {
    if path.is_empty() || path.ends_with('/') {
        return false;
    }

    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some()
}

fn evidence_rank_bonus(rank: usize) -> i32 {
    if rank == 0 {
        5
    } else {
        30i32.saturating_sub(rank as i32 * 3)
    }
}

fn score_execution_pressure_path(path: &str) -> i32 {
    let extension = Path::new(path).extension().and_then(|ext| ext.to_str());
    let mut score = match extension {
        Some("rs" | "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte") => 80,
        Some("html" | "css" | "json" | "toml" | "yml" | "yaml") => 45,
        Some("md" | "txt") => 10,
        Some(_) => 20,
        None => 0,
    };

    if path.starts_with("src/") {
        score += 30;
    } else if path.starts_with("crates/") {
        score += 20;
    }
    if path.contains("/test") || path.contains("/tests/") || path.ends_with("_test.rs") {
        score -= 10;
    }
    if path.ends_with("README.md") || path.ends_with("AGENTS.md") {
        score -= 20;
    }

    score
}

#[allow(dead_code)]
struct CompactionEngine {
    resolver: Arc<dyn ContextResolver>,
}

#[allow(dead_code)]
impl CompactionEngine {
    fn new(resolver: Arc<dyn ContextResolver>) -> Self {
        Self { resolver }
    }

    async fn execute(
        &self,
        artifacts: Vec<RetainedEvidence>,
        plan: CompactionPlan,
    ) -> Vec<RetainedEvidence> {
        let mut compacted = Vec::new();

        for mut artifact in artifacts {
            // We need a way to correlate RetainedEvidence back to artifacts in the plan.
            // For now, we'll assume we can use the source as a key if artifact_id isn't available,
            // but ideally RetainedEvidence should carry its TraceArtifactId.
            // Since we added locator to RetainedEvidence, we'll use that if present.
            let artifact_id =
                if let Some(ContextLocator::Transit { record_id, .. }) = &artifact.locator {
                    // This is a simplification
                    Some(TraceArtifactId::new(record_id.as_str()).unwrap())
                } else {
                    None
                };

            let decision = artifact_id.and_then(|id| plan.decisions.get(&id));

            match decision {
                Some(CompactionDecision::Keep { .. }) | None => {
                    compacted.push(artifact);
                }
                Some(CompactionDecision::Compact { summary }) => {
                    artifact.snippet = Some(summary.clone());
                    // The locator is preserved, pointing to the original source.
                    compacted.push(artifact);
                }
                Some(CompactionDecision::Discard { .. }) => {
                    // Dropped from context
                }
            }
        }

        compacted
    }
}

fn read_steps(loop_state: &PlannerLoopState) -> usize {
    loop_state
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step.action,
                PlannerAction::Workspace {
                    action: WorkspaceAction::Read { .. }
                }
            )
        })
        .count()
}

fn inspect_steps(loop_state: &PlannerLoopState) -> usize {
    loop_state
        .steps
        .iter()
        .filter(|step| {
            matches!(
                step.action,
                PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect { .. }
                }
            )
        })
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
    Ok(())
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
    let latest_gatherer_trace = loop_state.latest_gatherer_trace.clone();
    let summary = format!(
        "Planner lane `{}` executed {} step(s) for `{}` and collected {} evidence item(s); stop reason: {}.",
        prepared.planner.model_id,
        loop_state.steps.len(),
        prompt,
        loop_state.evidence_items.len(),
        stop_reason
    );
    let planner = PlannerTraceMetadata {
        mode: latest_gatherer_trace
            .as_ref()
            .map(|planner| planner.mode)
            .unwrap_or_default(),
        strategy: PlannerStrategyKind::ModelDriven,
        profile: Some(prepared.planner.model_id.clone()),
        session_id: latest_gatherer_trace
            .as_ref()
            .and_then(|planner| planner.session_id.clone()),
        completed,
        stop_reason: Some(stop_reason.to_string()),
        turn_count: loop_state.steps.len(),
        steps: loop_state
            .steps
            .iter()
            .map(|step| PlannerTraceStep {
                step_id: step.step_id.clone(),
                sequence: step.sequence,
                parent_step_id: None,
                decisions: vec![crate::domain::ports::PlannerDecision {
                    action: step.action.label().to_string(),
                    query: planner_action_query(&step.action),
                    rationale: Some(step.outcome.clone()),
                    next_step_id: None,
                    turn_id: None,
                    branch_id: None,
                    node_id: None,
                    target_branch_id: None,
                    target_node_id: None,
                    edge_id: None,
                    edge_kind: None,
                    frontier_id: None,
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
                locator: None,
            })
            .collect(),
        graph_episode: latest_gatherer_trace.and_then(|planner| planner.graph_episode),
        trace_artifact_ref: None,
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
        PlannerAction::Workspace { action } => match action {
            WorkspaceAction::Search { query, .. } => Some(query.clone()),
            WorkspaceAction::ListFiles { pattern } => pattern.clone(),
            WorkspaceAction::Read { path } => Some(path.clone()),
            WorkspaceAction::Inspect { command } => Some(command.clone()),
            WorkspaceAction::Shell { command } => Some(command.clone()),
            WorkspaceAction::Diff { path } => path
                .clone()
                .or_else(|| Some("git diff --no-ext-diff".to_string())),
            WorkspaceAction::WriteFile { path, .. } => Some(path.clone()),
            WorkspaceAction::ReplaceInFile { path, .. } => Some(path.clone()),
            WorkspaceAction::ApplyPatch { .. } => {
                Some("git apply --whitespace=nowarn -".to_string())
            }
        },
        PlannerAction::Refine { query, .. } => Some(query.clone()),
        PlannerAction::Branch { branches, .. } => Some(branches.join(" | ")),
        PlannerAction::Stop { reason } => Some(reason.clone()),
    }
}

fn workspace_action_evidence_source(action: &WorkspaceAction) -> String {
    match action {
        WorkspaceAction::Search { query, .. } => format!("search: {query}"),
        WorkspaceAction::ListFiles { pattern } => match pattern {
            Some(pattern) => format!("list_files: {pattern}"),
            None => "list_files".to_string(),
        },
        WorkspaceAction::Read { path } => path.clone(),
        WorkspaceAction::Inspect { command } => format!("command: {command}"),
        WorkspaceAction::Shell { command } => format!("command: {command}"),
        WorkspaceAction::Diff { path } => match path {
            Some(path) => format!("diff: {path}"),
            None => "git diff --no-ext-diff".to_string(),
        },
        WorkspaceAction::WriteFile { path, .. } => path.clone(),
        WorkspaceAction::ReplaceInFile { path, .. } => path.clone(),
        WorkspaceAction::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
    }
}

fn render_evidence_bundle_artifact(bundle: &EvidenceBundle) -> String {
    let mut lines = vec![bundle.summary.clone()];
    for item in &bundle.items {
        lines.push(format!(
            "- {}: {}",
            item.source,
            trim_for_planner(&item.snippet, 240)
        ));
    }
    if !bundle.warnings.is_empty() {
        lines.push("Warnings:".to_string());
        for warning in &bundle.warnings {
            lines.push(format!("- {}", trim_for_planner(warning, 160)));
        }
    }
    lines.join("\n")
}

fn render_planner_trace_artifact(planner: &PlannerTraceMetadata) -> String {
    let mut lines = vec![format!(
        "strategy={}, mode={}, turns={}, steps={}, stop={}",
        format_planner_strategy(&planner.strategy),
        planner.mode.label(),
        planner.turn_count,
        planner.steps.len(),
        planner.stop_reason.as_deref().unwrap_or("none"),
    )];
    for step in &planner.steps {
        lines.push(format!(
            "- {}#{} parent={}",
            step.step_id,
            step.sequence,
            step.parent_step_id.as_deref().unwrap_or("none")
        ));
        for decision in &step.decisions {
            lines.push(format!(
                "  action={} query={} branch={} stop={}",
                decision.action,
                decision.query.as_deref().unwrap_or("none"),
                decision.branch_id.as_deref().unwrap_or("none"),
                decision.stop_reason.as_deref().unwrap_or("none")
            ));
        }
    }
    lines.join("\n")
}

fn planner_retained_sources(planner: &PlannerTraceMetadata) -> Vec<String> {
    planner
        .retained_artifacts
        .iter()
        .map(|artifact| artifact.source.clone())
        .collect()
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
    use crate::application::{
        ActiveRuntimeState, GathererProvider, MechSuitService, PreparedGathererLane,
        PreparedModelLane, PreparedRuntimeLanes, RuntimeLaneConfig, RuntimeLaneRole, TurnIntent,
    };
    use crate::domain::model::{CompactionPlan, CompactionRequest};
    use crate::domain::model::{
        ConversationTranscriptUpdate, ThreadDecision, ThreadDecisionId, ThreadDecisionKind,
        TraceRecordKind, TranscriptUpdateSink, TurnEvent, TurnEventSink,
    };
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBundle, EvidenceItem,
        InitialAction, InitialActionDecision, InitialEditInstruction, InterpretationContext,
        InterpretationRequest, ModelPaths, ModelRegistry, PlannerAction, PlannerCapability,
        PlannerGraphBranch, PlannerGraphBranchStatus, PlannerGraphEpisode, PlannerLoopState,
        PlannerRequest, PlannerStrategyKind, PlannerTraceMetadata, RecursivePlanner,
        RecursivePlannerDecision, RetainedEvidence, RetrievalMode, SynthesizerEngine,
        ThreadDecisionRequest, TraceRecorder, WorkspaceAction,
    };
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
    use crate::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
    use crate::infrastructure::providers::ModelProvider;
    use anyhow::{Result, anyhow};
    use async_trait::async_trait;
    use sift::Conversation;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    fn test_service(workspace: &Path) -> MechSuitService {
        let operator_memory = Arc::new(AgentMemory::load(workspace));
        MechSuitService::new(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            Box::new(|_, _| Err(anyhow!("synthesizer factory not used in this test"))),
            Box::new(|_, _| Err(anyhow!("planner factory not used in this test"))),
            Box::new(|_, _, _, _| Err(anyhow!("gatherer factory not used in this test"))),
        )
    }

    fn test_service_with_recorder(
        workspace: &Path,
        recorder: Arc<dyn TraceRecorder>,
    ) -> MechSuitService {
        let operator_memory = Arc::new(AgentMemory::load(workspace));
        MechSuitService::with_trace_recorder(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            Box::new(|_, _| Err(anyhow!("synthesizer factory not used in this test"))),
            Box::new(|_, _| Err(anyhow!("planner factory not used in this test"))),
            Box::new(|_, _, _, _| Err(anyhow!("gatherer factory not used in this test"))),
            recorder,
        )
    }

    #[derive(Default)]
    struct StaticRegistry;

    #[async_trait]
    impl ModelRegistry for StaticRegistry {
        async fn get_model_paths(&self, _model_id: &str) -> Result<ModelPaths> {
            Err(anyhow!("test registry is not used in this suite"))
        }
    }

    #[derive(Default)]
    struct RecordingRegistry {
        requested_model_ids: Mutex<Vec<String>>,
    }

    impl RecordingRegistry {
        fn requested_model_ids(&self) -> Vec<String> {
            self.requested_model_ids
                .lock()
                .expect("requested model ids lock")
                .clone()
        }
    }

    #[async_trait]
    impl ModelRegistry for RecordingRegistry {
        async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths> {
            self.requested_model_ids
                .lock()
                .expect("requested model ids lock")
                .push(model_id.to_string());
            Ok(sample_model_paths(model_id))
        }
    }

    struct TestPlanner {
        initial_decision: InitialActionDecision,
        next_decisions: Mutex<VecDeque<RecursivePlannerDecision>>,
        recorded_requests: Arc<Mutex<Vec<PlannerRequest>>>,
    }

    impl TestPlanner {
        fn new(
            initial_decision: InitialActionDecision,
            next_decisions: Vec<RecursivePlannerDecision>,
            recorded_requests: Arc<Mutex<Vec<PlannerRequest>>>,
        ) -> Self {
            Self {
                initial_decision,
                next_decisions: Mutex::new(VecDeque::from(next_decisions)),
                recorded_requests,
            }
        }
    }

    #[async_trait]
    impl RecursivePlanner for TestPlanner {
        fn capability(&self) -> PlannerCapability {
            PlannerCapability::Available
        }

        async fn derive_interpretation_context(
            &self,
            request: &InterpretationRequest,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<InterpretationContext> {
            Ok(InterpretationContext {
                summary: format!(
                    "test interpretation assembled from {} operator-memory document(s).",
                    request.operator_memory.len()
                ),
                documents: request
                    .operator_memory
                    .iter()
                    .map(|document| crate::domain::ports::InterpretationDocument {
                        source: document.source.clone(),
                        excerpt: document.contents.clone(),
                        category: crate::domain::ports::GuidanceCategory::Rule,
                    })
                    .collect(),
                tool_hints: Vec::new(),
                decision_framework: Default::default(),
                ..Default::default()
            })
        }

        async fn select_initial_action(
            &self,
            request: &PlannerRequest,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<InitialActionDecision> {
            self.recorded_requests
                .lock()
                .expect("recorded requests lock")
                .push(request.clone());
            Ok(self.initial_decision.clone())
        }

        async fn select_next_action(
            &self,
            request: &PlannerRequest,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<RecursivePlannerDecision> {
            self.recorded_requests
                .lock()
                .expect("recorded requests lock")
                .push(request.clone());
            self.next_decisions
                .lock()
                .expect("planner decisions lock")
                .pop_front()
                .ok_or_else(|| anyhow!("test planner exhausted"))
        }

        async fn select_thread_decision(
            &self,
            request: &ThreadDecisionRequest,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<ThreadDecision> {
            Ok(ThreadDecision {
                decision_id: ThreadDecisionId::new(format!(
                    "{}.decision",
                    request.candidate.candidate_id.as_str()
                ))?,
                candidate_id: request.candidate.candidate_id.clone(),
                kind: ThreadDecisionKind::ContinueCurrent,
                rationale: "test planner keeps steering on the active thread".to_string(),
                target_thread: request.active_thread.thread_ref.clone(),
                new_thread_label: None,
                merge_mode: None,
                merge_summary: None,
            })
        }

        async fn assess_context_relevance(
            &self,
            _request: &CompactionRequest,
        ) -> Result<CompactionPlan> {
            Ok(CompactionPlan {
                decisions: std::collections::HashMap::new(),
            })
        }
    }
    #[derive(Default)]
    struct RecordingTurnEventSink {
        events: Mutex<Vec<TurnEvent>>,
    }

    impl RecordingTurnEventSink {
        fn recorded(&self) -> Vec<TurnEvent> {
            self.events.lock().expect("event lock").clone()
        }
    }

    impl TurnEventSink for RecordingTurnEventSink {
        fn emit(&self, event: TurnEvent) {
            self.events.lock().expect("event lock").push(event);
        }
    }

    #[derive(Default)]
    struct RecordingTranscriptUpdateSink {
        updates: Mutex<Vec<ConversationTranscriptUpdate>>,
    }

    impl RecordingTranscriptUpdateSink {
        fn recorded(&self) -> Vec<ConversationTranscriptUpdate> {
            self.updates.lock().expect("update lock").clone()
        }
    }

    impl TranscriptUpdateSink for RecordingTranscriptUpdateSink {
        fn emit(&self, update: ConversationTranscriptUpdate) {
            self.updates.lock().expect("update lock").push(update);
        }
    }

    struct RecordingGatherer {
        recorded_requests: Arc<Mutex<Vec<ContextGatherRequest>>>,
        bundle: EvidenceBundle,
    }

    #[async_trait]
    impl ContextGatherer for RecordingGatherer {
        fn capability(&self) -> crate::domain::ports::GathererCapability {
            crate::domain::ports::GathererCapability::Available
        }

        async fn gather_context(
            &self,
            request: &ContextGatherRequest,
        ) -> Result<ContextGatherResult> {
            self.recorded_requests
                .lock()
                .expect("gatherer requests lock")
                .push(request.clone());
            Ok(ContextGatherResult::available(self.bundle.clone()))
        }
    }

    struct StaticConversation {
        responses: VecDeque<String>,
        history: Vec<String>,
    }

    impl StaticConversation {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: VecDeque::from(responses),
                history: Vec::new(),
            }
        }
    }

    impl Conversation for StaticConversation {
        fn send(&mut self, message: &str, _max_tokens: usize) -> Result<String> {
            self.history.push(message.to_string());
            self.responses
                .pop_front()
                .ok_or_else(|| anyhow!("static conversation exhausted"))
        }

        fn history(&self) -> &[String] {
            &self.history
        }
    }

    #[derive(Default)]
    struct RecordingSynthesizer {
        executed_actions: Mutex<Vec<WorkspaceAction>>,
        gathered_summaries: Mutex<Vec<String>>,
    }

    impl SynthesizerEngine for RecordingSynthesizer {
        fn set_verbose(&self, _level: u8) {}

        fn respond_for_turn(
            &self,
            _prompt: &str,
            _turn_intent: TurnIntent,
            gathered_evidence: Option<&EvidenceBundle>,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<String> {
            if let Some(bundle) = gathered_evidence {
                self.gathered_summaries
                    .lock()
                    .expect("gathered summaries lock")
                    .push(bundle.summary.clone());
            }
            Ok("Applied the bounded action.".to_string())
        }

        fn recent_turn_summaries(&self) -> Result<Vec<String>> {
            Ok(Vec::new())
        }

        fn execute_workspace_action(
            &self,
            action: &WorkspaceAction,
        ) -> Result<crate::domain::ports::WorkspaceActionResult> {
            self.executed_actions
                .lock()
                .expect("executed actions lock")
                .push(action.clone());
            Ok(crate::domain::ports::WorkspaceActionResult {
                name: action.label().to_string(),
                summary: format!("executed {}", action.summary()),
            })
        }
    }

    fn initial_action_decision(action: InitialAction, rationale: &str) -> InitialActionDecision {
        InitialActionDecision {
            action,
            rationale: rationale.to_string(),
            edit: InitialEditInstruction::default(),
        }
    }

    #[test]
    fn runtime_lane_config_defaults_to_synthesizer_responses() {
        let config = RuntimeLaneConfig::new("qwen-1.5b", None);

        assert_eq!(config.default_response_role(), RuntimeLaneRole::Synthesizer);
        assert_eq!(config.synthesizer_model_id(), "qwen-1.5b");
        assert_eq!(config.gatherer_model_id(), None);
        assert_eq!(config.gatherer_provider(), GathererProvider::SiftDirect);
        assert!(!config.context1_harness_ready());
    }

    #[tokio::test]
    async fn prepare_runtime_lanes_mix_provider_selection_and_only_resolve_sift_paths() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(RecordingRegistry::default());
        let operator_memory = Arc::new(AgentMemory::load(workspace.path()));
        let captured_planner_lane = Arc::new(Mutex::new(None::<PreparedModelLane>));
        let captured_synthesizer_lane = Arc::new(Mutex::new(None::<PreparedModelLane>));
        let planner_capture = Arc::clone(&captured_planner_lane);
        let synthesizer_capture = Arc::clone(&captured_synthesizer_lane);
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let service = MechSuitService::new(
            workspace.path(),
            registry.clone(),
            operator_memory,
            Box::new(move |_, lane| {
                *synthesizer_capture
                    .lock()
                    .expect("captured synthesizer lane lock") = Some(lane.clone());
                Ok(Arc::new(RecordingSynthesizer::default()) as Arc<dyn SynthesizerEngine>)
            }),
            Box::new(move |_, lane| {
                *planner_capture.lock().expect("captured planner lane lock") = Some(lane.clone());
                Ok(Arc::new(TestPlanner::new(
                    initial_action_decision(InitialAction::Answer, "not used"),
                    Vec::new(),
                    Arc::clone(&recorded_requests),
                )) as Arc<dyn RecursivePlanner>)
            }),
            Box::new(|_, _, _, _| Ok(None)),
        );
        let config = RuntimeLaneConfig::new("gpt-4o".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_planner_provider(Some(ModelProvider::Sift))
            .with_planner_model_id(Some("qwen-1.5b".to_string()));

        let prepared = service
            .prepare_runtime_lanes(&config)
            .await
            .expect("prepare runtime lanes");

        assert_eq!(prepared.synthesizer.provider, ModelProvider::Openai);
        assert_eq!(prepared.synthesizer.paths, None);
        assert_eq!(prepared.planner.provider, ModelProvider::Sift);
        assert_eq!(
            prepared.planner.paths,
            Some(sample_model_paths("qwen-1.5b"))
        );
        assert_eq!(
            registry.requested_model_ids(),
            vec!["qwen-1.5b".to_string()]
        );
        assert_eq!(
            captured_planner_lane
                .lock()
                .expect("captured planner lane lock")
                .clone()
                .expect("planner lane")
                .provider,
            ModelProvider::Sift
        );
        assert_eq!(
            captured_synthesizer_lane
                .lock()
                .expect("captured synthesizer lane lock")
                .clone()
                .expect("synthesizer lane")
                .provider,
            ModelProvider::Openai
        );
    }

    #[test]
    fn prepared_runtime_lanes_keep_synthesizer_as_default_response_lane() {
        let planner = MechSuitService::build_lane(
            RuntimeLaneRole::Planner,
            ModelProvider::Sift,
            "qwen-1.5b",
            Some(sample_model_paths("planner")),
        );
        let synthesizer = MechSuitService::build_lane(
            RuntimeLaneRole::Synthesizer,
            ModelProvider::Sift,
            "qwen-1.5b",
            Some(sample_model_paths("synth")),
        );
        let gatherer = PreparedGathererLane {
            provider: GathererProvider::Local,
            label: "qwen-7b".to_string(),
            model_id: Some("qwen-7b".to_string()),
            paths: Some(sample_model_paths("gather")),
        };
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
        let gatherer = PreparedGathererLane {
            provider: GathererProvider::Context1,
            label: "context-1".to_string(),
            model_id: None,
            paths: None,
        };

        assert_eq!(gatherer.provider, GathererProvider::Context1);
        assert_eq!(gatherer.label, "context-1");
        assert_eq!(gatherer.model_id, None);
        assert_eq!(gatherer.paths, None);
    }

    #[test]
    fn sift_direct_boundary_can_be_prepared_without_local_model_paths() {
        let gatherer = PreparedGathererLane {
            provider: GathererProvider::SiftDirect,
            label: "sift-direct".to_string(),
            model_id: None,
            paths: None,
        };

        assert_eq!(gatherer.provider, GathererProvider::SiftDirect);
        assert_eq!(gatherer.label, "sift-direct");
        assert_eq!(gatherer.model_id, None);
        assert_eq!(gatherer.paths, None);
    }

    #[test]
    fn answer_initial_actions_route_to_direct_responses() {
        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };

        let plan = super::execution_plan_from_initial_action(
            &prepared,
            initial_action_decision(InitialAction::Answer, "no workspace resources needed"),
        );

        assert_eq!(plan.intent, TurnIntent::DirectResponse);
        assert_eq!(plan.path, super::PromptExecutionPath::SynthesizerOnly);
    }

    #[test]
    fn explicit_workspace_initial_actions_route_to_the_planner_loop() {
        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };

        let plan = super::execution_plan_from_initial_action(
            &prepared,
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Shell {
                        command: "git status".to_string(),
                    },
                },
                "explicit workspace action",
            ),
        );

        assert_eq!(plan.intent, TurnIntent::Planned);
        assert_eq!(plan.path, super::PromptExecutionPath::PlannerThenSynthesize);
        assert!(plan.route_summary.contains("git status"));
    }

    #[test]
    fn resource_initial_actions_route_to_the_planner_loop() {
        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: Some(PreparedGathererLane {
                provider: GathererProvider::SiftDirect,
                label: "sift-direct".to_string(),
                model_id: None,
                paths: None,
            }),
        };

        let plan = super::execution_plan_from_initial_action(
            &prepared,
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status".to_string(),
                    },
                },
                "inspect repo state first",
            ),
        );

        assert_eq!(plan.intent, TurnIntent::Planned);
        assert_eq!(plan.path, super::PromptExecutionPath::PlannerThenSynthesize);
        assert!(plan.route_summary.contains("git status"));
        assert!(plan.initial_planner_decision.is_some());
    }

    #[test]
    fn stop_initial_actions_fall_back_to_direct_responses() {
        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };

        let plan = super::execution_plan_from_initial_action(
            &prepared,
            initial_action_decision(
                InitialAction::Stop {
                    reason: "no recursive resource use needed".to_string(),
                },
                "answer directly",
            ),
        );

        assert_eq!(plan.intent, TurnIntent::DirectResponse);
        assert_eq!(plan.path, super::PromptExecutionPath::SynthesizerOnly);
    }

    #[test]
    fn process_prompt_assembles_interpretation_before_model_selected_initial_action() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse AGENTS guidance before choosing the next bounded action.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Answer,
                "the turn can be answered directly after interpretation",
            ),
            Vec::new(),
            Arc::clone(&request_log),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(StaticConversation::new(vec![
                "Hello from the planner path.".to_string(),
            ])),
        ));
        let service = test_service(workspace.path());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let response = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_with_sink("Howdy", sink.clone())
                .await
                .expect("process prompt")
        });

        assert_eq!(response, "Hello from the planner path.");

        let requests = request_log.lock().expect("request log");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0].user_prompt, "Howdy");
        assert_eq!(
            requests[0].interpretation.sources(),
            vec!["AGENTS.md".to_string()]
        );

        let events = sink.recorded();
        let interpretation_index = events
            .iter()
            .position(|event| matches!(event, TurnEvent::InterpretationContext { .. }))
            .expect("interpretation event");
        let action_index = events
            .iter()
            .position(|event| matches!(event, TurnEvent::PlannerActionSelected { .. }))
            .expect("planner action event");
        let classified_index = events
            .iter()
            .position(|event| matches!(event, TurnEvent::IntentClassified { .. }))
            .expect("intent classified event");

        assert!(interpretation_index < action_index);
        assert!(action_index < classified_index);
        assert!(matches!(
            &events[classified_index],
            TurnEvent::IntentClassified {
                intent: TurnIntent::DirectResponse
            }
        ));
    }

    #[test]
    fn process_prompt_records_trace_contract_records_beside_turn_events() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nRecord durable traces for recursive turns.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(InitialAction::Answer, "answer directly"),
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(StaticConversation::new(vec![
                "Recorded response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_with_sink("Record this turn", sink)
                .await
                .expect("process prompt")
        });

        let task_ids = recorder.task_ids();
        assert_eq!(task_ids.len(), 1);
        let replay = recorder.replay(&task_ids[0]).expect("replay");
        assert!(replay.records.len() >= 3);
        assert!(matches!(
            replay.records.first().map(|record| &record.kind),
            Some(TraceRecordKind::TaskRootStarted(_))
        ));
        assert!(
            replay
                .records
                .iter()
                .any(|record| matches!(record.kind, TraceRecordKind::PlannerAction { .. }))
        );
        assert!(
            replay
                .records
                .iter()
                .any(|record| matches!(record.kind, TraceRecordKind::CompletionCheckpoint(_)))
        );
    }

    #[test]
    fn shared_conversation_session_reuses_live_session_state() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());

        let shared = service.shared_conversation_session();
        let attached = service.shared_conversation_session();

        assert_eq!(shared.task_id(), attached.task_id());
        assert_eq!(shared.allocate_turn_id().as_str(), "task-000001.turn-0001");
        assert_eq!(
            attached.allocate_turn_id().as_str(),
            "task-000001.turn-0002"
        );
    }

    #[test]
    fn replay_conversation_transcript_projects_prompt_and_completion_records() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nProject a canonical transcript from durable trace records.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(InitialAction::Answer, "answer directly"),
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(StaticConversation::new(vec![
                "Transcript response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_sink(
                    "Project this conversation",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let transcript = service
            .replay_conversation_transcript(&session.task_id())
            .expect("transcript replay");

        assert_eq!(transcript.task_id, session.task_id());
        assert_eq!(transcript.entries.len(), 2);
        assert_eq!(transcript.entries[0].content, "Project this conversation");
        assert_eq!(transcript.entries[1].content, "Transcript response.");
    }

    #[test]
    fn replay_conversation_transcript_returns_empty_for_known_session_without_trace_records() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());
        let session = service.shared_conversation_session();

        let transcript = service
            .replay_conversation_transcript(&session.task_id())
            .expect("transcript replay");

        assert_eq!(transcript.task_id, session.task_id());
        assert!(transcript.entries.is_empty());
    }

    #[test]
    fn process_prompt_emits_transcript_updates_for_prompt_and_completion() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nEmit transcript updates for durable conversation changes.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(InitialAction::Answer, "answer directly"),
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(StaticConversation::new(vec![
                "Update response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let updates = Arc::new(RecordingTranscriptUpdateSink::default());
        service.register_transcript_observer(updates.clone());
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_sink(
                    "Emit transcript updates",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let recorded = updates.recorded();
        assert_eq!(recorded.len(), 2);
        assert!(
            recorded
                .iter()
                .all(|update| update.task_id == session.task_id())
        );
    }

    #[test]
    fn recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse recursive search when workspace evidence is required.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: Some(PreparedGathererLane {
                provider: GathererProvider::SiftDirect,
                label: "sift-direct".to_string(),
                model_id: None,
                paths: None,
            }),
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Search {
                        query: "what should the recursive gatherer inspect".to_string(),
                        mode: RetrievalMode::Graph,
                        strategy: crate::domain::ports::RetrievalStrategy::Vector,
                        intent: Some("repo-question".to_string()),
                    },
                },
                "start with bounded recursive retrieval",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "enough graph evidence".to_string(),
                },
                rationale: "synthesize after the graph gather".to_string(),
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(StaticConversation::new(vec![
                "Graph-backed answer.".to_string(),
            ])),
        ));
        let gatherer_requests = Arc::new(Mutex::new(Vec::new()));
        let gatherer_bundle = EvidenceBundle::new(
            "Autonomous `heuristic` graph gatherer collected 2 evidence item(s) for `what should the recursive gatherer inspect` across 1 turn(s); stop reason: goal-satisfied. branches: 2, frontier: 1, active branch: branch-root.".to_string(),
            vec![EvidenceItem {
                source: "src/application/mod.rs".to_string(),
                snippet: "Graph-mode gatherers feed the recursive harness.".to_string(),
                rationale: "Relevant recursive routing contract.".to_string(),
                rank: 1,
            }],
        )
        .with_planner(PlannerTraceMetadata {
            mode: RetrievalMode::Graph,
            strategy: PlannerStrategyKind::Heuristic,
            profile: None,
            session_id: Some("graph-session".to_string()),
            completed: true,
            stop_reason: Some("goal-satisfied".to_string()),
            turn_count: 1,
            steps: Vec::new(),
            retained_artifacts: vec![RetainedEvidence {
                source: "src/application/mod.rs".to_string(),
                snippet: Some("Graph-mode gatherers feed the recursive harness.".to_string()),
                rationale: Some("Retain the routing contract.".to_string()),
                locator: None,
            }],
            graph_episode: Some(PlannerGraphEpisode {
                root_node_id: Some("node-root".to_string()),
                active_branch_id: Some("branch-root".to_string()),
                frontier: vec![crate::domain::ports::PlannerGraphFrontierEntry {
                    frontier_id: "frontier-a".to_string(),
                    branch_id: "branch-root".to_string(),
                    node_id: "node-root".to_string(),
                    priority: 1,
                }],
                branches: vec![
                    PlannerGraphBranch {
                        branch_id: "branch-root".to_string(),
                        status: PlannerGraphBranchStatus::Active,
                        head_node_id: "node-root".to_string(),
                        retained_artifacts: vec![RetainedEvidence {
                            source: "src/application/mod.rs".to_string(),
                            snippet: Some(
                                "Graph-mode gatherers feed the recursive harness.".to_string(),
                            ),
                            rationale: Some("active branch evidence".to_string()),
                            locator: None,
                        }],
                    },
                    PlannerGraphBranch {
                        branch_id: "branch-b".to_string(),
                        status: PlannerGraphBranchStatus::Pending,
                        head_node_id: "node-b".to_string(),
                        retained_artifacts: Vec::new(),
                    },
                ],
                nodes: Vec::new(),
                edges: Vec::new(),
                completed: true,
                artifact_ref: None,
            }),
            trace_artifact_ref: None,
        });
        let gatherer = Arc::new(RecordingGatherer {
            recorded_requests: Arc::clone(&gatherer_requests),
            bundle: gatherer_bundle,
        });
        let service = test_service(workspace.path());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let response = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: Some(gatherer),
            });
            service
                .process_prompt_with_sink("What's next in the recursive graph?", sink.clone())
                .await
                .expect("process prompt")
        });

        assert!(!response.trim().is_empty());
        assert!(response.contains("src/application/mod.rs"));
        let recorded_requests = gatherer_requests.lock().expect("gatherer requests");
        assert_eq!(recorded_requests.len(), 1);
        assert_eq!(recorded_requests[0].planning.mode, RetrievalMode::Graph);
        assert_eq!(recorded_requests[0].planning.step_limit, 1);

        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerSummary {
                mode,
                active_branch_id,
                branch_count,
                frontier_count,
                ..
            } if mode == "graph"
                && active_branch_id.as_deref() == Some("branch-root")
                && *branch_count == Some(2)
                && *frontier_count == Some(1)
        )));
    }

    #[test]
    fn execution_pressure_targets_code_files_before_docs() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            evidence_items: vec![
                EvidenceItem {
                    source: "README.md".to_string(),
                    snippet: "overview".to_string(),
                    rationale: "doc".to_string(),
                    rank: 1,
                },
                EvidenceItem {
                    source: "src/application/mod.rs".to_string(),
                    snippet: "planner loop".to_string(),
                    rationale: "code".to_string(),
                    rank: 2,
                },
                EvidenceItem {
                    source: "command: git status".to_string(),
                    snippet: "clean".to_string(),
                    rationale: "not a file".to_string(),
                    rank: 0,
                },
            ],
            ..Default::default()
        };

        let ranked =
            super::likely_execution_pressure_targets(&loop_state, Path::new("/workspace"), 3);

        assert_eq!(
            ranked,
            vec![
                "src/application/mod.rs".to_string(),
                "README.md".to_string()
            ]
        );
    }

    #[test]
    fn execution_pressure_redirects_non_file_actions_before_any_search_step() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            evidence_items: vec![EvidenceItem {
                source: "src/application/mod.rs".to_string(),
                snippet: "planner loop".to_string(),
                rationale: "best candidate".to_string(),
                rank: 1,
            }],
            ..Default::default()
        };
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "cargo test".to_string(),
                },
            },
            rationale: "run a check first".to_string(),
        };

        let redirected = super::coerce_execution_pressure_action(
            "fix the execution pressure behavior",
            &InterpretationContext::default(),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        )
        .expect("redirected decision");

        assert!(matches!(
            redirected.action,
            PlannerAction::Workspace {
                action: WorkspaceAction::Read { path }
            } if path == "src/application/mod.rs"
        ));
    }

    #[test]
    fn execution_pressure_reads_best_candidate_after_initial_search() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nFavor concrete action after bounded search.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: Some(PreparedGathererLane {
                provider: GathererProvider::SiftDirect,
                label: "sift-direct".to_string(),
                model_id: None,
                paths: None,
            }),
        };
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Search {
                        query: "planner loop edit target".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                        intent: Some("implementation search".to_string()),
                    },
                },
                "start with one bounded search",
            ),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Search {
                            query: "keep searching broadly".to_string(),
                            mode: RetrievalMode::Linear,
                            strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                            intent: Some("implementation search".to_string()),
                        },
                    },
                    rationale: "continue exploring".to_string(),
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                },
            ],
            Arc::clone(&request_log),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let gatherer_requests = Arc::new(Mutex::new(Vec::new()));
        let gatherer = Arc::new(RecordingGatherer {
            recorded_requests: Arc::clone(&gatherer_requests),
            bundle: EvidenceBundle::new(
                "Found likely implementation targets.",
                vec![
                    EvidenceItem {
                        source: "src/application/mod.rs".to_string(),
                        snippet: "planner loop handles execution pressure".to_string(),
                        rationale: "best candidate".to_string(),
                        rank: 1,
                    },
                    EvidenceItem {
                        source: "README.md".to_string(),
                        snippet: "project overview".to_string(),
                        rationale: "secondary context".to_string(),
                        rank: 2,
                    },
                ],
            ),
        });
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: Some(gatherer),
            });
            service
                .process_prompt("Fix the execution-pressure behavior")
                .await
                .expect("process prompt")
        });

        let gatherer_requests = gatherer_requests.lock().expect("gatherer requests");
        assert_eq!(gatherer_requests.len(), 1);

        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions");
        assert!(executed_actions.iter().any(|action| matches!(
            action,
            WorkspaceAction::Read { path } if path == "src/application/mod.rs"
        )));
        assert!(!executed_actions.iter().any(|action| matches!(
            action,
            WorkspaceAction::Search { query, .. } if query == "keep searching broadly"
        )));
    }

    #[test]
    fn known_edit_initial_decision_bootstraps_to_a_file_read() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src/application")).expect("create app dir");
        fs::write(
            workspace.path().join("src/application/mod.rs"),
            "fn planner_loop() {}\n",
        )
        .expect("write app file");
        fs::write(workspace.path().join("README.md"), "# Docs\n").expect("write readme");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "known edit turn; controller should choose the file".to_string(),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec![
                        "README.md".to_string(),
                        "src/application/mod.rs".to_string(),
                    ],
                },
            },
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "the file was enough".to_string(),
                },
                rationale: "stop after the read".to_string(),
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("edit the planner loop")
                .await
                .expect("process prompt")
        });

        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions lock");
        assert!(matches!(
            executed_actions.first(),
            Some(WorkspaceAction::Read { path }) if path == "src/application/mod.rs"
        ));
    }

    #[test]
    fn mutation_turn_bootstraps_to_a_file_read_even_without_model_edit_flag() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src/application")).expect("create app dir");
        fs::write(
            workspace.path().join("src/application/mod.rs"),
            "fn planner_loop() {}\n",
        )
        .expect("write app file");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Answer,
                "controller should answer directly if left alone",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "the file was enough".to_string(),
                },
                rationale: "stop after the read".to_string(),
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("fix the planner loop")
                .await
                .expect("process prompt")
        });

        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions lock");
        assert!(matches!(
            executed_actions.first(),
            Some(WorkspaceAction::Read { path }) if path == "src/application/mod.rs"
        ));
    }

    #[test]
    fn known_edit_bootstrap_uses_vector_evidence_to_pick_the_best_candidate() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src")).expect("create src");
        fs::write(workspace.path().join("src/one.rs"), "fn one() {}\n").expect("write one");
        fs::write(workspace.path().join("src/two.rs"), "fn two() {}\n").expect("write two");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Sift,
                model_id: "planner".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "synth".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: Some(PreparedGathererLane {
                provider: GathererProvider::SiftDirect,
                label: "sift-direct".to_string(),
                model_id: None,
                paths: None,
            }),
        };
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Workspace {
                    action: WorkspaceAction::Search {
                        query: "find the edit target".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                        intent: Some("implementation search".to_string()),
                    },
                },
                rationale: "controller should still choose the file".to_string(),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["src/one.rs".to_string(), "src/two.rs".to_string()],
                },
            },
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "done".to_string(),
                },
                rationale: "stop".to_string(),
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let gatherer = Arc::new(RecordingGatherer {
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
            bundle: EvidenceBundle::new(
                "vector bootstrap ranked the file candidates".to_string(),
                vec![EvidenceItem {
                    source: "src/two.rs".to_string(),
                    snippet: "most relevant".to_string(),
                    rationale: "closest semantic match".to_string(),
                    rank: 1,
                }],
            ),
        });
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: Some(gatherer),
            });
            service
                .process_prompt("edit the second implementation")
                .await
                .expect("process prompt")
        });

        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions lock");
        assert!(matches!(
            executed_actions.first(),
            Some(WorkspaceAction::Read { path }) if path == "src/two.rs"
        ));
    }

    #[test]
    fn execution_pressure_falls_back_to_prompt_derived_file_candidates() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src/application")).expect("create app dir");
        fs::write(
            workspace.path().join("src/application/mod.rs"),
            "fn planner_loop() {}\n",
        )
        .expect("write app file");

        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "cargo test".to_string(),
                },
            },
            rationale: "run a check first".to_string(),
        };

        let redirected = super::coerce_execution_pressure_action(
            "fix the planner loop",
            &InterpretationContext::default(),
            &PlannerLoopState::default(),
            &decision,
            workspace.path(),
        )
        .expect("redirected decision");

        assert!(matches!(
            redirected.action,
            PlannerAction::Workspace {
                action: WorkspaceAction::Read { path }
            } if path == "src/application/mod.rs"
        ));
    }

    fn sample_model_paths(prefix: &str) -> ModelPaths {
        ModelPaths {
            weights: PathBuf::from(format!("{prefix}-weights.safetensors")),
            tokenizer: PathBuf::from(format!("{prefix}-tokenizer.json")),
            config: PathBuf::from(format!("{prefix}-config.json")),
        }
    }

    #[tokio::test]
    async fn compaction_engine_executes_plan_and_preserves_locators() {
        use super::*;
        use crate::infrastructure::adapters::transit_resolver::NoopContextResolver;
        use paddles_conversation::{ContextLocator, TaskTraceId, TraceRecordId};

        let engine = CompactionEngine::new(Arc::new(NoopContextResolver));
        let task_id = TaskTraceId::new("task-1").unwrap();
        let record_id = TraceRecordId::new("record-1").unwrap();

        let artifacts = vec![RetainedEvidence {
            source: "src/lib.rs".to_string(),
            snippet: Some("long content".to_string()),
            rationale: Some("test".to_string()),
            locator: Some(ContextLocator::Transit {
                task_id: task_id.clone(),
                record_id: record_id.clone(),
            }),
        }];

        let mut decisions = std::collections::HashMap::new();
        decisions.insert(
            TraceArtifactId::new("record-1").unwrap(),
            CompactionDecision::Compact {
                summary: "short summary".to_string(),
            },
        );

        let plan = CompactionPlan { decisions };
        let compacted = engine.execute(artifacts, plan).await;

        assert_eq!(compacted.len(), 1);
        assert_eq!(compacted[0].snippet.as_deref(), Some("short summary"));
        assert!(matches!(
            compacted[0].locator,
            Some(ContextLocator::Transit { .. })
        ));
    }

    #[test]
    fn validate_inspect_rejects_chained_commands() {
        assert!(super::validate_inspect_command("ls && rm -rf /").is_err());
        assert!(super::validate_inspect_command("cat file; echo done").is_err());
        assert!(super::validate_inspect_command("echo > /tmp/out").is_err());
        assert!(super::validate_inspect_command("cat < /etc/passwd").is_err());
        assert!(super::validate_inspect_command("true || false").is_err());
    }

    #[test]
    fn validate_inspect_allows_safe_read_only_commands() {
        assert!(super::validate_inspect_command("git status").is_ok());
        assert!(super::validate_inspect_command("git remote get-url origin").is_ok());
        assert!(super::validate_inspect_command("ls -la").is_ok());
        assert!(super::validate_inspect_command("cat README.md").is_ok());
    }

    #[test]
    fn validate_inspect_rejects_empty_commands() {
        assert!(super::validate_inspect_command("").is_err());
        assert!(super::validate_inspect_command("   ").is_err());
    }
}
