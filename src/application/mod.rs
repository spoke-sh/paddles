use crate::infrastructure::adapters::TransitContextResolver;
use crate::infrastructure::adapters::trace_recorders::TransitTraceRecorder;
use crate::infrastructure::adapters::transit_resolver::NoopContextResolver;
use crate::infrastructure::adapters::workspace_entity_resolver::WorkspaceEntityResolver;
use crate::infrastructure::conversation_history::ConversationHistoryStore;
use crate::infrastructure::native_transport::NativeTransportRegistry;
use crate::infrastructure::providers::ModelProvider;
use crate::infrastructure::terminal::run_background_terminal_command;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
pub use paddles_conversation::{ContextLocator, ConversationSession, TraceArtifactId};

use crate::domain::model::{
    ArtifactEnvelope, ArtifactKind, AuthoredResponse, BootContext, CompactionDecision,
    CompactionPlan, ConversationForensicProjection, ConversationForensicUpdate,
    ConversationManifoldProjection, ConversationProjectionSnapshot, ConversationProjectionUpdate,
    ConversationProjectionUpdateKind, ConversationThreadRef, ConversationTraceGraph,
    ConversationTranscript, ConversationTranscriptUpdate, ForensicArtifactCapture,
    ForensicTraceSink, ForensicUpdateSink, InstructionFrame, InstructionIntent, MultiplexEventSink,
    NativeTransportDiagnostic, PlanChecklistItem, PlanChecklistItemStatus, ResponseMode,
    SteeringGateKind, SteeringGatePhase, StrainFactor, StrainLevel, TaskTraceId, ThreadCandidate,
    ThreadDecision, ThreadDecisionKind, ThreadMergeMode, ThreadMergeRecord, TraceBranch,
    TraceBranchId, TraceBranchStatus, TraceCheckpointId, TraceCheckpointKind,
    TraceCompletionCheckpoint, TraceLineage, TraceLineageEdge, TraceLineageNodeKind,
    TraceLineageNodeRef, TraceLineageRelation, TraceModelExchangeArtifact, TraceModelExchangePhase,
    TraceRecord, TraceRecordId, TraceRecordKind, TraceSelectionArtifact, TraceSelectionKind,
    TraceSignalContribution, TraceSignalKind, TraceSignalSnapshot, TraceTaskRoot, TraceToolCall,
    TraceTurnStarted, TranscriptUpdateSink, TurnEvent, TurnEventSink, TurnIntent, TurnTraceId,
};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, ContextResolver, EntityLookupMode,
    EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest, EntityResolver,
    EvidenceBudget, EvidenceBundle, EvidenceItem, GathererCapability, GroundingRequirement,
    InitialAction, InitialActionDecision, InitialEditInstruction, InterpretationContext,
    InterpretationProcedure, InterpretationProcedureStep, InterpretationRequest,
    InterpretationToolHint, ModelPaths, ModelRegistry, NoopTraceRecorder, NormalizedEntityHint,
    OperatorMemory, PlannerAction, PlannerBudget, PlannerCapability, PlannerConfig,
    PlannerLoopState, PlannerRequest, PlannerStepRecord, PlannerStrategyKind, PlannerTraceMetadata,
    PlannerTraceStep, RecursivePlanner, RecursivePlannerDecision, RetainedEvidence, RetrievalMode,
    RetrievalStrategy, RetrieverOption, SynthesisHandoff, SynthesizerEngine, ThreadDecisionRequest,
    TraceRecorder, WorkspaceAction,
};
use anyhow::Result;
use clap::ValueEnum;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
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
    entity_resolver: Arc<dyn EntityResolver>,
    runtime: RwLock<Option<ActiveRuntimeState>>,
    verbose: AtomicU8,
    event_sink: Arc<dyn TurnEventSink>,
    event_observers: Mutex<Vec<Arc<dyn TurnEventSink>>>,
    transcript_observers: Mutex<Vec<Arc<dyn TranscriptUpdateSink>>>,
    forensic_observers: Mutex<Vec<Arc<dyn ForensicUpdateSink>>>,
    trace_recorder: Arc<dyn TraceRecorder>,
    trace_counter: AtomicU64,
    sessions: Mutex<HashMap<String, ConversationSession>>,
    shared_session_id: Mutex<Option<String>>,
    conversation_history_store: Mutex<Option<Arc<ConversationHistoryStore>>>,
    native_transport_registry: Mutex<Arc<NativeTransportRegistry>>,
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
    entity_resolver: Arc<dyn EntityResolver>,
    interpretation: InterpretationContext,
    recent_turns: Vec<String>,
    recent_thread_summary: Option<String>,
    instruction_frame: Option<InstructionFrame>,
    initial_edit: InitialEditInstruction,
    grounding: Option<GroundingRequirement>,
}

struct PlannerGatherSpec {
    query: String,
    intent_reason: String,
    mode: RetrievalMode,
    strategy: RetrievalStrategy,
    retrievers: Vec<RetrieverOption>,
    max_evidence_items: usize,
    success_summary_override: Option<String>,
    no_bundle_message: &'static str,
    failure_label: &'static str,
    unavailable_label: &'static str,
    missing_backend_message: &'static str,
}

#[derive(Default)]
struct ConsoleTurnEventSink {
    render_lock: Mutex<()>,
}

impl TurnEventSink for ConsoleTurnEventSink {
    fn emit(&self, event: TurnEvent) {
        let _guard = self.render_lock.lock().expect("turn event render lock");
        let rendered = render_turn_event(&event);
        if !rendered.is_empty() {
            println!("{}", rendered);
        }
    }
}

#[derive(Clone)]
struct StructuredTurnTrace {
    downstream: Arc<dyn TurnEventSink>,
    recorder: Arc<dyn TraceRecorder>,
    forensic_observers: Vec<Arc<dyn ForensicUpdateSink>>,
    session: ConversationSession,
    turn_id: TurnTraceId,
    active_thread: ConversationThreadRef,
    last_synthesis: Arc<Mutex<Option<SynthesisTraceState>>>,
    last_turn_response_exchange_id: Arc<Mutex<Option<String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SynthesisTraceState {
    grounded: bool,
    citations: Vec<String>,
    insufficient_evidence: bool,
}

struct SignalSnapshotRecord {
    kind: TraceSignalKind,
    gate: Option<SteeringGateKind>,
    phase: Option<SteeringGatePhase>,
    summary: String,
    level: String,
    magnitude_percent: u8,
    applies_to: Option<TraceLineageNodeRef>,
    contributions: Vec<TraceSignalContribution>,
    details: serde_json::Value,
}

impl StructuredTurnTrace {
    fn new(
        downstream: Arc<dyn TurnEventSink>,
        recorder: Arc<dyn TraceRecorder>,
        forensic_observers: Vec<Arc<dyn ForensicUpdateSink>>,
        session: ConversationSession,
        turn_id: TurnTraceId,
        active_thread: ConversationThreadRef,
    ) -> Self {
        Self {
            downstream,
            recorder,
            forensic_observers,
            session,
            turn_id,
            active_thread,
            last_synthesis: Arc::new(Mutex::new(None)),
            last_turn_response_exchange_id: Arc::new(Mutex::new(None)),
        }
    }

    fn as_event_sink(self: &Arc<Self>) -> Arc<dyn TurnEventSink> {
        let sink: Arc<dyn TurnEventSink> = self.clone();
        sink
    }

    fn default_branch_id(&self) -> Option<TraceBranchId> {
        self.active_thread.branch_id()
    }

    fn conversation_node(&self) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::Conversation,
            id: format!("conversation:{}", self.session.task_id().as_str()),
            label: "conversation".to_string(),
        }
    }

    fn turn_node(&self) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::Turn,
            id: format!("turn:{}", self.turn_id.as_str()),
            label: self.turn_id.as_str().to_string(),
        }
    }

    fn planner_step_node(&self, record_id: &TraceRecordId) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::PlannerStep,
            id: format!("planner-step:{}", record_id.as_str()),
            label: record_id.as_str().to_string(),
        }
    }

    fn model_call_node(&self, exchange_id: &str) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::ModelCall,
            id: format!("model-call:{exchange_id}"),
            label: exchange_id.to_string(),
        }
    }

    fn artifact_node(&self, artifact_id: &TraceArtifactId) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::Artifact,
            id: format!("artifact:{}", artifact_id.as_str()),
            label: artifact_id.as_str().to_string(),
        }
    }

    fn output_node(&self, artifact_id: &TraceArtifactId) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::Output,
            id: format!("output:{}", artifact_id.as_str()),
            label: artifact_id.as_str().to_string(),
        }
    }

    fn signal_node(&self, kind: TraceSignalKind, record_id: &TraceRecordId) -> TraceLineageNodeRef {
        TraceLineageNodeRef {
            kind: TraceLineageNodeKind::Signal,
            id: format!("signal:{}", record_id.as_str()),
            label: kind.label().to_string(),
        }
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
            self.record_lineage_edge(
                None,
                self.conversation_node(),
                self.turn_node(),
                TraceLineageRelation::Contains,
                "conversation contains initial turn",
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
        self.record_lineage_edge(
            self.default_branch_id(),
            self.conversation_node(),
            self.turn_node(),
            TraceLineageRelation::Contains,
            "conversation contains turn",
        );
    }

    fn record_planner_action(
        &self,
        action: &str,
        rationale: &str,
        branch_id: Option<TraceBranchId>,
    ) {
        let branch_id = branch_id.or_else(|| self.default_branch_id());
        let record_id = self.record_kind(
            branch_id.clone(),
            TraceRecordKind::PlannerAction {
                action: action.to_string(),
                rationale: rationale.to_string(),
            },
        );
        self.record_lineage_edge(
            branch_id,
            self.turn_node(),
            self.planner_step_node(&record_id),
            TraceLineageRelation::Contains,
            format!("turn contains planner step `{action}`"),
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

    fn completion_response_mode_for_synthesis(
        &self,
        instruction_frame: Option<&InstructionFrame>,
    ) -> ResponseMode {
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

        if let Some(frame) = instruction_frame {
            if frame.has_pending_workspace_obligation() {
                return ResponseMode::BlockedEdit;
            }
            if frame.primary_intent == InstructionIntent::Edit {
                return ResponseMode::CompletedEdit;
            }
        }

        if synthesis.grounded && !synthesis.insufficient_evidence {
            ResponseMode::GroundedAnswer
        } else {
            ResponseMode::DirectAnswer
        }
    }

    fn record_completion(&self, response: &AuthoredResponse) {
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
                response.to_plain_text(),
                1_200,
            )
            .with_label("citations", synthesis.citations.join(", "))
            .with_label("paddles.response_mode", response.mode.label());
        let response_artifact_id = response.artifact_id.clone();
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
        self.record_lineage_edge(
            self.default_branch_id(),
            self.turn_node(),
            self.output_node(&response_artifact_id),
            TraceLineageRelation::ResultsIn,
            "turn produced final output",
        );
        if let Some(exchange_id) = self
            .last_turn_response_exchange_id
            .lock()
            .expect("turn response exchange lock")
            .clone()
        {
            self.record_lineage_edge(
                self.default_branch_id(),
                self.model_call_node(&exchange_id),
                self.output_node(&response_artifact_id),
                TraceLineageRelation::ResultsIn,
                "model call resulted in final output",
            );
        }
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

    fn current_parent_record_id(&self, branch_id: Option<&TraceBranchId>) -> Option<TraceRecordId> {
        let session_state = self.session.state();
        let state = session_state.lock().expect("conversation session lock");
        self.parent_record_id_for(
            state.root_last_record_id.clone(),
            &state.branch_last_record_ids,
            branch_id,
        )
    }

    fn record_kind(
        &self,
        branch_id: Option<TraceBranchId>,
        kind: TraceRecordKind,
    ) -> TraceRecordId {
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
        let record_id = record.record_id.clone();
        self.record_or_warn(record);
        record_id
    }

    fn record_lineage_edge(
        &self,
        branch_id: Option<TraceBranchId>,
        source: TraceLineageNodeRef,
        target: TraceLineageNodeRef,
        relation: TraceLineageRelation,
        summary: impl Into<String>,
    ) {
        self.record_kind(
            branch_id,
            TraceRecordKind::LineageEdge(TraceLineageEdge {
                source,
                target,
                relation,
                summary: summary.into(),
                labels: Default::default(),
            }),
        );
    }

    fn record_signal_snapshot(&self, record: SignalSnapshotRecord) {
        let summary = record.summary;
        let artifact = self.exact_artifact(
            ArtifactKind::PlannerTrace,
            summary.clone(),
            record.details.to_string(),
            "application/json",
        );
        let signal_record_id = self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                kind: record.kind,
                gate: record.gate.or_else(|| Some(record.kind.steering_gate())),
                phase: record.phase.or_else(|| Some(record.kind.steering_phase())),
                summary: summary.clone(),
                level: record.level,
                magnitude_percent: record.magnitude_percent,
                applies_to: record.applies_to.clone(),
                contributions: record.contributions,
                artifact,
            }),
        );
        let signal_node = self.signal_node(record.kind, &signal_record_id);
        self.record_lineage_edge(
            self.default_branch_id(),
            self.turn_node(),
            signal_node.clone(),
            TraceLineageRelation::Contains,
            format!("turn carries {} steering signal", record.kind.label()),
        );
        if let Some(target) = record.applies_to {
            self.record_lineage_edge(
                self.default_branch_id(),
                signal_node,
                target.clone(),
                TraceLineageRelation::Constrains,
                format!(
                    "{} steering signal constrains {}",
                    record.kind.label(),
                    target.kind.label()
                ),
            );
        }
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
        let record_id = record.record_id.clone();
        let task_id = record.lineage.task_id.clone();
        let turn_id = record.lineage.turn_id.clone();
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
        } else {
            let update = ConversationForensicUpdate {
                task_id,
                turn_id,
                record_id,
            };
            for observer in &self.forensic_observers {
                observer.emit(update.clone());
            }
        }
    }

    fn record_entity_resolution_outcome(
        &self,
        outcome: &EntityResolutionOutcome,
        source: &'static str,
    ) {
        let (kind, summary, level, magnitude_percent, contributions, details) =
            entity_resolution_signal_record(outcome, source);
        self.record_signal_snapshot(SignalSnapshotRecord {
            kind,
            gate: None,
            phase: None,
            summary,
            level: level.to_string(),
            magnitude_percent,
            applies_to: Some(self.turn_node()),
            contributions,
            details,
        });
    }
}

impl TurnEventSink for StructuredTurnTrace {
    fn emit(&self, event: TurnEvent) {
        self.downstream.emit(event.clone());
        if let Some(snapshot) = crate::domain::model::derive_harness_snapshot(&event) {
            self.downstream.emit(TurnEvent::HarnessState { snapshot });
        }
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
            TurnEvent::ContextStrain { strain } => {
                let contributions = strain_signal_contributions(&strain);
                self.record_signal_snapshot(SignalSnapshotRecord {
                    kind: TraceSignalKind::ContextStrain,
                    gate: None,
                    phase: None,
                    summary: format!("context strain reached {}", strain.level.label()),
                    level: strain.level.label().to_string(),
                    magnitude_percent: strain_level_magnitude(strain.level),
                    applies_to: Some(self.turn_node()),
                    contributions,
                    details: serde_json::json!({
                        "level": strain.level.label(),
                        "truncation_count": strain.truncation_count,
                        "factors": strain.factors.iter().map(|factor| factor.label()).collect::<Vec<_>>(),
                    }),
                });
            }
            TurnEvent::Fallback { stage, reason } => {
                let signal_kind = if stage == "action-bias" {
                    TraceSignalKind::ActionBias
                } else {
                    TraceSignalKind::Fallback
                };
                let (level, magnitude_percent, contributions) =
                    fallback_signal_details(stage.as_str(), reason.as_str());
                let (summary, details) = if stage == "entity-resolution" {
                    (
                        entity_resolution_fallback_summary(reason.as_str()),
                        entity_resolution_fallback_details(reason.as_str()),
                    )
                } else {
                    (
                        format!("{stage} fallback"),
                        serde_json::json!({
                            "stage": stage,
                            "reason": reason,
                        }),
                    )
                };
                let (gate, phase) = if stage == "premise-challenge" {
                    (
                        Some(SteeringGateKind::Convergence),
                        Some(SteeringGatePhase::Narrowing),
                    )
                } else {
                    (None, None)
                };
                self.record_signal_snapshot(SignalSnapshotRecord {
                    kind: signal_kind,
                    gate,
                    phase,
                    summary,
                    level: level.to_string(),
                    magnitude_percent,
                    applies_to: Some(self.turn_node()),
                    contributions,
                    details,
                });
            }
            TurnEvent::RefinementApplied {
                reason,
                before_summary,
                after_summary,
            } => {
                self.record_signal_snapshot(SignalSnapshotRecord {
                    kind: TraceSignalKind::CompactionCue,
                    gate: None,
                    phase: None,
                    summary: "context refinement applied".to_string(),
                    level: "medium".to_string(),
                    magnitude_percent: 58,
                    applies_to: Some(self.turn_node()),
                    contributions: compaction_signal_contributions(),
                    details: serde_json::json!({
                        "reason": reason,
                        "before_summary": before_summary,
                        "after_summary": after_summary,
                    }),
                });
            }
            TurnEvent::PlannerSummary { stop_reason, .. } => {
                if let Some(stop_reason) = stop_reason.filter(|reason| {
                    reason.contains("budget")
                        || reason.contains("boundary")
                        || reason.contains("challenge")
                }) {
                    let (level, magnitude_percent, contributions) =
                        budget_signal_details(stop_reason.as_str());
                    self.record_signal_snapshot(SignalSnapshotRecord {
                        kind: TraceSignalKind::BudgetBoundary,
                        gate: None,
                        phase: None,
                        summary: format!("planner stop reason `{stop_reason}`"),
                        level: level.to_string(),
                        magnitude_percent,
                        applies_to: Some(self.turn_node()),
                        contributions,
                        details: serde_json::json!({
                            "stop_reason": stop_reason,
                        }),
                    });
                }
            }
            _ => {}
        }
    }

    fn forensic_trace_sink(&self) -> Option<&dyn ForensicTraceSink> {
        Some(self)
    }
}

impl ForensicTraceSink for StructuredTurnTrace {
    fn allocate_model_exchange_id(
        &self,
        _lane: crate::domain::model::TraceModelExchangeLane,
        _category: crate::domain::model::TraceModelExchangeCategory,
    ) -> String {
        self.session.next_exchange_id(&self.turn_id)
    }

    fn record_forensic_artifact(
        &self,
        capture: ForensicArtifactCapture,
    ) -> Option<TraceArtifactId> {
        let branch_id = self.default_branch_id();
        let current_parent_record_id = self.current_parent_record_id(branch_id.as_ref());
        let parent_artifact_id = capture.parent_artifact_id.clone();
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
            branch_id.clone(),
            TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                exchange_id: capture.exchange_id.clone(),
                lane: capture.lane,
                category: capture.category,
                phase: capture.phase,
                provider: capture.provider,
                model: capture.model,
                parent_artifact_id: parent_artifact_id.clone(),
                artifact,
            }),
        );
        if parent_artifact_id.is_none() {
            self.record_lineage_edge(
                branch_id.clone(),
                self.turn_node(),
                self.model_call_node(&capture.exchange_id),
                TraceLineageRelation::Contains,
                format!("turn contains model call {}", capture.exchange_id),
            );
            if matches!(
                capture.lane,
                crate::domain::model::TraceModelExchangeLane::Planner
            ) && let Some(parent_record_id) = current_parent_record_id
            {
                self.record_lineage_edge(
                    branch_id.clone(),
                    self.planner_step_node(&parent_record_id),
                    self.model_call_node(&capture.exchange_id),
                    TraceLineageRelation::Triggers,
                    "planner step triggered model call",
                );
            }
        }
        self.record_lineage_edge(
            branch_id.clone(),
            self.model_call_node(&capture.exchange_id),
            self.artifact_node(&artifact_id),
            TraceLineageRelation::Produces,
            format!("model call produced {}", capture.phase.label()),
        );
        if let Some(parent_artifact_id) = &parent_artifact_id {
            self.record_lineage_edge(
                branch_id.clone(),
                self.artifact_node(parent_artifact_id),
                self.artifact_node(&artifact_id),
                TraceLineageRelation::Transforms,
                format!(
                    "{} transformed into {}",
                    parent_artifact_id.as_str(),
                    artifact_id.as_str()
                ),
            );
        }
        if capture.category == crate::domain::model::TraceModelExchangeCategory::TurnResponse
            && capture.phase == TraceModelExchangePhase::RenderedResponse
        {
            let mut state = self
                .last_turn_response_exchange_id
                .lock()
                .expect("turn response exchange lock");
            *state = Some(capture.exchange_id);
        }
        Some(artifact_id)
    }
}

fn strain_level_magnitude(level: StrainLevel) -> u8 {
    match level {
        StrainLevel::Low => 10,
        StrainLevel::Medium => 45,
        StrainLevel::High => 72,
        StrainLevel::Critical => 92,
    }
}

fn strain_signal_contributions(
    strain: &crate::domain::model::ContextStrain,
) -> Vec<TraceSignalContribution> {
    if strain.factors.is_empty() {
        return vec![TraceSignalContribution {
            source: "context".to_string(),
            share_percent: 100,
            rationale: "No specific factor was isolated, so the strain is attributed to the overall assembled context.".to_string(),
        }];
    }

    let share = (100 / strain.factors.len()).max(1) as u8;
    let mut remaining = 100u8;
    strain
        .factors
        .iter()
        .enumerate()
        .map(|(index, factor)| {
            let assigned = if index + 1 == strain.factors.len() {
                remaining
            } else {
                let value = share.min(remaining);
                remaining = remaining.saturating_sub(value);
                value
            };
            let (source, rationale) = match factor {
                StrainFactor::MemoryTruncated => (
                    "operator_memory",
                    "Operator memory truncation raised context strain.",
                ),
                StrainFactor::ArtifactTruncated => (
                    "retained_artifacts",
                    "Retained artifacts were truncated to fit the active context budget.",
                ),
                StrainFactor::ThreadSummaryTrimmed => (
                    "thread_summaries",
                    "Thread summaries were trimmed, reducing recalled state.",
                ),
                StrainFactor::EvidenceBudgetExhausted => (
                    "evidence_budget",
                    "Evidence budget exhaustion constrained how much supporting context could be retained.",
                ),
            };
            TraceSignalContribution {
                source: source.to_string(),
                share_percent: assigned,
                rationale: rationale.to_string(),
            }
        })
        .collect()
}

fn compaction_signal_contributions() -> Vec<TraceSignalContribution> {
    vec![
        TraceSignalContribution {
            source: "controller_policy".to_string(),
            share_percent: 60,
            rationale: "The controller compacted or refined context to preserve actionability under budget.".to_string(),
        },
        TraceSignalContribution {
            source: "retained_artifacts".to_string(),
            share_percent: 40,
            rationale: "Existing retained artifacts shaped what was summarized or dropped.".to_string(),
        },
    ]
}

fn fallback_signal_details(
    stage: &str,
    reason: &str,
) -> (&'static str, u8, Vec<TraceSignalContribution>) {
    if stage == "action-bias" {
        return (
            "high",
            84,
            vec![
                TraceSignalContribution {
                    source: "controller_policy".to_string(),
                    share_percent: 45,
                    rationale: "Action bias nudged the planner toward likely target files quickly.".to_string(),
                },
                TraceSignalContribution {
                    source: "prompt_edit_signal".to_string(),
                    share_percent: 30,
                    rationale: "The turn was interpreted as an edit-oriented request requiring file action.".to_string(),
                },
                TraceSignalContribution {
                    source: "candidate_file_evidence".to_string(),
                    share_percent: 25,
                    rationale: format!("The controller had plausible file evidence: {reason}"),
                },
            ],
        );
    }

    if stage == "entity-resolution" {
        return (
            "high",
            82,
            vec![
                TraceSignalContribution {
                    source: "workspace_editor_boundary".to_string(),
                    share_percent: 55,
                    rationale:
                        "Deterministic entity resolution blocked workspace mutation until the target state became safe."
                            .to_string(),
                },
                TraceSignalContribution {
                    source: "candidate_file_evidence".to_string(),
                    share_percent: 45,
                    rationale: format!(
                        "The resolver outcome was surfaced explicitly: {reason}"
                    ),
                },
            ],
        );
    }

    if stage == "premise-challenge" {
        return (
            "medium",
            56,
            vec![
                TraceSignalContribution {
                    source: "premise_challenge".to_string(),
                    share_percent: 60,
                    rationale: reason.to_string(),
                },
                TraceSignalContribution {
                    source: "controller_policy".to_string(),
                    share_percent: 40,
                    rationale:
                        "The controller forced a premise review before allowing another evidence probe."
                            .to_string(),
                },
            ],
        );
    }

    (
        "medium",
        56,
        vec![
            TraceSignalContribution {
                source: "provider_or_parser".to_string(),
                share_percent: 60,
                rationale: format!("The fallback was triggered at `{stage}` because `{reason}`."),
            },
            TraceSignalContribution {
                source: "controller_safety".to_string(),
                share_percent: 40,
                rationale: "The controller substituted a safer path to keep the turn recoverable."
                    .to_string(),
            },
        ],
    )
}

fn format_steering_review_fallback_reason(
    decision: &RecursivePlannerDecision,
    reviewed: &RecursivePlannerDecision,
) -> String {
    let original_summary = decision.action.summary();
    let reviewed_summary = reviewed.action.summary();
    if original_summary == reviewed_summary {
        format!(
            "Reviewed `{original_summary}` and kept the same action after judging the current sources."
        )
    } else {
        format!(
            "Replaced `{original_summary}` with `{reviewed_summary}` after judging the current sources."
        )
    }
}

fn entity_resolution_status_from_reason(reason: &str) -> &'static str {
    let normalized = reason.to_ascii_lowercase();
    if normalized.contains("ambiguous") {
        "ambiguous"
    } else {
        "missing"
    }
}

fn entity_resolution_fallback_summary(reason: &str) -> String {
    match entity_resolution_status_from_reason(reason) {
        "ambiguous" => "deterministic resolver ambiguous".to_string(),
        _ => "deterministic resolver missing".to_string(),
    }
}

fn entity_resolution_fallback_details(reason: &str) -> serde_json::Value {
    serde_json::json!({
        "stage": "entity-resolution",
        "status": entity_resolution_status_from_reason(reason),
        "reason": reason,
    })
}

fn entity_resolution_signal_record(
    outcome: &EntityResolutionOutcome,
    source: &'static str,
) -> (
    TraceSignalKind,
    String,
    &'static str,
    u8,
    Vec<TraceSignalContribution>,
    serde_json::Value,
) {
    match outcome {
        EntityResolutionOutcome::Resolved {
            target,
            alternatives,
            explanation,
        } => (
            TraceSignalKind::ActionBias,
            format!("deterministic resolver resolved {}", target.path),
            "high",
            79,
            vec![
                TraceSignalContribution {
                    source: "candidate_file_evidence".to_string(),
                    share_percent: 60,
                    rationale: format!(
                        "Deterministic ranking converged on `{}` before further workspace churn.",
                        target.path
                    ),
                },
                TraceSignalContribution {
                    source: "controller_policy".to_string(),
                    share_percent: 40,
                    rationale:
                        "Known-edit steering elevated the resolved authored target into the edit path."
                            .to_string(),
                },
            ],
            serde_json::json!({
                "stage": "entity-resolution",
                "status": "resolved",
                "source": source,
                "path": target.path,
                "candidates": std::iter::once(target.path.clone())
                    .chain(alternatives.iter().map(|candidate| candidate.path.clone()))
                    .collect::<Vec<_>>(),
                "explanation": explanation,
            }),
        ),
        EntityResolutionOutcome::Ambiguous {
            candidates,
            explanation,
        } => (
            TraceSignalKind::Fallback,
            "deterministic resolver ambiguous".to_string(),
            "high",
            82,
            vec![
                TraceSignalContribution {
                    source: "workspace_editor_boundary".to_string(),
                    share_percent: 55,
                    rationale:
                        "The controller held the edit boundary because multiple authored targets remained viable."
                            .to_string(),
                },
                TraceSignalContribution {
                    source: "candidate_file_evidence".to_string(),
                    share_percent: 45,
                    rationale:
                        "The turn surfaced the tied authored candidates instead of guessing."
                            .to_string(),
                },
            ],
            serde_json::json!({
                "stage": "entity-resolution",
                "status": "ambiguous",
                "source": source,
                "candidates": candidates.iter().map(|candidate| candidate.path.clone()).collect::<Vec<_>>(),
                "explanation": explanation,
            }),
        ),
        EntityResolutionOutcome::Missing {
            attempted_hints,
            explanation,
        } => (
            TraceSignalKind::Fallback,
            "deterministic resolver missing".to_string(),
            "high",
            78,
            vec![
                TraceSignalContribution {
                    source: "workspace_editor_boundary".to_string(),
                    share_percent: 50,
                    rationale:
                        "The controller refused to mutate the workspace without a safe authored target."
                            .to_string(),
                },
                TraceSignalContribution {
                    source: "controller_policy".to_string(),
                    share_percent: 50,
                    rationale:
                        "Missing deterministic resolution stayed explicit so the turn could replan instead of hallucinating."
                            .to_string(),
                },
            ],
            serde_json::json!({
                "stage": "entity-resolution",
                "status": "missing",
                "source": source,
                "attempted_hint_count": attempted_hints.len(),
                "explanation": explanation,
            }),
        ),
    }
}

fn budget_signal_details(stop_reason: &str) -> (&'static str, u8, Vec<TraceSignalContribution>) {
    if stop_reason.contains("workspace-editor-boundary")
        || stop_reason.contains("workspace-commit-boundary")
    {
        let (boundary_source, boundary_summary) = if stop_reason
            .contains("workspace-commit-boundary")
        {
            (
                "workspace_commit_boundary",
                "The turn crossed the boundary where planning should yield to recording the requested git commit.",
            )
        } else {
            (
                "workspace_editor_boundary",
                "The turn crossed the boundary where planning should yield to the workspace editor for an applied edit.",
            )
        };
        let secondary_source = if stop_reason.contains("search-budget") {
            "search_budget"
        } else if stop_reason.contains("inspect-budget") {
            "inspect_budget"
        } else if stop_reason.contains("read-budget") {
            "read_budget"
        } else if stop_reason.contains("planner-budget") {
            "planner_budget"
        } else {
            "controller_policy"
        };
        return (
            "high",
            86,
            vec![
                TraceSignalContribution {
                    source: boundary_source.to_string(),
                    share_percent: 60,
                    rationale: boundary_summary.to_string(),
                },
                TraceSignalContribution {
                    source: secondary_source.to_string(),
                    share_percent: 40,
                    rationale: format!(
                        "The planner stop reason still carried `{stop_reason}` while the applied-edit obligation remained open."
                    ),
                },
            ],
        );
    }

    let source = if stop_reason.contains("search-budget") {
        "search_budget"
    } else if stop_reason.contains("inspect-budget") {
        "inspect_budget"
    } else if stop_reason.contains("read-budget") {
        "read_budget"
    } else if stop_reason.contains("premise-challenge") {
        "premise_challenge"
    } else {
        "planner_budget"
    };

    (
        "high",
        if stop_reason.contains("challenge") || stop_reason.contains("bias") {
            68
        } else {
            78
        },
        vec![
            TraceSignalContribution {
                source: source.to_string(),
                share_percent: 65,
                rationale: format!("The planner stopped because `{stop_reason}`."),
            },
            TraceSignalContribution {
                source: "planner_budget".to_string(),
                share_percent: 35,
                rationale: "The planner budget bounded additional recursion and retrieval."
                    .to_string(),
            },
        ],
    )
}

fn render_turn_event(event: &TurnEvent) -> String {
    match event {
        TurnEvent::IntentClassified { intent } => {
            format!("• Classified turn\n  └ {}", intent.label())
        }
        TurnEvent::HarnessState { snapshot } => {
            if !event.should_emit_to_projection_stream() {
                return String::new();
            }

            format!(
                "{}\n  └ {}",
                snapshot.governor_header(),
                trim_event_detail(&snapshot.governor_summary(false), 3)
            )
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
        TurnEvent::PlanUpdated { items } => format!(
            "• Updated Plan\n  └ {}",
            trim_event_detail(
                &items
                    .iter()
                    .map(|item| format!("{} {}", item.status.marker(), item.label))
                    .collect::<Vec<_>>()
                    .join("\n"),
                8
            )
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
        TurnEvent::ContextStrain { strain } => {
            let factors: Vec<_> = strain.factors.iter().map(|f| f.label()).collect();
            format!(
                "• Context strain: {}\n  └ {} truncation(s), factors: [{}]",
                strain.level.label(),
                strain.truncation_count,
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
        TurnEvent::ToolOutput {
            tool_name,
            stream,
            output,
            ..
        } => format!(
            "• {tool_name} {stream}\n  └ {}",
            trim_event_detail(output, 6)
        ),
        TurnEvent::ToolFinished {
            tool_name, summary, ..
        } => format!(
            "• Completed {tool_name}\n  └ {}",
            trim_event_detail(summary, 6)
        ),
        TurnEvent::WorkspaceEditApplied {
            tool_name, edit, ..
        } => format!(
            "• Applied {tool_name}\n  └ files: {}\n  └ change: +{} -{}\n  └ {}",
            if edit.files.is_empty() {
                "(unknown file)".to_string()
            } else {
                trim_event_detail(&edit.files.join(", "), 1)
            },
            edit.insertions,
            edit.deletions,
            trim_event_detail(&edit.diff, 4)
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
            format!("• Hunting ({phase})\n  └ elapsed {elapsed_seconds}s{eta}{strategy}{suffix}")
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
            entity_resolver: Arc::new(WorkspaceEntityResolver::new()),
            runtime: RwLock::new(None),
            verbose: AtomicU8::new(0),
            event_sink: Arc::new(ConsoleTurnEventSink::default()),
            event_observers: Mutex::new(Vec::new()),
            transcript_observers: Mutex::new(Vec::new()),
            forensic_observers: Mutex::new(Vec::new()),
            trace_recorder,
            trace_counter: AtomicU64::new(1),
            sessions: Mutex::new(HashMap::new()),
            shared_session_id: Mutex::new(None),
            conversation_history_store: Mutex::new(None),
            native_transport_registry: Mutex::new(Arc::new(NativeTransportRegistry::default())),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    pub fn verbose(&self) -> u8 {
        self.verbose.load(Ordering::Relaxed)
    }

    pub fn set_conversation_history_store(&self, store: Arc<ConversationHistoryStore>) {
        *self
            .conversation_history_store
            .lock()
            .expect("conversation history store lock") = Some(store);
    }

    pub fn set_native_transport_registry(&self, registry: Arc<NativeTransportRegistry>) {
        *self
            .native_transport_registry
            .lock()
            .expect("native transport registry lock") = registry;
    }

    pub fn native_transport_diagnostics(&self) -> Vec<NativeTransportDiagnostic> {
        self.native_transport_registry
            .lock()
            .expect("native transport registry lock")
            .diagnostics()
    }

    pub fn prompt_history(&self) -> Result<Vec<String>> {
        match self.conversation_history_store() {
            Some(store) => store.prompt_history(),
            None => Ok(Vec::new()),
        }
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

    pub fn register_forensic_observer(&self, observer: Arc<dyn ForensicUpdateSink>) {
        self.forensic_observers
            .lock()
            .expect("forensic observers lock")
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

    fn replay_for_known_session(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<Option<crate::domain::model::TraceReplay>> {
        match self.trace_recorder.replay(task_id) {
            Ok(replay) => Ok(Some(replay)),
            Err(err) => {
                let known_session = self
                    .sessions
                    .lock()
                    .expect("conversation sessions lock")
                    .contains_key(task_id.as_str());
                if known_session { Ok(None) } else { Err(err) }
            }
        }
    }

    pub fn replay_conversation_forensics(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationForensicProjection> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationForensicProjection::from_trace_replay(&replay)),
            None => Ok(ConversationForensicProjection {
                task_id: task_id.clone(),
                turns: Vec::new(),
            }),
        }
    }

    pub fn replay_turn_forensics(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<crate::domain::model::ForensicTurnProjection>> {
        Ok(self.replay_conversation_forensics(task_id)?.turn(turn_id))
    }

    pub fn replay_conversation_manifold(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationManifoldProjection> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationManifoldProjection::from_trace_replay(&replay)),
            None => Ok(ConversationManifoldProjection {
                task_id: task_id.clone(),
                turns: Vec::new(),
            }),
        }
    }

    pub fn replay_turn_manifold(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<crate::domain::model::ManifoldTurnProjection>> {
        Ok(self.replay_conversation_manifold(task_id)?.turn(turn_id))
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

    fn cloned_forensic_observers(&self) -> Vec<Arc<dyn ForensicUpdateSink>> {
        self.forensic_observers
            .lock()
            .expect("forensic observers lock")
            .clone()
    }

    fn conversation_history_store(&self) -> Option<Arc<ConversationHistoryStore>> {
        self.conversation_history_store
            .lock()
            .expect("conversation history store lock")
            .clone()
    }

    fn recent_turn_summaries(
        &self,
        synthesizer_engine: &dyn SynthesizerEngine,
    ) -> Result<Vec<String>> {
        if let Some(store) = self.conversation_history_store() {
            let recent_turns = store.recent_turn_summaries()?;
            if !recent_turns.is_empty() {
                return Ok(recent_turns);
            }
        }

        synthesizer_engine.recent_turn_summaries()
    }

    fn persist_prompt_history(&self, prompt: &str) {
        let Some(store) = self.conversation_history_store() else {
            return;
        };

        if let Err(err) = store.record_prompt(prompt) {
            self.warn_history_store_error("persist prompt history", &err);
        }
    }

    fn persist_recent_turn_summary(&self, prompt: &str, reply: &str) {
        let Some(store) = self.conversation_history_store() else {
            return;
        };

        if let Err(err) = store.record_turn(prompt, reply) {
            self.warn_history_store_error("persist recent turn history", &err);
        }
    }

    fn finalize_turn_response(
        &self,
        trace: &StructuredTurnTrace,
        session: &ConversationSession,
        active_thread: &ConversationThreadRef,
        prompt: &str,
        response: &AuthoredResponse,
    ) -> String {
        let reply = response.to_plain_text();
        trace.record_completion(response);
        self.emit_transcript_update(&session.task_id());
        session.note_thread_reply(active_thread, prompt, &reply);
        self.persist_recent_turn_summary(prompt, &reply);
        reply
    }

    async fn execute_planner_gather_step(
        &self,
        context: &PlannerLoopContext,
        loop_state: &mut PlannerLoopState,
        trace: Arc<StructuredTurnTrace>,
        gatherer_provider: &str,
        spec: PlannerGatherSpec,
        used_workspace_resources: &mut bool,
    ) -> String {
        let PlannerGatherSpec {
            query,
            intent_reason,
            mode,
            strategy,
            retrievers,
            max_evidence_items,
            success_summary_override,
            no_bundle_message,
            failure_label,
            unavailable_label,
            missing_backend_message,
        } = spec;
        let Some(gatherer) = context.gatherer.as_ref() else {
            return missing_backend_message.to_string();
        };
        let planning = PlannerConfig::default()
            .with_mode(mode)
            .with_retrieval_strategy(strategy)
            .with_retrievers(retrievers)
            .with_step_limit(1);
        let capability = gatherer.capability_for_planning(&planning);

        trace.emit(TurnEvent::GathererCapability {
            provider: gatherer_provider.to_string(),
            capability: format_gatherer_capability(&capability),
        });

        match capability {
            GathererCapability::Available => {
                let request = ContextGatherRequest::new(
                    query.clone(),
                    self.workspace_root.clone(),
                    intent_reason,
                    EvidenceBudget::default(),
                )
                .with_planning(planning)
                .with_prior_context(
                    build_planner_prior_context(
                        &context.interpretation,
                        &context.recent_turns,
                        loop_state,
                        Some(context.resolver.clone()),
                    )
                    .await,
                );
                gatherer.set_event_sink(Some(trace.clone() as Arc<dyn TurnEventSink>));
                match gatherer.gather_context(&request).await {
                    Ok(result) => {
                        let bundle = result.evidence_bundle;
                        if let Some(bundle) = bundle.as_ref() {
                            trace.emit(TurnEvent::GathererSummary {
                                provider: gatherer_provider.to_string(),
                                summary: bundle.summary.clone(),
                                sources: evidence_sources(&self.workspace_root, bundle),
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
                                loop_state.latest_gatherer_trace = Some(planner.clone());
                            }
                            merge_evidence_items(
                                &mut loop_state.evidence_items,
                                bundle.items.clone(),
                                max_evidence_items,
                            );
                            loop_state.notes.extend(bundle.warnings.clone());
                            *used_workspace_resources = true;
                            success_summary_override.unwrap_or_else(|| bundle.summary.clone())
                        } else {
                            no_bundle_message.to_string()
                        }
                    }
                    Err(err) => format!("{failure_label}: {err:#}"),
                }
            }
            GathererCapability::Warming { reason }
            | GathererCapability::Unsupported { reason }
            | GathererCapability::HarnessRequired { reason } => {
                format!("{unavailable_label}: {reason}")
            }
        }
    }

    fn warn_history_store_error(&self, action: &str, err: &anyhow::Error) {
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            eprintln!("[WARN] Could not {action}: {err:#}");
        }
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
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationTranscript::from_trace_replay(&replay)),
            None => Ok(ConversationTranscript {
                task_id: task_id.clone(),
                entries: Vec::new(),
            }),
        }
    }

    pub fn replay_conversation_trace_graph(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationTraceGraph> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationTraceGraph::from_trace_replay(&replay)),
            None => Ok(ConversationTraceGraph::empty(task_id.clone())),
        }
    }

    pub fn replay_conversation_projection(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationProjectionSnapshot> {
        Ok(ConversationProjectionSnapshot {
            task_id: task_id.clone(),
            transcript: self.replay_conversation_transcript(task_id)?,
            forensics: self.replay_conversation_forensics(task_id)?,
            manifold: self.replay_conversation_manifold(task_id)?,
            trace_graph: self.replay_conversation_trace_graph(task_id)?,
        })
    }

    pub fn projection_update_for_transcript(
        &self,
        update: &ConversationTranscriptUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        Ok(ConversationProjectionUpdate {
            task_id: update.task_id.clone(),
            kind: ConversationProjectionUpdateKind::Transcript,
            transcript_update: Some(update.clone()),
            forensic_update: None,
            snapshot: self.replay_conversation_projection(&update.task_id)?,
        })
    }

    pub fn projection_update_for_forensic(
        &self,
        update: &ConversationForensicUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        Ok(ConversationProjectionUpdate {
            task_id: update.task_id.clone(),
            kind: ConversationProjectionUpdateKind::Forensic,
            transcript_update: None,
            forensic_update: Some(update.clone()),
            snapshot: self.replay_conversation_projection(&update.task_id)?,
        })
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
        self.persist_prompt_history(prompt);
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
            self.cloned_forensic_observers(),
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

        let recent_turns = self.recent_turn_summaries(synthesizer_engine.as_ref())?;
        let recent_thread_summary = session.recent_thread_summary(&active_thread);
        let request = PlannerRequest::new(
            prompt,
            self.workspace_root.clone(),
            interpretation.clone(),
            PlannerBudget::default(),
        )
        .with_recent_turns(recent_turns.clone())
        .with_recent_thread_summary(recent_thread_summary.clone())
        .with_runtime_notes(planner_runtime_notes_for_gatherer(gatherer.as_ref()))
        .with_entity_resolver(Arc::clone(&self.entity_resolver));

        let execution_plan = match planner_capability {
            PlannerCapability::Available => {
                let mut decision = planner_engine
                    .select_initial_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                    .await?;
                let controller_edit =
                    controller_prompt_edit_instruction(&self.workspace_root, prompt);
                let provider_edit_missing = !decision.edit.known_edit && controller_edit.known_edit;
                decision.edit = merge_initial_edit_instruction(&decision.edit, &controller_edit);
                if provider_edit_missing {
                    let candidate_summary = if decision.edit.candidate_files.is_empty() {
                        "no candidate files surfaced yet".to_string()
                    } else {
                        format!(
                            "candidate files: {}",
                            decision.edit.candidate_files.join(", ")
                        )
                    };
                    trace.emit(TurnEvent::Fallback {
                        stage: "action-bias".to_string(),
                        reason: format!(
                            "controller inferred a concrete repository edit from the prompt and activated workspace editor pressure; {candidate_summary}"
                        ),
                    });
                }
                let controller_commit_instruction = controller_prompt_commit_instruction(prompt);
                if let Some(bootstrapped) = bootstrap_git_commit_initial_action(prompt, &decision) {
                    trace.emit(TurnEvent::Fallback {
                        stage: "commit-bootstrap".to_string(),
                        reason: format!(
                            "commit-oriented turn bypassed initial `{}` and forced `{}` to inspect workspace status before committing",
                            decision.action.summary(),
                            bootstrapped.action.summary()
                        ),
                    });
                    decision = bootstrapped;
                }
                if let Some(bootstrapped) = self
                    .bootstrap_known_edit_initial_action(
                        prompt,
                        &interpretation,
                        &recent_turns,
                        gatherer.as_ref(),
                        &decision,
                        trace.as_ref(),
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
                if let Some(bootstrapped) =
                    bootstrap_repository_grounding_initial_action(prompt, &decision)
                {
                    trace.emit(TurnEvent::Fallback {
                        stage: "grounding-bootstrap".to_string(),
                        reason: format!(
                            "repo-scoped conversational turn bypassed initial `{}` and forced `{}` to ground the reply locally",
                            decision.action.summary(),
                            bootstrapped.action.summary()
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
                let mut execution_plan = execution_plan_from_initial_action(&prepared, decision);
                execution_plan.instruction_frame = merge_instruction_frames(
                    execution_plan.instruction_frame.clone(),
                    controller_commit_instruction,
                );
                execution_plan
            }
            PlannerCapability::Unsupported { reason } => {
                trace.emit(TurnEvent::Fallback {
                    stage: "planner".to_string(),
                    reason: format!("planner unavailable before first action selection: {reason}"),
                });
                fallback_execution_plan(&prepared)
            }
        };

        let mut execution_checklist =
            build_execution_checklist(prompt, &recent_turns, &execution_plan);

        trace.emit(TurnEvent::IntentClassified {
            intent: execution_plan.intent.clone(),
        });
        trace.emit(TurnEvent::RouteSelected {
            summary: execution_plan.route_summary.clone(),
        });
        if let Some(checklist) = execution_checklist.as_mut() {
            checklist.emit(trace.as_ref());
        }

        let planner_outcome = match execution_plan.path {
            PromptExecutionPath::PlannerThenSynthesize => {
                let recent_turns = self.recent_turn_summaries(synthesizer_engine.as_ref())?;

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
                        entity_resolver: Arc::clone(&self.entity_resolver),
                        interpretation: interpretation.clone(),
                        recent_turns,
                        recent_thread_summary: recent_thread_summary.clone(),
                        instruction_frame: execution_plan.instruction_frame.clone(),
                        initial_edit: execution_plan.initial_edit.clone(),
                        grounding: execution_plan.grounding.clone(),
                    },
                    execution_plan.initial_planner_decision.clone(),
                    execution_checklist,
                    Arc::clone(&trace),
                )
                .await?
            }
            PromptExecutionPath::SynthesizerOnly => PlannerLoopOutcome {
                evidence: None,
                direct_answer: execution_plan.direct_answer.clone(),
                instruction_frame: execution_plan.instruction_frame.clone(),
                grounding: execution_plan.grounding.clone(),
            },
        };

        if let Some(reply) = planner_outcome.direct_answer {
            let response = if let Some(frame) = planner_outcome
                .instruction_frame
                .as_ref()
                .filter(|frame| frame.has_pending_workspace_obligation())
            {
                blocked_instruction_response(frame)
            } else {
                reply
            };
            trace.emit(TurnEvent::SynthesisReady {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            });
            let reply =
                self.finalize_turn_response(&trace, &session, &active_thread, prompt, &response);
            return Ok(reply);
        }

        let prompt = prompt.to_string();
        let intent = execution_plan.intent;
        let engine = synthesizer_engine;
        let event_sink = trace.as_event_sink();
        let session_for_reply = session.clone();
        let thread_for_reply = active_thread;
        let prompt_for_model = prompt.clone();
        let handoff = SynthesisHandoff {
            recent_turns,
            recent_thread_summary,
            instruction_frame: planner_outcome.instruction_frame.clone(),
            grounding: planner_outcome.grounding.clone(),
        };
        if let Some(frame) = planner_outcome
            .instruction_frame
            .as_ref()
            .filter(|frame| frame.has_pending_workspace_obligation())
        {
            let response = blocked_instruction_response(frame);
            trace.emit(TurnEvent::SynthesisReady {
                grounded: false,
                citations: Vec::new(),
                insufficient_evidence: false,
            });
            let reply = self.finalize_turn_response(
                &trace,
                &session_for_reply,
                &thread_for_reply,
                &prompt,
                &response,
            );
            return Ok(reply);
        }
        let reply = tokio::task::spawn_blocking(move || {
            engine.respond_for_turn(
                &prompt_for_model,
                intent,
                planner_outcome.evidence.as_ref(),
                &handoff,
                event_sink,
            )
        })
        .await
        .map_err(|err| anyhow::anyhow!("Sift session task failed: {err}"))??;
        let response = AuthoredResponse::from_plain_text(
            trace
                .completion_response_mode_for_synthesis(planner_outcome.instruction_frame.as_ref()),
            &reply,
        );
        let reply = self.finalize_turn_response(
            &trace,
            &session_for_reply,
            &thread_for_reply,
            &prompt,
            &response,
        );
        Ok(reply)
    }

    async fn bootstrap_known_edit_initial_action(
        &self,
        prompt: &str,
        interpretation: &InterpretationContext,
        recent_turns: &[String],
        gatherer: Option<&Arc<dyn ContextGatherer>>,
        decision: &InitialActionDecision,
        trace: &StructuredTurnTrace,
    ) -> Result<Option<InitialActionDecision>> {
        if !decision.edit.known_edit {
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

        let resolution = resolve_known_edit_target(
            &self.entity_resolver,
            &self.workspace_root,
            prompt,
            &candidates,
            &[],
            decision.edit.resolution.as_ref(),
        )
        .await;
        if let Some(outcome @ EntityResolutionOutcome::Resolved { .. }) = resolution.as_ref() {
            trace.record_entity_resolution_outcome(outcome, "bootstrap");
        }
        let seeded_candidates = resolution
            .as_ref()
            .map(|outcome| merge_resolution_candidate_paths(&candidates, outcome))
            .unwrap_or_else(|| candidates.clone());

        let ranked_candidates = if matches!(
            resolution.as_ref(),
            Some(EntityResolutionOutcome::Resolved { .. })
        ) {
            seeded_candidates.clone()
        } else if let Some(gatherer) = gatherer {
            rerank_known_edit_candidates_with_vector_lookup(
                gatherer,
                &self.workspace_root,
                prompt,
                interpretation,
                recent_turns,
                &seeded_candidates,
            )
            .await
        } else {
            seeded_candidates.clone()
        };

        let best_path = resolution
            .as_ref()
            .and_then(EntityResolutionOutcome::resolved_path)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| {
                ranked_candidates
                    .first()
                    .cloned()
                    .unwrap_or_else(|| candidates[0].clone())
            });
        Ok(Some(InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Read {
                    path: best_path.clone(),
                },
            },
            rationale: format!(
                "known edit turn; action produces information, so read `{best_path}` before broader planning"
            ),
            answer: None,
            edit: crate::domain::ports::InitialEditInstruction {
                known_edit: decision.edit.known_edit,
                candidate_files: ranked_candidates,
                resolution: resolution.or_else(|| decision.edit.resolution.clone()),
            },
            grounding: decision.grounding.clone(),
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
            self.cloned_forensic_observers(),
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

        let recent_turns = self.recent_turn_summaries(synthesizer_engine.as_ref())?;
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
        let interpretation = match planner
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
        };

        enrich_interpretation_with_local_harness_profile(
            interpretation,
            local_harness_capabilities(),
        )
    }

    async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        initial_decision: Option<RecursivePlannerDecision>,
        execution_checklist: Option<ExecutionChecklistState>,
        trace: Arc<StructuredTurnTrace>,
    ) -> Result<PlannerLoopOutcome> {
        let mut context = context;
        let mut execution_checklist = execution_checklist;
        let base_budget =
            planner_budget_for_turn(context.instruction_frame.as_ref(), &context.initial_edit);
        let mut budget = planner_budget_for_replan_attempt(&base_budget, 0);
        let mut loop_state = PlannerLoopState {
            target_resolution: context.initial_edit.resolution.clone(),
            ..PlannerLoopState::default()
        };
        if let Some(checklist) = execution_checklist.as_ref() {
            checklist.sync_loop_state_notes(&mut loop_state);
        }
        let mut used_workspace_resources = false;
        let mut stop_reason = None;
        let mut direct_answer = None;
        let mut instruction_frame = context.instruction_frame.clone();
        let mut pending_initial_decision = initial_decision;
        let mut steps_without_new_evidence = 0usize;
        let mut replan_count = 0usize;
        let mut sequence = 1usize;
        let gatherer_provider = context
            .prepared
            .gatherer
            .as_ref()
            .map(|lane| lane.label.clone())
            .unwrap_or_else(|| "workspace".to_string());

        loop {
            if sequence > budget.max_steps {
                if activate_replan(
                    "planner-budget-exhausted",
                    ReplanActivation {
                        instruction_frame: instruction_frame.as_ref(),
                        base_budget: &base_budget,
                        completed_replans: &mut replan_count,
                        budget: &mut budget,
                        loop_state: &mut loop_state,
                        execution_checklist: execution_checklist.as_mut(),
                        trace: trace.as_ref(),
                    },
                ) {
                    continue;
                }
                break;
            }

            let evidence_count_before = loop_state.evidence_items.len();
            let planner_selected_this_step = pending_initial_decision.is_none();
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
                .with_recent_thread_summary(context.recent_thread_summary.clone())
                .with_runtime_notes(planner_runtime_notes_for_gatherer(
                    context.gatherer.as_ref(),
                ))
                .with_loop_state(loop_state.clone())
                .with_resolver(context.resolver.clone())
                .with_entity_resolver(context.entity_resolver.clone());
                context
                    .planner_engine
                    .select_next_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
                    .await?
            };

            if planner_selected_this_step {
                decision = review_decision_under_signals(
                    prompt,
                    &context,
                    &budget,
                    &loop_state,
                    decision,
                    &self.workspace_root,
                    trace.clone(),
                )
                .await?;
                trace.emit(TurnEvent::PlannerActionSelected {
                    sequence,
                    action: decision.action.summary(),
                    rationale: decision.rationale.clone(),
                });
                trace.record_planner_action(&decision.action.summary(), &decision.rationale, None);
            }

            instruction_frame =
                merge_instruction_frame_with_edit_signal(instruction_frame, &decision.edit);
            if let Some(resolution) = decision.edit.resolution.clone() {
                loop_state.target_resolution = Some(resolution);
            }

            trace.emit(TurnEvent::PlannerStepProgress {
                step_number: sequence,
                step_limit: budget.max_steps,
                action: decision.action.summary(),
                query: decision.action.target_query(),
                evidence_count: loop_state.evidence_items.len(),
            });

            let mut accepted_stop = false;
            let mut completed_exact_edit = false;
            let outcome = match &decision.action {
                PlannerAction::Workspace { action } => match action {
                    WorkspaceAction::Search {
                        query,
                        mode,
                        strategy,
                        retrievers,
                        intent,
                    } => {
                        if search_steps(&loop_state) >= budget.max_searches {
                            stop_reason = Some("search-budget-exhausted".to_string());
                            "planner search budget exhausted".to_string()
                        } else {
                            self.execute_planner_gather_step(
                                &context,
                                &mut loop_state,
                                trace.clone(),
                                &gatherer_provider,
                                PlannerGatherSpec {
                                    query: query.clone(),
                                    intent_reason: intent
                                        .clone()
                                        .unwrap_or_else(|| "planner-search".to_string()),
                                    mode: *mode,
                                    strategy: *strategy,
                                    retrievers: retrievers.clone(),
                                    max_evidence_items: budget.max_evidence_items,
                                    success_summary_override: None,
                                    no_bundle_message: "planner search returned no evidence bundle",
                                    failure_label: "planner search failed",
                                    unavailable_label: "planner search backend unavailable",
                                    missing_backend_message:
                                        "no gatherer backend is configured for planner search",
                                },
                                &mut used_workspace_resources,
                            )
                            .await
                        }
                    }
                    WorkspaceAction::Inspect { command } => {
                        if inspect_steps(&loop_state) >= budget.max_inspects {
                            stop_reason = Some("inspect-budget-exhausted".to_string());
                            "planner inspect budget exhausted".to_string()
                        } else {
                            let call_id = format!("planner-tool-{sequence}");
                            trace.emit(TurnEvent::ToolCalled {
                                call_id: call_id.clone(),
                                tool_name: "inspect".to_string(),
                                invocation: command.clone(),
                            });
                            match run_planner_inspect_command(
                                &self.workspace_root,
                                command,
                                &call_id,
                                trace.as_ref(),
                            ) {
                                Ok(output) => {
                                    let summary =
                                        planner_terminal_tool_success_summary("inspect", &output);
                                    trace.emit(TurnEvent::ToolFinished {
                                        call_id,
                                        tool_name: "inspect".to_string(),
                                        summary,
                                    });
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
                                    trace.emit(TurnEvent::ToolFinished {
                                        call_id,
                                        tool_name: "inspect".to_string(),
                                        summary: format!("inspect failed: {err:#}"),
                                    });
                                    format!("inspect failed: {err:#}")
                                }
                            }
                        }
                    }
                    WorkspaceAction::Shell { command } => {
                        let call_id = format!("planner-tool-{sequence}");
                        trace.emit(TurnEvent::ToolCalled {
                            call_id: call_id.clone(),
                            tool_name: "shell".to_string(),
                            invocation: command.clone(),
                        });
                        match run_planner_shell_command(
                            &self.workspace_root,
                            command,
                            &call_id,
                            trace.as_ref(),
                        ) {
                            Ok(result) => {
                                let summary =
                                    planner_terminal_tool_success_summary("shell", &result);
                                trace.emit(TurnEvent::ToolFinished {
                                    call_id,
                                    tool_name: "shell".to_string(),
                                    summary,
                                });
                                append_evidence_item(
                                    &mut loop_state.evidence_items,
                                    EvidenceItem {
                                        source: format!("command: {command}"),
                                        snippet: trim_for_planner(&result, 1_200),
                                        rationale: decision.rationale.clone(),
                                        rank: 0,
                                    },
                                    budget.max_evidence_items,
                                );
                                if let Some(frame) = instruction_frame.as_mut() {
                                    frame.note_successful_workspace_action(action);
                                }
                                used_workspace_resources = true;
                                result
                            }
                            Err(err) => {
                                let summary = format!("Tool `shell` failed: {err:#}");
                                trace.emit(TurnEvent::ToolFinished {
                                    call_id,
                                    tool_name: "shell".to_string(),
                                    summary: summary.clone(),
                                });
                                append_evidence_item(
                                    &mut loop_state.evidence_items,
                                    EvidenceItem {
                                        source: format!("command: {command}"),
                                        snippet: trim_for_planner(&summary, 1_200),
                                        rationale: decision.rationale.clone(),
                                        rank: 0,
                                    },
                                    budget.max_evidence_items,
                                );
                                used_workspace_resources = true;
                                stop_reason
                                    .get_or_insert_with(|| "workspace-action-failed".to_string());
                                summary
                            }
                        }
                    }
                    WorkspaceAction::Read { .. }
                    | WorkspaceAction::ListFiles { .. }
                    | WorkspaceAction::Diff { .. }
                    | WorkspaceAction::WriteFile { .. }
                    | WorkspaceAction::ReplaceInFile { .. }
                    | WorkspaceAction::ApplyPatch { .. } => {
                        let previous_resolution = loop_state.target_resolution.clone();
                        maybe_promote_missing_resolution_for_mutation(
                            &self.workspace_root,
                            &context.initial_edit.candidate_files,
                            &mut loop_state,
                            action,
                        );
                        if previous_resolution != loop_state.target_resolution
                            && let Some(outcome @ EntityResolutionOutcome::Resolved { .. }) =
                                loop_state.target_resolution.as_ref()
                        {
                            trace.record_entity_resolution_outcome(outcome, "exact-mutation-path");
                        }
                        if let Some((reason, summary)) =
                            unresolved_target_mutation_boundary(action, &loop_state)
                        {
                            trace.emit(TurnEvent::Fallback {
                                stage: "entity-resolution".to_string(),
                                reason: summary.clone(),
                            });
                            stop_reason = Some(reason);
                            accepted_stop = true;
                            summary
                        } else if matches!(action, WorkspaceAction::Read { .. })
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
                                    completed_exact_edit = decision_is_exact_edit(&decision.action);
                                    if let Some(frame) = instruction_frame.as_mut() {
                                        frame.note_successful_workspace_action(action);
                                    }
                                    if let Some(edit) = result.applied_edit.clone() {
                                        trace.emit(TurnEvent::WorkspaceEditApplied {
                                            call_id,
                                            tool_name: result.name.to_string(),
                                            edit,
                                        });
                                    } else {
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: result.name.to_string(),
                                            summary: result.summary.clone(),
                                        });
                                    }
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
                    retrievers,
                    ..
                } => {
                    if search_steps(&loop_state) >= budget.max_searches {
                        stop_reason = Some("search-budget-exhausted".to_string());
                        "planner refine budget exhausted".to_string()
                    } else {
                        self.execute_planner_gather_step(
                            &context,
                            &mut loop_state,
                            trace.clone(),
                            &gatherer_provider,
                            PlannerGatherSpec {
                                query: query.clone(),
                                intent_reason: "planner-refine".to_string(),
                                mode: *mode,
                                strategy: *strategy,
                                retrievers: retrievers.clone(),
                                max_evidence_items: budget.max_evidence_items,
                                success_summary_override: Some(format!(
                                    "refined search toward `{query}`"
                                )),
                                no_bundle_message: "planner refine returned no evidence bundle",
                                failure_label: "planner refine failed",
                                unavailable_label: "planner refine backend unavailable",
                                missing_backend_message:
                                    "no gatherer backend is configured for refined planner search",
                            },
                            &mut used_workspace_resources,
                        )
                        .await
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
                    if let Some(frame) = instruction_frame
                        .as_ref()
                        .filter(|frame| frame.has_pending_workspace_obligation())
                    {
                        let note = instruction_unsatisfied_note(frame);
                        if !loop_state.notes.contains(&note) {
                            loop_state.notes.push(note.clone());
                        }
                        trace.emit(TurnEvent::Fallback {
                            stage: "instruction-manifold".to_string(),
                            reason: note.clone(),
                        });
                        direct_answer = None;
                        stop_reason = Some("instruction-unsatisfied".to_string());
                        "planner stop converted into a blocked reply because the requested applied edit is still unsatisfied"
                            .to_string()
                    } else {
                        direct_answer = stop_reason_direct_answer(reason, decision.answer.clone());
                        stop_reason = Some(reason.clone());
                        accepted_stop = true;
                        format!("planner requested synthesis: {reason}")
                    }
                }
            };

            loop_state.steps.push(PlannerStepRecord {
                step_id: format!("planner-step-{sequence}"),
                sequence,
                branch_id: None,
                action: decision.action.clone(),
                outcome: outcome.clone(),
            });

            if let Some(checklist) = execution_checklist.as_mut() {
                let mut changed = false;
                if sequence == 1 {
                    changed |= checklist.mark_completed("initial-action");
                }
                if completed_exact_edit {
                    changed |= checklist.mark_completed("apply-edit");
                }
                if decision_is_git_commit(&decision.action) {
                    changed |= checklist.mark_completed("record-commit");
                }
                if accepted_stop
                    && !instruction_frame
                        .as_ref()
                        .is_some_and(|frame| frame.has_pending_workspace_obligation())
                {
                    changed |= checklist.mark_completed("finalize");
                }
                checklist.sync_loop_state_notes(&mut loop_state);
                if changed {
                    checklist.emit(trace.as_ref());
                }
            }

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
                        sequence += 1;
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

            if accepted_stop {
                break;
            }

            if let Some(reason) = stop_reason.clone() {
                if activate_replan(
                    &reason,
                    ReplanActivation {
                        instruction_frame: instruction_frame.as_ref(),
                        base_budget: &base_budget,
                        completed_replans: &mut replan_count,
                        budget: &mut budget,
                        loop_state: &mut loop_state,
                        execution_checklist: execution_checklist.as_mut(),
                        trace: trace.as_ref(),
                    },
                ) {
                    stop_reason = None;
                    sequence += 1;
                    continue;
                }
                break;
            }

            sequence += 1;
        }

        let completed = stop_reason.is_some();
        let stop_reason = annotate_stop_reason_for_pending_instruction(
            stop_reason.unwrap_or_else(|| "planner-budget-exhausted".to_string()),
            instruction_frame.as_ref(),
        );
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
            return Ok(PlannerLoopOutcome {
                evidence: None,
                direct_answer: direct_answer.or_else(|| {
                    instruction_frame
                        .as_ref()
                        .filter(|frame| frame.has_pending_workspace_obligation())
                        .map(blocked_instruction_response)
                }),
                instruction_frame,
                grounding: context.grounding.clone(),
            });
        }

        Ok(PlannerLoopOutcome {
            evidence: Some(build_planner_evidence_bundle(
                &context.prepared,
                prompt,
                &loop_state,
                completed,
                &stop_reason,
            )),
            direct_answer: direct_answer.or_else(|| {
                instruction_frame
                    .as_ref()
                    .filter(|frame| frame.has_pending_workspace_obligation())
                    .map(blocked_instruction_response)
            }),
            instruction_frame,
            grounding: context.grounding.clone(),
        })
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
                "premise-challenge ({} evidence items after {} quiet steps)",
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
    direct_answer: Option<AuthoredResponse>,
    instruction_frame: Option<InstructionFrame>,
    initial_edit: InitialEditInstruction,
    grounding: Option<GroundingRequirement>,
}

struct PlannerLoopOutcome {
    evidence: Option<EvidenceBundle>,
    direct_answer: Option<AuthoredResponse>,
    instruction_frame: Option<InstructionFrame>,
    grounding: Option<GroundingRequirement>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExecutionChecklistPressure {
    continuation: bool,
    multi_phase: bool,
    edit_goal: bool,
    commit_goal: bool,
    grounding_goal: bool,
}

impl ExecutionChecklistPressure {
    fn is_active(self) -> bool {
        self.continuation
            || self.multi_phase
            || self.edit_goal
            || self.commit_goal
            || self.grounding_goal
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ExecutionChecklistState {
    items: Vec<PlanChecklistItem>,
    last_emitted_items: Option<Vec<PlanChecklistItem>>,
}

impl ExecutionChecklistState {
    fn stream_items(&self) -> Vec<PlanChecklistItem> {
        let visible = self
            .items
            .iter()
            .filter(|item| item.id != "initial-action")
            .cloned()
            .collect::<Vec<_>>();

        if visible.is_empty() {
            return self.items.clone();
        }

        visible
    }

    fn emit(&mut self, trace: &StructuredTurnTrace) {
        let items = self.stream_items();
        if self.last_emitted_items.as_ref() == Some(&items) {
            return;
        }

        trace.emit(TurnEvent::PlanUpdated {
            items: items.clone(),
        });
        self.last_emitted_items = Some(items);
    }

    fn sync_loop_state_notes(&self, loop_state: &mut PlannerLoopState) {
        const EXECUTION_CHECKLIST_NOTE_PREFIX: &str = "Execution checklist:";

        loop_state
            .notes
            .retain(|note| !note.starts_with(EXECUTION_CHECKLIST_NOTE_PREFIX));

        let pending = self
            .items
            .iter()
            .filter(|item| item.status == PlanChecklistItemStatus::Pending)
            .collect::<Vec<_>>();
        if pending.is_empty() {
            return;
        }

        let mut lines = vec![
            "Execution checklist: advance the next unfinished item before opening new work."
                .to_string(),
        ];
        lines.extend(pending.into_iter().map(|item| format!("- {}", item.label)));
        loop_state.notes.push(lines.join("\n"));
    }

    fn mark_completed(&mut self, item_id: &str) -> bool {
        let Some(item) = self.items.iter_mut().find(|item| item.id == item_id) else {
            return false;
        };
        if item.status == PlanChecklistItemStatus::Completed {
            return false;
        }
        item.status = PlanChecklistItemStatus::Completed;
        true
    }

    fn note_replan(&mut self, stop_reason: &str) -> bool {
        let label = format!(
            "Replanned from current evidence after {}.",
            planner_budget_stop_reason_label(stop_reason)
        );

        if let Some(item) = self.items.iter_mut().find(|item| item.id == "replan") {
            let changed = item.label != label || item.status != PlanChecklistItemStatus::Completed;
            item.label = label;
            item.status = PlanChecklistItemStatus::Completed;
            return changed;
        }

        let insert_at = self
            .items
            .iter()
            .position(|item| item.id == "apply-edit")
            .unwrap_or_else(|| {
                self.items
                    .iter()
                    .position(|item| item.id == "finalize")
                    .unwrap_or(self.items.len())
            });
        self.items.insert(
            insert_at,
            PlanChecklistItem {
                id: "replan".to_string(),
                label,
                status: PlanChecklistItemStatus::Completed,
            },
        );
        true
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LocalHarnessCapabilities {
    git: bool,
    rg: bool,
    gh: bool,
    cargo: bool,
    just: bool,
    nix: bool,
    keel: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PromptExecutionPath {
    SynthesizerOnly,
    PlannerThenSynthesize,
}

const POLICY_VIOLATION_DIRECT_REPLY: &str = "I can't help with that because it violates policy.";
const EDIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY: &str = "I haven't completed the requested repository edit yet. This turn stays open until Paddles applies a workspace change.";
const COMMIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY: &str = "I haven't completed the requested git commit yet. This turn stays open until Paddles records a commit in the workspace.";
const EDIT_AND_COMMIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY: &str = "I haven't completed the requested repository work yet. This turn stays open until Paddles applies the requested workspace change and records the requested git commit.";

fn fallback_execution_plan(prepared: &PreparedRuntimeLanes) -> PromptExecutionPlan {
    PromptExecutionPlan {
        intent: TurnIntent::DirectResponse,
        path: PromptExecutionPath::SynthesizerOnly,
        route_summary: format!(
            "planner lane '{}' is unavailable, so the turn will fall back to synthesizer lane '{}' for a direct response",
            prepared.planner.model_id, prepared.synthesizer.model_id
        ),
        initial_planner_decision: None,
        direct_answer: None,
        instruction_frame: None,
        initial_edit: InitialEditInstruction::default(),
        grounding: None,
    }
}

fn execution_checklist_pressure(
    prompt: &str,
    recent_turns: &[String],
    execution_plan: &PromptExecutionPlan,
) -> ExecutionChecklistPressure {
    let instruction_frame = execution_plan.instruction_frame.as_ref();
    ExecutionChecklistPressure {
        continuation: !recent_turns.is_empty(),
        multi_phase: prompt_has_execution_chain(prompt),
        edit_goal: instruction_frame.is_some_and(InstructionFrame::requires_applied_edit),
        commit_goal: instruction_frame.is_some_and(InstructionFrame::requires_applied_commit),
        grounding_goal: execution_plan.grounding.is_some(),
    }
}

fn build_execution_checklist(
    prompt: &str,
    recent_turns: &[String],
    execution_plan: &PromptExecutionPlan,
) -> Option<ExecutionChecklistState> {
    if execution_plan.path != PromptExecutionPath::PlannerThenSynthesize {
        return None;
    }

    let pressure = execution_checklist_pressure(prompt, recent_turns, execution_plan);
    if !pressure.is_active() {
        return None;
    }

    let initial_decision = execution_plan.initial_planner_decision.as_ref()?;
    let mut items = vec![PlanChecklistItem {
        id: "initial-action".to_string(),
        label: sentence_case(&initial_decision.action.summary()),
        status: PlanChecklistItemStatus::Pending,
    }];

    if execution_plan
        .instruction_frame
        .as_ref()
        .is_some_and(InstructionFrame::requires_applied_edit)
        && !decision_is_exact_edit(&initial_decision.action)
    {
        items.push(PlanChecklistItem {
            id: "apply-edit".to_string(),
            label: "Apply the requested repository change.".to_string(),
            status: PlanChecklistItemStatus::Pending,
        });
    }

    if execution_plan
        .instruction_frame
        .as_ref()
        .is_some_and(InstructionFrame::requires_applied_commit)
        && !decision_is_git_commit(&initial_decision.action)
    {
        items.push(PlanChecklistItem {
            id: "record-commit".to_string(),
            label: "Record the requested git commit.".to_string(),
            status: PlanChecklistItemStatus::Pending,
        });
    }

    let finalize_label = if execution_plan
        .instruction_frame
        .as_ref()
        .is_some_and(|frame| frame.requires_applied_edit() && frame.requires_applied_commit())
    {
        "Verify the workspace change and commit, then summarize the outcome.".to_string()
    } else if execution_plan
        .instruction_frame
        .as_ref()
        .is_some_and(InstructionFrame::requires_applied_edit)
    {
        "Verify the change and summarize the outcome.".to_string()
    } else if execution_plan
        .instruction_frame
        .as_ref()
        .is_some_and(InstructionFrame::requires_applied_commit)
    {
        "Verify the commit and summarize the outcome.".to_string()
    } else if execution_plan.grounding.is_some() {
        "Assemble the required evidence and summarize the answer.".to_string()
    } else if pressure.continuation || pressure.multi_phase {
        "Carry the remaining turn work to completion and summarize the result.".to_string()
    } else {
        "Verify the result and summarize the outcome.".to_string()
    };

    items.push(PlanChecklistItem {
        id: "finalize".to_string(),
        label: finalize_label,
        status: PlanChecklistItemStatus::Pending,
    });

    Some(ExecutionChecklistState {
        items,
        last_emitted_items: None,
    })
}

fn planner_budget_stop_reason_label(stop_reason: &str) -> String {
    stop_reason
        .strip_suffix("-exhausted")
        .unwrap_or(stop_reason)
        .replace('-', " ")
}

fn planner_budget_for_replan_attempt(
    base_budget: &PlannerBudget,
    completed_replans: usize,
) -> PlannerBudget {
    let multiplier = completed_replans.saturating_add(1);
    PlannerBudget {
        max_steps: base_budget.max_steps.saturating_mul(multiplier),
        max_branch_factor: base_budget.max_branch_factor,
        max_evidence_items: base_budget.max_evidence_items,
        max_reads: base_budget.max_reads.saturating_mul(multiplier),
        max_inspects: base_budget.max_inspects.saturating_mul(multiplier),
        max_searches: base_budget.max_searches.saturating_mul(multiplier),
        max_replans: base_budget.max_replans,
    }
}

fn stop_reason_supports_replan(stop_reason: &str) -> bool {
    stop_reason.contains("budget-exhausted")
        || stop_reason == "planner-budget-exhausted"
        || stop_reason == "instruction-unsatisfied"
}

fn sync_replan_note(
    loop_state: &mut PlannerLoopState,
    stop_reason: &str,
    instruction_frame: Option<&InstructionFrame>,
) {
    const REPLAN_NOTE_PREFIX: &str = "Replan from current evidence";

    loop_state
        .notes
        .retain(|note| !note.starts_with(REPLAN_NOTE_PREFIX));

    let mut lines = vec![format!(
        "Replan from current evidence after {}.",
        planner_budget_stop_reason_label(stop_reason)
    )];
    lines.push("Do not restart broad exploration or repeat missing or failed paths.".to_string());
    let next_step_line = match instruction_frame {
        Some(frame) if frame.requires_applied_edit() && frame.requires_applied_commit() => {
            "Choose the single most direct next step toward the requested workspace change and git commit."
        }
        Some(frame) if frame.requires_applied_commit() => {
            "Choose the single most direct next step toward recording the requested git commit."
        }
        _ => "Choose the single most direct next step toward an applied repository change.",
    };
    lines.push(next_step_line.to_string());
    if let Some(summary) = instruction_frame.and_then(InstructionFrame::candidate_summary) {
        lines.push(format!("Authored candidate files: {summary}"));
    }
    loop_state.notes.push(lines.join("\n"));
}

fn should_activate_replan(
    stop_reason: &str,
    instruction_frame: Option<&InstructionFrame>,
    completed_replans: usize,
    base_budget: &PlannerBudget,
) -> bool {
    instruction_frame.is_some_and(InstructionFrame::has_pending_workspace_obligation)
        && completed_replans < base_budget.max_replans
        && stop_reason_supports_replan(stop_reason)
}

struct ReplanActivation<'a> {
    instruction_frame: Option<&'a InstructionFrame>,
    base_budget: &'a PlannerBudget,
    completed_replans: &'a mut usize,
    budget: &'a mut PlannerBudget,
    loop_state: &'a mut PlannerLoopState,
    execution_checklist: Option<&'a mut ExecutionChecklistState>,
    trace: &'a StructuredTurnTrace,
}

fn activate_replan(stop_reason: &str, activation: ReplanActivation<'_>) -> bool {
    let ReplanActivation {
        instruction_frame,
        base_budget,
        completed_replans,
        budget,
        loop_state,
        execution_checklist,
        trace,
    } = activation;

    if !should_activate_replan(
        stop_reason,
        instruction_frame,
        *completed_replans,
        base_budget,
    ) {
        return false;
    }

    *completed_replans += 1;
    *budget = planner_budget_for_replan_attempt(base_budget, *completed_replans);
    sync_replan_note(loop_state, stop_reason, instruction_frame);

    if let Some(checklist) = execution_checklist {
        let changed = checklist.note_replan(stop_reason);
        checklist.sync_loop_state_notes(loop_state);
        if changed {
            checklist.emit(trace);
        }
    }

    trace.emit(TurnEvent::Fallback {
        stage: "replan".to_string(),
        reason: format!(
            "pending edit remained open after {}; extending planner budget to {} steps, {} reads, {} inspects, and {} searches while continuing from current evidence",
            planner_budget_stop_reason_label(stop_reason),
            budget.max_steps,
            budget.max_reads,
            budget.max_inspects,
            budget.max_searches,
        ),
    });
    true
}

fn instruction_frame_from_initial_edit(edit: &InitialEditInstruction) -> Option<InstructionFrame> {
    if edit.known_edit {
        let mut frame = InstructionFrame::for_edit(edit.candidate_files.clone());
        if let Some(resolution) = edit.resolution.clone() {
            frame.note_resolution(resolution);
        }
        Some(frame)
    } else {
        None
    }
}

fn merge_instruction_frames(
    current: Option<InstructionFrame>,
    incoming: Option<InstructionFrame>,
) -> Option<InstructionFrame> {
    match (current, incoming) {
        (None, None) => None,
        (Some(frame), None) | (None, Some(frame)) => Some(frame),
        (Some(mut current), Some(incoming)) => {
            if incoming.requires_applied_edit() {
                current.ensure_applied_edit(incoming.candidate_files.clone());
            }
            if incoming.requires_applied_commit() {
                current.ensure_applied_commit();
            }
            if current.resolution.is_none() {
                current.resolution = incoming.resolution.clone();
            }
            Some(current)
        }
    }
}

fn merge_instruction_frame_with_edit_signal(
    instruction_frame: Option<InstructionFrame>,
    edit: &InitialEditInstruction,
) -> Option<InstructionFrame> {
    if !edit.known_edit {
        return instruction_frame;
    }

    match instruction_frame {
        Some(mut frame) => {
            frame.ensure_applied_edit(edit.candidate_files.clone());
            if let Some(resolution) = edit.resolution.clone() {
                frame.note_resolution(resolution);
            }
            Some(frame)
        }
        None => instruction_frame_from_initial_edit(edit),
    }
}

fn merge_initial_edit_instruction(
    current: &InitialEditInstruction,
    inferred: &InitialEditInstruction,
) -> InitialEditInstruction {
    let mut candidate_files = current.candidate_files.clone();
    for candidate in &inferred.candidate_files {
        if !candidate_files.contains(candidate) {
            candidate_files.push(candidate.clone());
        }
    }

    InitialEditInstruction {
        known_edit: current.known_edit || inferred.known_edit,
        candidate_files,
        resolution: inferred
            .resolution
            .clone()
            .or_else(|| current.resolution.clone()),
    }
}

fn controller_prompt_edit_instruction(
    workspace_root: &Path,
    prompt: &str,
) -> InitialEditInstruction {
    if !prompt_requests_workspace_edit(prompt) {
        return InitialEditInstruction::default();
    }

    InitialEditInstruction {
        known_edit: true,
        candidate_files: known_edit_bootstrap_candidates(workspace_root, &[], prompt, 3),
        resolution: None,
    }
}

fn controller_prompt_commit_instruction(prompt: &str) -> Option<InstructionFrame> {
    if prompt_requests_git_commit(prompt) {
        Some(InstructionFrame::for_commit())
    } else {
        None
    }
}

fn prompt_requests_workspace_edit(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    let tokens = lower
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .collect::<Vec<_>>();
    let has_change_verb = tokens.iter().any(|token| {
        matches!(
            *token,
            "fix"
                | "change"
                | "update"
                | "edit"
                | "add"
                | "remove"
                | "replace"
                | "rename"
                | "bump"
                | "set"
                | "make"
                | "move"
                | "delete"
                | "revert"
                | "apply"
                | "show"
                | "hide"
                | "need"
                | "needs"
                | "should"
                | "must"
        )
    });
    if !has_change_verb {
        return false;
    }

    if [
        "src/",
        "apps/",
        "tests/",
        "test/",
        ".keel/",
        "cargo.toml",
        "cargo.lock",
        "flake.nix",
        "flake.lock",
        "readme.md",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
    {
        return true;
    }

    if prompt
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                !ch.is_ascii_alphanumeric()
                    && ch != '.'
                    && ch != '#'
                    && ch != '/'
                    && ch != '_'
                    && ch != '-'
            })
        })
        .any(is_code_anchor_token)
    {
        return true;
    }

    tokens.iter().any(|token| {
        matches!(
            *token,
            "css"
                | "class"
                | "selector"
                | "component"
                | "function"
                | "module"
                | "button"
                | "div"
                | "padding"
                | "loop"
                | "test"
                | "file"
        )
    })
}

fn prompt_requests_git_commit(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    lower.contains("git commit") || {
        let tokens = lower
            .split(|ch: char| !ch.is_ascii_alphanumeric())
            .filter(|token| !token.is_empty())
            .collect::<Vec<_>>();
        tokens.contains(&"git") && tokens.contains(&"commit")
    }
}

fn prompt_has_execution_chain(prompt: &str) -> bool {
    let lower = format!(" {} ", prompt.to_ascii_lowercase());
    [
        " then ",
        " after ",
        " before ",
        " next ",
        " finally ",
        " also ",
        " and ",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

fn sentence_case(input: &str) -> String {
    let mut chars = input.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    let mut rendered = first.to_uppercase().collect::<String>();
    rendered.push_str(chars.as_str());
    rendered
}

fn prompt_requires_repository_grounding(prompt: &str) -> bool {
    let tokens = normalized_prompt_tokens(prompt);
    if tokens.is_empty() {
        return false;
    }

    let mentions_workspace = tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "paddles" | "repository" | "repo" | "codebase" | "workspace"
        )
    });
    let local_reference = tokens
        .iter()
        .any(|token| matches!(token.as_str(), "our" | "we" | "this" | "that" | "here"));
    let architecture_subject = tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "layer"
                | "layers"
                | "architecture"
                | "architectural"
                | "component"
                | "components"
                | "feature"
                | "features"
                | "pipeline"
                | "pipelines"
                | "system"
                | "systems"
                | "stack"
                | "runtime"
                | "runtimes"
                | "planner"
                | "planners"
                | "synthesizer"
                | "synthesizers"
                | "framework"
                | "frameworks"
                | "model"
                | "models"
                | "adapter"
                | "adapters"
                | "integration"
                | "integrations"
                | "generative"
                | "multimodal"
        )
    });
    let relation_probe = tokens.iter().any(|token| {
        matches!(
            token.as_str(),
            "fit" | "fits" | "belong" | "belongs" | "part" | "lives" | "live" | "used"
        )
    });

    (mentions_workspace && (architecture_subject || relation_probe))
        || (local_reference && architecture_subject)
}

fn bootstrap_repository_grounding_initial_action(
    prompt: &str,
    decision: &InitialActionDecision,
) -> Option<InitialActionDecision> {
    match decision.action {
        InitialAction::Answer | InitialAction::Stop { .. } => {}
        _ => return None,
    }

    if !prompt_requires_repository_grounding(prompt) {
        return None;
    }

    Some(InitialActionDecision {
        action: InitialAction::Workspace {
            action: WorkspaceAction::Inspect {
                command: repository_grounding_probe_command(prompt),
            },
        },
        rationale:
            "controller bootstrapped a local repository grounding probe before direct synthesis"
                .to_string(),
        answer: None,
        edit: decision.edit.clone(),
        grounding: decision.grounding.clone(),
    })
}

fn bootstrap_git_commit_initial_action(
    prompt: &str,
    decision: &InitialActionDecision,
) -> Option<InitialActionDecision> {
    match decision.action {
        InitialAction::Answer | InitialAction::Stop { .. } => {}
        _ => return None,
    }

    if !prompt_requests_git_commit(prompt) {
        return None;
    }

    Some(InitialActionDecision {
        action: InitialAction::Workspace {
            action: WorkspaceAction::Inspect {
                command: "git status --short".to_string(),
            },
        },
        rationale:
            "controller bootstrapped a git status probe before allowing a commit turn to answer directly"
                .to_string(),
        answer: None,
        edit: decision.edit.clone(),
        grounding: decision.grounding.clone(),
    })
}

fn repository_grounding_probe_command(prompt: &str) -> String {
    let terms = repository_grounding_probe_terms(prompt);
    let pattern = if terms.is_empty() {
        "paddles|generative|planner|synthesizer|runtime".to_string()
    } else {
        terms.join("|")
    };

    format!(
        "rg -n -i --hidden --glob '!target' --glob '!node_modules' --glob '!.git' \"({pattern})\" ."
    )
}

fn repository_grounding_probe_terms(prompt: &str) -> Vec<String> {
    let tokens = normalized_prompt_tokens(prompt);
    let mut prioritized = Vec::new();

    for token in &tokens {
        if matches!(
            token.as_str(),
            "paddles"
                | "repository"
                | "repo"
                | "codebase"
                | "workspace"
                | "layer"
                | "layers"
                | "architecture"
                | "architectural"
                | "component"
                | "components"
                | "feature"
                | "features"
                | "pipeline"
                | "pipelines"
                | "system"
                | "systems"
                | "stack"
                | "runtime"
                | "runtimes"
                | "planner"
                | "planners"
                | "synthesizer"
                | "synthesizers"
                | "framework"
                | "frameworks"
                | "model"
                | "models"
                | "adapter"
                | "adapters"
                | "integration"
                | "integrations"
                | "generative"
                | "multimodal"
        ) && !prioritized.contains(token)
        {
            prioritized.push(token.clone());
        }
    }

    if !prioritized.is_empty() {
        prioritized.truncate(4);
        return prioritized;
    }

    let mut fallback = Vec::new();
    for token in tokens {
        if token.len() < 4
            || matches!(
                token.as_str(),
                "that"
                    | "this"
                    | "with"
                    | "from"
                    | "into"
                    | "your"
                    | "have"
                    | "think"
                    | "would"
                    | "could"
                    | "should"
                    | "there"
                    | "here"
                    | "perfect"
            )
        {
            continue;
        }
        if !fallback.contains(&token) {
            fallback.push(token);
        }
        if fallback.len() >= 4 {
            break;
        }
    }

    fallback
}

fn normalized_prompt_tokens(prompt: &str) -> Vec<String> {
    prompt
        .split(|ch: char| !ch.is_ascii_alphanumeric())
        .filter(|token| !token.is_empty())
        .map(|token| token.to_ascii_lowercase())
        .collect()
}

fn is_code_anchor_token(token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    if token.starts_with('.') || token.starts_with('#') {
        return token
            .chars()
            .skip(1)
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    }

    Path::new(token)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some()
}

fn instruction_unsatisfied_note(frame: &InstructionFrame) -> String {
    match (
        frame.requires_applied_edit(),
        frame.requires_applied_commit(),
        frame.candidate_summary(),
    ) {
        (true, true, Some(candidates)) => format!(
            "Instruction manifold [applied-edit, applied-commit]\nUser requested a repository change and a recorded git commit. Recommendation text is not completion. Hand the turn to the workspace editor, apply the workspace change, and record the commit before finishing. Candidate files: {candidates}"
        ),
        (true, true, None) => "Instruction manifold [applied-edit, applied-commit]\nUser requested a repository change and a recorded git commit. Recommendation text is not completion. Hand the turn to the workspace editor, apply the workspace change, and record the commit before finishing.".to_string(),
        (true, false, Some(candidates)) => format!(
            "Instruction manifold [applied-edit]\nUser requested an applied repository edit. Recommendation text is not completion. Hand the turn to the workspace editor and apply a workspace change before finishing. Candidate files: {candidates}"
        ),
        (true, false, None) => "Instruction manifold [applied-edit]\nUser requested an applied repository edit. Recommendation text is not completion. Hand the turn to the workspace editor and apply a workspace change before finishing.".to_string(),
        (false, true, _) => "Instruction manifold [applied-commit]\nUser requested a recorded git commit. Recommendation text is not completion. Inspect the current diff if needed, then record the commit before finishing.".to_string(),
        (false, false, _) => "Instruction obligations are currently satisfied.".to_string(),
    }
}

fn instruction_unsatisfied_direct_reply(frame: &InstructionFrame) -> String {
    let base = match (
        frame.requires_applied_edit(),
        frame.requires_applied_commit(),
    ) {
        (true, true) => EDIT_AND_COMMIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY,
        (true, false) => EDIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY,
        (false, true) => COMMIT_INSTRUCTION_UNSATISFIED_DIRECT_REPLY,
        (false, false) => "",
    };

    if let Some(candidates) = frame.candidate_summary() {
        format!("{base}\n\nLikely target files: {candidates}")
    } else {
        base.to_string()
    }
}

fn pending_workspace_boundary_prefix(frame: &InstructionFrame) -> &'static str {
    if frame.requires_applied_edit() {
        "workspace-editor-boundary"
    } else if frame.requires_applied_commit() {
        "workspace-commit-boundary"
    } else {
        "workspace-boundary"
    }
}

fn annotate_stop_reason_for_pending_instruction(
    stop_reason: String,
    instruction_frame: Option<&InstructionFrame>,
) -> String {
    let Some(frame) = instruction_frame else {
        return stop_reason;
    };
    let boundary_prefix = pending_workspace_boundary_prefix(frame);
    if !frame.has_pending_workspace_obligation() || stop_reason.contains(boundary_prefix) {
        return stop_reason;
    }

    if stop_reason == "instruction-unsatisfied" {
        boundary_prefix.to_string()
    } else if stop_reason.contains("budget") || stop_reason.contains("challenge") {
        format!("{boundary_prefix}:{stop_reason}")
    } else {
        stop_reason
    }
}

fn blocked_instruction_response(frame: &InstructionFrame) -> AuthoredResponse {
    AuthoredResponse::from_plain_text(
        ResponseMode::BlockedEdit,
        &instruction_unsatisfied_direct_reply(frame),
    )
}

fn execution_plan_from_initial_action(
    prepared: &PreparedRuntimeLanes,
    decision: InitialActionDecision,
) -> PromptExecutionPlan {
    let InitialActionDecision {
        action,
        rationale,
        answer,
        edit,
        grounding,
    } = decision;
    let instruction_frame = instruction_frame_from_initial_edit(&edit);
    match action {
        InitialAction::Answer => {
            let direct_answer = normalized_direct_answer(answer);
            let route_summary = if direct_answer.is_some() {
                "model selected a direct response; controller will render it directly".to_string()
            } else {
                format!(
                    "model selected a direct response on synthesizer lane '{}'",
                    prepared.synthesizer.model_id
                )
            };

            PromptExecutionPlan {
                intent: TurnIntent::DirectResponse,
                path: PromptExecutionPath::SynthesizerOnly,
                route_summary,
                initial_planner_decision: None,
                direct_answer,
                instruction_frame,
                initial_edit: edit,
                grounding,
            }
        }
        InitialAction::Stop { reason } => {
            let direct_answer = stop_reason_direct_answer(&reason, answer);
            let route_summary = if direct_answer.is_some() {
                format!(
                    "model selected stop before recursive resource use ({reason}); controller will render the direct response"
                )
            } else {
                format!(
                    "model selected stop before recursive resource use ({reason}); synthesizer lane '{}' will answer directly",
                    prepared.synthesizer.model_id
                )
            };

            PromptExecutionPlan {
                intent: TurnIntent::DirectResponse,
                path: PromptExecutionPath::SynthesizerOnly,
                route_summary,
                initial_planner_decision: None,
                direct_answer,
                instruction_frame,
                initial_edit: edit,
                grounding,
            }
        }
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
                    answer: None,
                    edit: edit.clone(),
                    grounding: grounding.clone(),
                }),
                direct_answer: None,
                instruction_frame,
                initial_edit: edit,
                grounding,
            }
        }
    }
}

fn format_gatherer_capability(capability: &GathererCapability) -> String {
    match capability {
        GathererCapability::Available => "available".to_string(),
        GathererCapability::Warming { reason } => format!("warming: {reason}"),
        GathererCapability::Unsupported { reason } => format!("unsupported: {reason}"),
        GathererCapability::HarnessRequired { reason } => {
            format!("harness-required: {reason}")
        }
    }
}

fn gatherer_readiness_label(capability: &GathererCapability) -> &'static str {
    match capability {
        GathererCapability::Available => "available",
        GathererCapability::Warming { .. } => "warming",
        GathererCapability::Unsupported { .. } => "unsupported",
        GathererCapability::HarnessRequired { .. } => "harness-required",
    }
}

fn planner_runtime_notes_for_gatherer(gatherer: Option<&Arc<dyn ContextGatherer>>) -> Vec<String> {
    let Some(gatherer) = gatherer else {
        return Vec::new();
    };

    let lexical = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Lexical),
    );
    let vector = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Vector),
    );

    if matches!(lexical, GathererCapability::Available)
        && matches!(vector, GathererCapability::Available)
    {
        return Vec::new();
    }

    let guidance = match (&lexical, &vector) {
        (GathererCapability::Available, GathererCapability::Warming { .. }) => {
            "Prefer bm25 if search is needed immediately; avoid vector until warmup completes."
                .to_string()
        }
        (GathererCapability::Warming { .. }, GathererCapability::Available) => {
            "Prefer vector if semantic retrieval is already warm; avoid bm25 until warmup completes."
                .to_string()
        }
        (GathererCapability::Warming { .. }, GathererCapability::Warming { .. }) => {
            "Avoid `search` or `refine` until the requested strategy is ready. Prefer `list_files`, `read`, or `inspect` for now."
                .to_string()
        }
        _ => {
            "Do not choose `search` or `refine` unless the requested retrieval strategy reports available."
                .to_string()
        }
    };

    vec![format!(
        "Workspace retrieval readiness: bm25={}, vector={}. {}",
        gatherer_readiness_label(&lexical),
        gatherer_readiness_label(&vector),
        guidance
    )]
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

fn planner_budget_for_turn(
    instruction_frame: Option<&InstructionFrame>,
    initial_edit: &InitialEditInstruction,
) -> PlannerBudget {
    if instruction_frame.is_some_and(InstructionFrame::requires_applied_edit)
        || initial_edit.known_edit
    {
        PlannerBudget {
            max_steps: 10,
            max_reads: 4,
            max_inspects: 3,
            max_searches: 2,
            max_replans: 1,
            ..PlannerBudget::default()
        }
    } else if instruction_frame.is_some_and(InstructionFrame::requires_applied_commit) {
        PlannerBudget {
            max_steps: 6,
            max_reads: 0,
            max_inspects: 2,
            max_searches: 0,
            max_replans: 1,
            ..PlannerBudget::default()
        }
    } else {
        PlannerBudget::default()
    }
}

fn local_harness_capabilities() -> &'static LocalHarnessCapabilities {
    static CAPABILITIES: OnceLock<LocalHarnessCapabilities> = OnceLock::new();
    CAPABILITIES.get_or_init(LocalHarnessCapabilities::probe)
}

impl LocalHarnessCapabilities {
    fn probe() -> Self {
        Self {
            git: command_available("git"),
            rg: command_available("rg"),
            gh: command_available("gh"),
            cargo: command_available("cargo"),
            just: command_available("just"),
            nix: command_available("nix"),
            keel: command_available("keel"),
        }
    }

    fn labels(&self) -> Vec<&'static str> {
        let mut labels = Vec::new();
        if self.git {
            labels.push("git");
        }
        if self.rg {
            labels.push("rg");
        }
        if self.gh {
            labels.push("gh");
        }
        if self.cargo {
            labels.push("cargo");
        }
        if self.just {
            labels.push("just");
        }
        if self.nix {
            labels.push("nix");
        }
        if self.keel {
            labels.push("keel");
        }
        labels
    }
}

fn command_available(command: &str) -> bool {
    std::process::Command::new("sh")
        .arg("-lc")
        .arg(format!("command -v {command} >/dev/null 2>&1"))
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

fn enrich_interpretation_with_local_harness_profile(
    mut context: InterpretationContext,
    capabilities: &LocalHarnessCapabilities,
) -> InterpretationContext {
    let available = capabilities.labels();
    if available.is_empty() {
        return context;
    }

    let source = "paddles-harness";
    let capability_summary = format!(
        "Paddles can execute local workspace actions through the harness. Available local tools: {}.",
        available.join(", ")
    );
    if context.summary.trim().is_empty() {
        context.summary = capability_summary.clone();
    } else if !context
        .summary
        .contains("Paddles can execute local workspace actions through the harness")
    {
        context.summary = format!("{}\n\n{}", context.summary.trim(), capability_summary);
    }

    if capabilities.git {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::Inspect {
                    command: "git status --short".to_string(),
                },
                note: "Start by checking the local workspace state before asking the user for repository status.".to_string(),
            },
        );
    }

    if capabilities.rg {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::Inspect {
                    command: "rg --files".to_string(),
                },
                note: "Prefer `rg --files` for file discovery and `rg -n` for text search; prefer `rg` over `grep` in this harness.".to_string(),
            },
        );
    }

    if capabilities.keel {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::Inspect {
                    command: "keel doctor --status".to_string(),
                },
                note: "Use keel directly for board health and structural drift when the task touches missions, stories, or repo health.".to_string(),
            },
        );
    }

    if capabilities.gh {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::Inspect {
                    command: "gh run list --limit 10".to_string(),
                },
                note: "Use the GitHub CLI locally when repository work touches pull requests, checks, Actions runs, or workflow state.".to_string(),
            },
        );
    }

    append_interpretation_procedure(
        &mut context,
        InterpretationProcedure {
            source: source.to_string(),
            label: "Inspect Local Workspace".to_string(),
            purpose: "Probe local repository state and find the relevant files before asking the user for information the harness can discover itself.".to_string(),
            steps: local_workspace_procedure_steps(capabilities),
        },
    );

    if capabilities.gh {
        append_interpretation_procedure(
            &mut context,
            InterpretationProcedure {
                source: source.to_string(),
                label: "Diagnose CI Or Actions".to_string(),
                purpose: "Use local and GitHub-aware tools to inspect failing CI and reproduce the failure inside the harness when possible.".to_string(),
                steps: ci_diagnostic_procedure_steps(capabilities),
            },
        );
    }

    context
}

fn append_interpretation_tool_hint(
    context: &mut InterpretationContext,
    hint: InterpretationToolHint,
) {
    let duplicate = context.tool_hints.iter().any(|existing| {
        existing.source == hint.source
            && existing.action.summary() == hint.action.summary()
            && existing.note == hint.note
    });
    if !duplicate {
        context.tool_hints.push(hint);
    }
}

fn append_interpretation_procedure(
    context: &mut InterpretationContext,
    procedure: InterpretationProcedure,
) {
    if procedure.steps.is_empty() {
        return;
    }
    let duplicate = context
        .decision_framework
        .procedures
        .iter()
        .any(|existing| existing.source == procedure.source && existing.label == procedure.label);
    if !duplicate {
        context.decision_framework.procedures.push(procedure);
    }
}

fn local_workspace_procedure_steps(
    capabilities: &LocalHarnessCapabilities,
) -> Vec<InterpretationProcedureStep> {
    let mut steps = Vec::new();
    if capabilities.git {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "git status --short".to_string(),
            },
            note: "Read the local workspace state first.".to_string(),
        });
    }
    if capabilities.rg {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "rg --files".to_string(),
            },
            note: "Use ripgrep for fast file discovery; prefer it over slower directory scans."
                .to_string(),
        });
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "rg -n \"pattern\" src".to_string(),
            },
            note: "Use `rg -n` for targeted text search; prefer it over `grep` in this harness."
                .to_string(),
        });
    }
    steps
}

fn ci_diagnostic_procedure_steps(
    capabilities: &LocalHarnessCapabilities,
) -> Vec<InterpretationProcedureStep> {
    let mut steps = Vec::new();
    if capabilities.gh {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "gh run list --limit 10".to_string(),
            },
            note: "Inspect recent GitHub Actions runs locally.".to_string(),
        });
    }
    if capabilities.nix && capabilities.just {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Shell {
                command: "nix develop --command just test".to_string(),
            },
            note: "Reproduce the CI test path locally inside the Nix shell when possible."
                .to_string(),
        });
    } else if capabilities.just {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Shell {
                command: "just test".to_string(),
            },
            note: "Reproduce the repo test path locally with just.".to_string(),
        });
    } else if capabilities.cargo {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Shell {
                command: "cargo test -q".to_string(),
            },
            note: "Reproduce the Rust test path locally.".to_string(),
        });
    }
    steps
}

fn known_edit_bootstrap_candidates(
    workspace_root: &Path,
    hinted_paths: &[String],
    prompt: &str,
    limit: usize,
) -> Vec<String> {
    const MIN_EDIT_TARGET_SCORE: i32 = 60;
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let normalized_hints = hinted_paths
        .iter()
        .filter_map(|path| normalize_known_edit_candidate(workspace_root, &path_policy, path))
        .collect::<Vec<_>>();
    let tokens = known_edit_search_tokens(prompt, &normalized_hints);
    let mut scored = HashMap::<String, i32>::new();

    for (index, path) in normalized_hints.iter().enumerate() {
        scored.insert(
            path.clone(),
            400 + score_action_bias_path(path) - index as i32 * 10,
        );
    }

    visit_known_edit_candidates(
        workspace_root,
        workspace_root,
        &path_policy,
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
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let planning = PlannerConfig::default()
        .with_mode(RetrievalMode::Linear)
        .with_retrieval_strategy(RetrievalStrategy::Vector)
        .with_step_limit(1);
    if !matches!(
        gatherer.capability_for_planning(&planning),
        GathererCapability::Available
    ) {
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
    .with_planning(planning)
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
        let Some(path) = normalize_action_bias_source(&item.source, workspace_root, &path_policy)
        else {
            continue;
        };
        if let Some(score) = scored.get_mut(&path) {
            *score += 90 + evidence_rank_bonus(item.rank);
        }
    }

    if let Some(trace) = &bundle.planner {
        for (index, artifact) in trace.retained_artifacts.iter().enumerate() {
            let Some(path) =
                normalize_action_bias_source(&artifact.source, workspace_root, &path_policy)
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
                    | WorkspaceAction::Diff { .. }
                    | WorkspaceAction::WriteFile { .. }
                    | WorkspaceAction::ReplaceInFile { .. }
                    | WorkspaceAction::ApplyPatch { .. }
            }
        )
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SteeringReviewKind {
    Evidence,
    Execution,
}

impl SteeringReviewKind {
    fn stage(self) -> &'static str {
        match self {
            Self::Evidence => "premise-challenge",
            Self::Execution => "action-bias",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SteeringReviewNote {
    kind: SteeringReviewKind,
    note: String,
}

async fn review_decision_under_signals(
    prompt: &str,
    context: &PlannerLoopContext,
    budget: &PlannerBudget,
    loop_state: &PlannerLoopState,
    decision: RecursivePlannerDecision,
    workspace_root: &Path,
    trace: Arc<StructuredTurnTrace>,
) -> Result<RecursivePlannerDecision> {
    let mut review_loop_state = loop_state.clone();
    if context.initial_edit.known_edit {
        let likely_targets = likely_action_bias_targets(loop_state, workspace_root, 3);
        if let Some(resolution) = resolve_known_edit_target(
            &context.entity_resolver,
            workspace_root,
            prompt,
            &context.initial_edit.candidate_files,
            &likely_targets,
            context
                .initial_edit
                .resolution
                .as_ref()
                .or(loop_state.target_resolution.as_ref()),
        )
        .await
        {
            review_loop_state.target_resolution = Some(resolution);
        }
    }

    let steering_notes =
        collect_steering_review_notes(context, &review_loop_state, &decision, workspace_root);
    if steering_notes.is_empty() {
        return Ok(decision);
    }

    for note in &steering_notes {
        review_loop_state.notes.push(note.note.clone());
    }

    let request = PlannerRequest::new(
        prompt,
        workspace_root.to_path_buf(),
        context.interpretation.clone(),
        budget.clone(),
    )
    .with_recent_turns(context.recent_turns.clone())
    .with_recent_thread_summary(context.recent_thread_summary.clone())
    .with_runtime_notes(planner_runtime_notes_for_gatherer(
        context.gatherer.as_ref(),
    ))
    .with_loop_state(review_loop_state)
    .with_resolver(context.resolver.clone())
    .with_entity_resolver(context.entity_resolver.clone());

    let reviewed = context
        .planner_engine
        .select_next_action(&request, trace.clone() as Arc<dyn TurnEventSink>)
        .await?;

    if steering_review_failed_closed(&reviewed) {
        return Ok(decision);
    }

    if reviewed != decision {
        let stage = steering_review_stage(&steering_notes, &reviewed);
        trace.emit(TurnEvent::Fallback {
            stage: stage.to_string(),
            reason: format_steering_review_fallback_reason(&decision, &reviewed),
        });
    }

    Ok(reviewed)
}

fn collect_steering_review_notes(
    context: &PlannerLoopContext,
    loop_state: &PlannerLoopState,
    decision: &RecursivePlannerDecision,
    workspace_root: &Path,
) -> Vec<SteeringReviewNote> {
    let mut notes = Vec::new();

    if !loop_state.evidence_items.is_empty()
        && !decision.action.is_terminal()
        && !decision_targets_file(&decision.action)
    {
        notes.push(SteeringReviewNote {
            kind: SteeringReviewKind::Evidence,
            note: format_premise_challenge_review_note(decision, loop_state),
        });
    }

    let likely_targets = if context.initial_edit.known_edit {
        let likely_targets = resolution_backed_action_bias_targets(loop_state, workspace_root, 3);
        if likely_targets.is_empty() {
            normalize_candidate_files(workspace_root, &context.initial_edit.candidate_files, 3)
        } else {
            likely_targets
        }
    } else {
        Vec::new()
    };

    if context.initial_edit.known_edit
        && should_apply_execution_review(loop_state, decision, &likely_targets)
    {
        notes.push(SteeringReviewNote {
            kind: SteeringReviewKind::Execution,
            note: format_action_bias_review_note(
                decision,
                &likely_targets,
                workspace_editor_pressure(loop_state, decision, &likely_targets),
                loop_state.target_resolution.as_ref(),
            ),
        });
    }

    if context
        .instruction_frame
        .as_ref()
        .is_some_and(InstructionFrame::requires_applied_commit)
        && should_apply_commit_review(loop_state, decision)
    {
        notes.push(SteeringReviewNote {
            kind: SteeringReviewKind::Execution,
            note: format_commit_bias_review_note(decision, git_commit_pressure(loop_state)),
        });
    }

    notes
}

fn should_apply_execution_review(
    loop_state: &PlannerLoopState,
    decision: &RecursivePlannerDecision,
    likely_targets: &[String],
) -> bool {
    if decision_is_exact_edit(&decision.action) {
        return false;
    }

    if let Some(resolved_path) = loop_state
        .target_resolution
        .as_ref()
        .and_then(EntityResolutionOutcome::resolved_path)
    {
        match decision_workspace_path(&decision.action) {
            Some(path) if path == resolved_path => {}
            _ => return true,
        }
    }

    if !has_file_targeting_step(loop_state) && !decision_targets_file(&decision.action) {
        return true;
    }

    let pressure = workspace_editor_pressure(loop_state, decision, likely_targets);
    pressure.has_read_target || pressure.repeated_read.is_some()
}

fn normalize_candidate_files(
    workspace_root: &Path,
    candidates: &[String],
    limit: usize,
) -> Vec<String> {
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    candidates
        .iter()
        .filter_map(|candidate| {
            normalize_known_edit_candidate(workspace_root, &path_policy, candidate)
        })
        .take(limit)
        .collect()
}

fn format_premise_challenge_review_note(
    decision: &RecursivePlannerDecision,
    loop_state: &PlannerLoopState,
) -> String {
    let mut lines = vec![
        "Steering review [premise-challenge]".to_string(),
        format!("Proposed action under review: {}", decision.action.summary()),
        "Treat the reported failure as a hypothesis and judge the gathered sources before spending more budget.".to_string(),
        "If the current sources already weaken or resolve the premise, choose `stop` and let synthesis judge them. Otherwise choose the single most informative next action.".to_string(),
        "Source snapshot:".to_string(),
    ];

    for item in loop_state.evidence_items.iter().take(3) {
        lines.push(format!(
            "- {}: {}",
            item.source,
            trim_for_planner(&item.snippet, 180)
        ));
    }

    lines.join("\n")
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct WorkspaceEditorPressure {
    has_read_target: bool,
    repeated_read: Option<(String, usize)>,
}

fn workspace_editor_pressure(
    loop_state: &PlannerLoopState,
    decision: &RecursivePlannerDecision,
    likely_targets: &[String],
) -> WorkspaceEditorPressure {
    let read_counts = prior_read_counts(loop_state);
    let has_read_target = likely_targets
        .iter()
        .any(|path| read_counts.contains_key(path));
    let repeated_read = match &decision.action {
        PlannerAction::Workspace {
            action: WorkspaceAction::Read { path },
        } => read_counts
            .get(path)
            .copied()
            .filter(|count| *count >= 1)
            .map(|count| (path.clone(), count)),
        _ => None,
    };

    WorkspaceEditorPressure {
        has_read_target,
        repeated_read,
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct GitCommitPressure {
    status_inspected: bool,
    diff_inspected: bool,
    commit_attempted: bool,
}

fn git_commit_pressure(loop_state: &PlannerLoopState) -> GitCommitPressure {
    let mut pressure = GitCommitPressure::default();

    for step in &loop_state.steps {
        match &step.action {
            PlannerAction::Workspace {
                action: WorkspaceAction::Inspect { command },
            } => {
                let command = command.trim().to_ascii_lowercase();
                if command == "git status --short"
                    || command == "git status"
                    || command.starts_with("git status ")
                {
                    pressure.status_inspected = true;
                }
                if command == "git diff"
                    || command.starts_with("git diff ")
                    || command.starts_with("git diff --")
                {
                    pressure.diff_inspected = true;
                }
            }
            PlannerAction::Workspace {
                action: WorkspaceAction::Diff { .. },
            } => {
                pressure.diff_inspected = true;
            }
            PlannerAction::Workspace {
                action: WorkspaceAction::Shell { command },
            } => {
                if is_git_commit_command(command) {
                    pressure.commit_attempted = true;
                }
            }
            _ => {}
        }
    }

    pressure
}

fn should_apply_commit_review(
    loop_state: &PlannerLoopState,
    decision: &RecursivePlannerDecision,
) -> bool {
    if decision_is_git_commit(&decision.action) {
        return false;
    }

    let pressure = git_commit_pressure(loop_state);
    !pressure.commit_attempted
        && pressure.status_inspected
        && pressure.diff_inspected
        && (decision.action.is_terminal()
            || matches!(
                decision.action,
                PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect { .. }
                }
            ))
}

fn format_commit_bias_review_note(
    decision: &RecursivePlannerDecision,
    pressure: GitCommitPressure,
) -> String {
    let mut lines = vec![
        "Steering review [action-bias]".to_string(),
        format!(
            "Proposed action under review: {}",
            decision.action.summary()
        ),
        "This turn is commit-oriented. The user asked Paddles to record a git commit.".to_string(),
        "Advice is not completion for this turn.".to_string(),
    ];

    if pressure.status_inspected {
        lines.push("`git status` has already been inspected.".to_string());
    }
    if pressure.diff_inspected {
        lines.push("`git diff` has already been inspected.".to_string());
    }

    lines.push(
        "Do not stop with guidance or spend another inspect unless commit safety is still unclear."
            .to_string(),
    );
    lines.push(
        "Choose the single safest next step toward a recorded commit. Prefer `shell git commit -m \"...\"` now."
            .to_string(),
    );
    lines.join("\n")
}

fn prior_read_counts(loop_state: &PlannerLoopState) -> HashMap<String, usize> {
    let mut counts = HashMap::new();
    for step in &loop_state.steps {
        if let PlannerAction::Workspace {
            action: WorkspaceAction::Read { path },
        } = &step.action
        {
            *counts.entry(path.clone()).or_insert(0) += 1;
        }
    }
    counts
}

fn workspace_action_path(action: &WorkspaceAction) -> Option<&str> {
    match action {
        WorkspaceAction::Read { path }
        | WorkspaceAction::Diff { path: Some(path) }
        | WorkspaceAction::WriteFile { path, .. }
        | WorkspaceAction::ReplaceInFile { path, .. } => Some(path.as_str()),
        WorkspaceAction::ListFiles { .. }
        | WorkspaceAction::Diff { path: None }
        | WorkspaceAction::ApplyPatch { .. }
        | WorkspaceAction::Search { .. }
        | WorkspaceAction::Inspect { .. }
        | WorkspaceAction::Shell { .. } => None,
    }
}

fn maybe_promote_missing_resolution_for_mutation(
    workspace_root: &Path,
    candidate_files: &[String],
    loop_state: &mut PlannerLoopState,
    action: &WorkspaceAction,
) {
    let Some(EntityResolutionOutcome::Missing { .. }) = loop_state.target_resolution.as_ref()
    else {
        return;
    };
    let Some(path) = workspace_action_path(action) else {
        return;
    };
    let Some(normalized_path) = normalize_candidate_files(workspace_root, &[path.to_string()], 1)
        .into_iter()
        .next()
    else {
        return;
    };

    let normalized_candidates = normalize_candidate_files(workspace_root, candidate_files, 8);
    let read_counts = prior_read_counts(loop_state);
    let supported_by_turn_state = normalized_candidates
        .iter()
        .any(|candidate| candidate == &normalized_path)
        || read_counts.contains_key(&normalized_path);
    if !supported_by_turn_state {
        return;
    }

    loop_state.target_resolution = Some(EntityResolutionOutcome::Resolved {
        target: EntityResolutionCandidate::new(normalized_path, EntityLookupMode::ExactPath, 1),
        alternatives: Vec::new(),
        explanation:
            "exact mutation path matched an authored candidate already present in the turn state"
                .to_string(),
    });
}

fn decision_is_exact_edit(action: &PlannerAction) -> bool {
    matches!(
        action,
        PlannerAction::Workspace {
            action: WorkspaceAction::WriteFile { .. }
                | WorkspaceAction::ReplaceInFile { .. }
                | WorkspaceAction::ApplyPatch { .. }
        }
    )
}

fn is_git_commit_command(command: &str) -> bool {
    let trimmed = command.trim();
    trimmed == "git commit" || trimmed.starts_with("git commit ")
}

fn decision_is_git_commit(action: &PlannerAction) -> bool {
    matches!(
        action,
        PlannerAction::Workspace {
            action: WorkspaceAction::Shell { command }
        } if is_git_commit_command(command)
    )
}

fn unresolved_target_mutation_boundary(
    action: &WorkspaceAction,
    loop_state: &PlannerLoopState,
) -> Option<(String, String)> {
    if !matches!(
        action,
        WorkspaceAction::WriteFile { .. }
            | WorkspaceAction::ReplaceInFile { .. }
            | WorkspaceAction::ApplyPatch { .. }
    ) {
        return None;
    }

    match loop_state.target_resolution.as_ref()? {
        EntityResolutionOutcome::Resolved { .. } => None,
        EntityResolutionOutcome::Ambiguous {
            candidates,
            explanation,
        } => {
            let candidate_summary = candidates
                .iter()
                .take(3)
                .map(|candidate| candidate.path.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            let reason = if candidate_summary.is_empty() {
                format!(
                    "deterministic entity resolution remained ambiguous; safe workspace mutation is blocked. {explanation}"
                )
            } else {
                format!(
                    "deterministic entity resolution remained ambiguous; safe workspace mutation is blocked until the target is narrowed. Candidates: {candidate_summary}. {explanation}"
                )
            };
            Some(("unresolved-entity-target:ambiguous".to_string(), reason))
        }
        EntityResolutionOutcome::Missing { explanation, .. } => Some((
            "unresolved-entity-target:missing".to_string(),
            format!(
                "deterministic entity resolution did not find a safe authored target; safe workspace mutation is blocked. {explanation}"
            ),
        )),
    }
}

fn format_action_bias_review_note(
    decision: &RecursivePlannerDecision,
    likely_targets: &[String],
    pressure: WorkspaceEditorPressure,
    resolution: Option<&EntityResolutionOutcome>,
) -> String {
    let mut lines = vec![
        "Steering review [action-bias]".to_string(),
        format!("Proposed action under review: {}", decision.action.summary()),
        "This turn is edit-oriented. Action produces information.".to_string(),
        "Workspace editor pressure: active.".to_string(),
        "If there is a plausible target file, prefer read/diff/edit over another broad search, `list_files`, inspect, or generic gather." .to_string(),
        "Avoid repeating list/search/inspect in the same turn when an exact file is already likely.".to_string(),
        "If the requested change is local and mechanical (padding, copy, one selector, one condition, or a small UI tweak), move into exact-diff state space now.".to_string(),
        "Hand the turn to the workspace editor. Use `replace_in_file` when you can name the exact old and new text. Use `apply_patch` when the change spans a few nearby lines.".to_string(),
    ];

    if let Some(resolution) = resolution {
        match resolution {
            EntityResolutionOutcome::Resolved { target, .. } => lines.push(format!(
                "Deterministic resolver outcome: resolved -> {}",
                target.path
            )),
            EntityResolutionOutcome::Ambiguous { candidates, .. } => {
                lines.push("Deterministic resolver outcome: ambiguous.".to_string());
                for candidate in candidates.iter().take(3) {
                    lines.push(format!("- {}", candidate.path));
                }
            }
            EntityResolutionOutcome::Missing { explanation, .. } => lines.push(format!(
                "Deterministic resolver outcome: missing -> {}",
                explanation
            )),
        }
    }

    if !likely_targets.is_empty() {
        lines.push("Likely target files:".to_string());
        for path in likely_targets.iter().take(3) {
            lines.push(format!("- {}", path));
        }
    }

    if let Some((path, count)) = pressure.repeated_read {
        lines.push(format!(
            "`{path}` has already been read {count} time(s). The workspace editor already has enough context; another read is unlikely to add information."
        ));
    } else if pressure.has_read_target {
        lines.push(
            "A likely target file has already been read. If the requested change is concrete, let the workspace editor act directly instead of rereading."
                .to_string(),
        );
    }

    lines.join("\n")
}

fn steering_review_failed_closed(decision: &RecursivePlannerDecision) -> bool {
    matches!(
        decision.action,
        PlannerAction::Stop { ref reason }
            if reason.contains("planner-action-unavailable")
    )
}

fn steering_review_stage(
    notes: &[SteeringReviewNote],
    reviewed: &RecursivePlannerDecision,
) -> &'static str {
    if decision_targets_file(&reviewed.action)
        && notes
            .iter()
            .any(|note| note.kind == SteeringReviewKind::Execution)
    {
        SteeringReviewKind::Execution.stage()
    } else if reviewed.action.is_terminal()
        && notes
            .iter()
            .any(|note| note.kind == SteeringReviewKind::Evidence)
    {
        SteeringReviewKind::Evidence.stage()
    } else {
        notes
            .first()
            .map(|note| note.kind.stage())
            .unwrap_or("steering-review")
    }
}

fn decision_targets_file(action: &PlannerAction) -> bool {
    matches!(
        action,
        PlannerAction::Workspace {
            action: WorkspaceAction::Read { .. }
                | WorkspaceAction::Diff { .. }
                | WorkspaceAction::WriteFile { .. }
                | WorkspaceAction::ReplaceInFile { .. }
                | WorkspaceAction::ApplyPatch { .. }
        }
    )
}

fn likely_action_bias_targets(
    loop_state: &PlannerLoopState,
    workspace_root: &Path,
    limit: usize,
) -> Vec<String> {
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let mut scored = HashMap::<String, i32>::new();

    for item in &loop_state.evidence_items {
        let Some(path) = normalize_action_bias_source(&item.source, workspace_root, &path_policy)
        else {
            continue;
        };
        let score = score_action_bias_path(&path) + evidence_rank_bonus(item.rank);
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
            let Some(path) =
                normalize_action_bias_source(&artifact.source, workspace_root, &path_policy)
            else {
                continue;
            };
            let score = score_action_bias_path(&path) + (40 - index as i32 * 5);
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

fn resolution_backed_action_bias_targets(
    loop_state: &PlannerLoopState,
    workspace_root: &Path,
    limit: usize,
) -> Vec<String> {
    if let Some(resolution) = loop_state.target_resolution.as_ref() {
        let candidates = resolution.candidate_paths();
        if !candidates.is_empty() {
            return candidates.into_iter().take(limit).collect();
        }
    }

    likely_action_bias_targets(loop_state, workspace_root, limit)
}

fn decision_workspace_path(action: &PlannerAction) -> Option<&str> {
    match action {
        PlannerAction::Workspace {
            action:
                WorkspaceAction::Read { path }
                | WorkspaceAction::Diff { path: Some(path) }
                | WorkspaceAction::WriteFile { path, .. }
                | WorkspaceAction::ReplaceInFile { path, .. },
        } => Some(path.as_str()),
        _ => None,
    }
}

fn normalize_known_edit_candidate(
    workspace_root: &Path,
    path_policy: &WorkspacePathPolicy,
    path: &str,
) -> Option<String> {
    let normalized = normalize_action_bias_source(path, workspace_root, path_policy)?;
    workspace_root
        .join(&normalized)
        .is_file()
        .then_some(normalized)
}

fn visit_known_edit_candidates(
    dir: &Path,
    workspace_root: &Path,
    path_policy: &WorkspacePathPolicy,
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
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            let Some(relative_dir) = path
                .strip_prefix(workspace_root)
                .ok()
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            else {
                continue;
            };
            if !path_policy.allows_relative_directory(&relative_dir) {
                continue;
            }
            visit_known_edit_candidates(
                &path,
                workspace_root,
                path_policy,
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
        if !is_plausible_workspace_file(path_policy, &rel) {
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
    let mut score = score_action_bias_path(path);
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

async fn resolve_known_edit_target(
    entity_resolver: &Arc<dyn EntityResolver>,
    workspace_root: &Path,
    prompt: &str,
    candidate_files: &[String],
    likely_targets: &[String],
    existing_resolution: Option<&EntityResolutionOutcome>,
) -> Option<EntityResolutionOutcome> {
    if let Some(resolution) = existing_resolution {
        return Some(sanitize_existing_entity_resolution(
            workspace_root,
            resolution,
        ));
    }

    let normalized_candidates = normalize_candidate_files(workspace_root, candidate_files, 6);
    let normalized_likely_targets = normalize_candidate_files(workspace_root, likely_targets, 6);
    let hints =
        known_edit_resolution_hints(prompt, &normalized_candidates, &normalized_likely_targets);
    let seeded_targets =
        merge_known_edit_target_lists(&normalized_candidates, &normalized_likely_targets);
    if hints.is_empty() && seeded_targets.is_empty() {
        return None;
    }

    let raw_hint = seeded_targets
        .first()
        .cloned()
        .unwrap_or_else(|| prompt.to_string());
    entity_resolver
        .resolve(
            &EntityResolutionRequest::new(workspace_root.to_path_buf(), raw_hint, hints)
                .with_likely_targets(seeded_targets),
        )
        .await
        .ok()
        .map(|outcome| sanitize_entity_resolution_outcome(workspace_root, outcome))
}

fn sanitize_existing_entity_resolution(
    workspace_root: &Path,
    outcome: &EntityResolutionOutcome,
) -> EntityResolutionOutcome {
    let EntityResolutionOutcome::Resolved {
        target,
        alternatives,
        explanation,
    } = outcome
    else {
        return sanitize_entity_resolution_outcome(workspace_root, outcome.clone());
    };

    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let Some(preserved_target) =
        normalize_action_bias_source(&target.path, workspace_root, &path_policy)
    else {
        return sanitize_entity_resolution_outcome(workspace_root, outcome.clone());
    };

    let mut sanitized_alternatives =
        sanitize_entity_resolution_candidates(workspace_root, alternatives.clone())
            .into_iter()
            .filter(|candidate| candidate.path != preserved_target)
            .collect::<Vec<_>>();
    for (index, candidate) in sanitized_alternatives.iter_mut().enumerate() {
        candidate.rank = index + 2;
    }

    EntityResolutionOutcome::Resolved {
        target: EntityResolutionCandidate::new(preserved_target, target.matched_by, 1),
        alternatives: sanitized_alternatives,
        explanation: explanation.clone(),
    }
}

fn known_edit_resolution_hints(
    prompt: &str,
    candidate_files: &[String],
    likely_targets: &[String],
) -> Vec<NormalizedEntityHint> {
    let mut hints = Vec::new();
    for candidate in merge_known_edit_target_lists(candidate_files, likely_targets) {
        push_known_edit_resolution_hint(&mut hints, EntityLookupMode::ExactPath, candidate.clone());
        push_known_edit_resolution_hint(
            &mut hints,
            EntityLookupMode::PathFragment,
            candidate.clone(),
        );

        let path = Path::new(&candidate);
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            push_known_edit_resolution_hint(
                &mut hints,
                EntityLookupMode::Basename,
                name.to_ascii_lowercase(),
            );
        }
        if let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) {
            let stem = stem.to_ascii_lowercase();
            push_known_edit_resolution_hint(&mut hints, EntityLookupMode::Basename, stem.clone());
            push_known_edit_resolution_hint(&mut hints, EntityLookupMode::SymbolFragment, stem);
        }
    }

    for token in known_edit_search_tokens(prompt, candidate_files)
        .into_iter()
        .take(6)
    {
        push_known_edit_resolution_hint(&mut hints, EntityLookupMode::SymbolFragment, token);
    }

    hints
}

fn push_known_edit_resolution_hint(
    hints: &mut Vec<NormalizedEntityHint>,
    mode: EntityLookupMode,
    value: impl Into<String>,
) {
    let hint = NormalizedEntityHint::new(mode, value.into());
    if hint.value.trim().is_empty() || hints.contains(&hint) {
        return;
    }
    hints.push(hint);
}

fn merge_known_edit_target_lists(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut merged = primary.to_vec();
    for path in secondary {
        if !merged.contains(path) {
            merged.push(path.clone());
        }
    }
    merged
}

fn merge_resolution_candidate_paths(
    candidates: &[String],
    resolution: &EntityResolutionOutcome,
) -> Vec<String> {
    merge_known_edit_target_lists(&resolution.candidate_paths(), candidates)
}

fn sanitize_entity_resolution_outcome(
    workspace_root: &Path,
    outcome: EntityResolutionOutcome,
) -> EntityResolutionOutcome {
    match outcome {
        EntityResolutionOutcome::Resolved {
            target,
            alternatives,
            explanation,
        } => {
            let sanitized = sanitize_entity_resolution_candidates(
                workspace_root,
                std::iter::once(target).chain(alternatives).collect(),
            );
            collapse_sanitized_entity_resolution(sanitized, explanation)
        }
        EntityResolutionOutcome::Ambiguous {
            candidates,
            explanation,
        } => {
            let sanitized = sanitize_entity_resolution_candidates(workspace_root, candidates);
            collapse_sanitized_entity_resolution(sanitized, explanation)
        }
        EntityResolutionOutcome::Missing { .. } => outcome,
    }
}

fn collapse_sanitized_entity_resolution(
    mut candidates: Vec<EntityResolutionCandidate>,
    explanation: String,
) -> EntityResolutionOutcome {
    match candidates.len() {
        0 => EntityResolutionOutcome::Missing {
            attempted_hints: Vec::new(),
            explanation: format!(
                "deterministic resolver returned no safe authored targets. {explanation}"
            ),
        },
        1 => EntityResolutionOutcome::Resolved {
            target: candidates.remove(0),
            alternatives: Vec::new(),
            explanation,
        },
        _ => EntityResolutionOutcome::Ambiguous {
            candidates,
            explanation,
        },
    }
}

fn sanitize_entity_resolution_candidates(
    workspace_root: &Path,
    candidates: Vec<EntityResolutionCandidate>,
) -> Vec<EntityResolutionCandidate> {
    let mut sanitized = Vec::new();
    for candidate in candidates {
        let Some(path) =
            normalize_candidate_files(workspace_root, std::slice::from_ref(&candidate.path), 1)
                .into_iter()
                .next()
        else {
            continue;
        };
        if sanitized
            .iter()
            .any(|existing: &EntityResolutionCandidate| existing.path == path)
        {
            continue;
        }
        sanitized.push(EntityResolutionCandidate::new(
            path,
            candidate.matched_by,
            sanitized.len() + 1,
        ));
    }
    sanitized
}

fn normalize_action_bias_source(
    source: &str,
    workspace_root: &Path,
    path_policy: &WorkspacePathPolicy,
) -> Option<String> {
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
    is_plausible_workspace_file(path_policy, &path_text).then_some(path_text)
}

fn is_plausible_workspace_file(path_policy: &WorkspacePathPolicy, path: &str) -> bool {
    path_policy.allows_relative_file(path)
}

fn evidence_rank_bonus(rank: usize) -> i32 {
    if rank == 0 {
        5
    } else {
        30i32.saturating_sub(rank as i32 * 3)
    }
}

fn score_action_bias_path(path: &str) -> i32 {
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

fn run_planner_inspect_command(
    workspace_root: &Path,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<String> {
    validate_inspect_command(command)?;
    let output =
        run_background_terminal_command(workspace_root, command, "inspect", call_id, event_sink)?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let rendered = if stderr.trim().is_empty() {
        stdout
    } else {
        format!("{stdout}\n{stderr}")
    };

    Ok(trim_for_planner(&rendered, 1_200))
}

fn run_planner_shell_command(
    workspace_root: &Path,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<String> {
    let output =
        run_background_terminal_command(workspace_root, command, "shell", call_id, event_sink)?;
    let summary = format_command_output_summary(command, &output);
    if !output.status.success() {
        anyhow::bail!("{summary}");
    }
    Ok(summary)
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

fn format_command_output_summary(command: &str, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let rendered = if stderr.trim().is_empty() {
        stdout
    } else if stdout.trim().is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    };
    let status = output
        .status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| output.status.to_string());

    trim_for_planner(
        &format!(
            "Shell command: {command}\nExit status: {status}\n{}",
            rendered.trim()
        ),
        1_200,
    )
}

fn planner_terminal_tool_success_summary(tool_name: &str, output: &str) -> String {
    let had_output = !output.trim().is_empty();
    match (tool_name, had_output) {
        ("inspect", true) => "inspection completed".to_string(),
        ("inspect", false) => "inspection completed with no output".to_string(),
        ("shell", true) => "command completed".to_string(),
        ("shell", false) => "command completed with no output".to_string(),
        (_, true) => "tool completed".to_string(),
        (_, false) => "tool completed with no output".to_string(),
    }
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

fn normalized_direct_answer(answer: Option<String>) -> Option<AuthoredResponse> {
    answer.and_then(|answer| {
        let trimmed = answer.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(AuthoredResponse::from_plain_text(
                ResponseMode::DirectAnswer,
                trimmed,
            ))
        }
    })
}

fn stop_reason_direct_answer(reason: &str, answer: Option<String>) -> Option<AuthoredResponse> {
    if let Some(answer) = normalized_direct_answer(answer) {
        return Some(answer);
    }

    if reason == "refusal" {
        return Some(AuthoredResponse::from_plain_text(
            ResponseMode::PolicyRefusal,
            POLICY_VIOLATION_DIRECT_REPLY,
        ));
    }

    None
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
        ActiveRuntimeState, GathererProvider, MechSuitService, POLICY_VIOLATION_DIRECT_REPLY,
        PreparedGathererLane, PreparedModelLane, PreparedRuntimeLanes, RuntimeLaneConfig,
        RuntimeLaneRole, StructuredTurnTrace, TurnIntent, budget_signal_details, render_turn_event,
    };
    use crate::domain::model::{AuthoredResponse, CompactionPlan, CompactionRequest, ResponseMode};
    use crate::domain::model::{
        ContextStrain, ConversationForensicUpdate, ConversationThreadRef,
        ConversationTranscriptUpdate, ForensicArtifactCapture, ForensicLifecycle,
        ForensicTraceSink, ForensicUpdateSink, StrainFactor, TaskTraceId, ThreadDecision,
        ThreadDecisionId, ThreadDecisionKind, TraceLineageNodeKind, TraceLineageRelation,
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase,
        TraceRecordKind, TraceSignalKind, TranscriptUpdateSink, TurnEvent, TurnEventSink,
    };
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherResult, ContextGatherer, EntityLookupMode,
        EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest,
        EntityResolver, EvidenceBundle, EvidenceItem, GroundingDomain, GroundingRequirement,
        InitialAction, InitialActionDecision, InitialEditInstruction, InterpretationContext,
        InterpretationRequest, ModelPaths, ModelRegistry, PlannerAction, PlannerBudget,
        PlannerCapability, PlannerGraphBranch, PlannerGraphBranchStatus, PlannerGraphEpisode,
        PlannerLoopState, PlannerRequest, PlannerStepRecord, PlannerStrategyKind,
        PlannerTraceMetadata, RecursivePlanner, RecursivePlannerDecision, RetainedEvidence,
        RetrievalMode, RetrievalStrategy, RetrieverOption, SynthesisHandoff, SynthesizerEngine,
        ThreadDecisionRequest, TraceRecorder, WorkspaceAction,
    };
    use crate::infrastructure::adapters::NoopContextResolver;
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
    use crate::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
    use crate::infrastructure::adapters::workspace_entity_resolver::WorkspaceEntityResolver;
    use crate::infrastructure::conversation_history::ConversationHistoryStore;
    use crate::infrastructure::providers::ModelProvider;
    use anyhow::{Result, anyhow};
    use async_trait::async_trait;
    use paddles_conversation::ConversationSession;
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

    #[derive(Default)]
    struct RecordingForensicUpdateSink {
        updates: Mutex<Vec<ConversationForensicUpdate>>,
    }

    impl RecordingForensicUpdateSink {
        fn recorded(&self) -> Vec<ConversationForensicUpdate> {
            self.updates.lock().expect("update lock").clone()
        }
    }

    impl ForensicUpdateSink for RecordingForensicUpdateSink {
        fn emit(&self, update: ConversationForensicUpdate) {
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

    #[derive(Debug)]
    struct StaticEntityResolver {
        outcome: EntityResolutionOutcome,
        recorded_requests: Arc<Mutex<Vec<EntityResolutionRequest>>>,
    }

    #[async_trait]
    impl EntityResolver for StaticEntityResolver {
        async fn resolve(
            &self,
            request: &EntityResolutionRequest,
        ) -> Result<EntityResolutionOutcome> {
            self.recorded_requests
                .lock()
                .expect("entity resolver requests lock")
                .push(request.clone());
            Ok(self.outcome.clone())
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
        handoffs: Mutex<Vec<SynthesisHandoff>>,
    }

    impl SynthesizerEngine for RecordingSynthesizer {
        fn set_verbose(&self, _level: u8) {}

        fn respond_for_turn(
            &self,
            _prompt: &str,
            _turn_intent: TurnIntent,
            gathered_evidence: Option<&EvidenceBundle>,
            handoff: &SynthesisHandoff,
            _event_sink: Arc<dyn TurnEventSink>,
        ) -> Result<String> {
            if let Some(bundle) = gathered_evidence {
                self.gathered_summaries
                    .lock()
                    .expect("gathered summaries lock")
                    .push(bundle.summary.clone());
            }
            self.handoffs
                .lock()
                .expect("handoffs lock")
                .push(handoff.clone());
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
                applied_edit: mock_applied_edit_for_action(action),
            })
        }
    }

    fn mock_applied_edit_for_action(
        action: &WorkspaceAction,
    ) -> Option<crate::domain::model::AppliedEdit> {
        match action {
            WorkspaceAction::WriteFile { path, content } => {
                Some(mock_applied_edit(path, "", content))
            }
            WorkspaceAction::ReplaceInFile { path, old, new, .. } => {
                Some(mock_applied_edit(path, old, new))
            }
            WorkspaceAction::ApplyPatch { patch } => Some(crate::domain::model::AppliedEdit {
                files: Vec::new(),
                diff: patch.clone(),
                insertions: patch
                    .lines()
                    .filter(|line| line.starts_with('+') && !line.starts_with("+++"))
                    .count(),
                deletions: patch
                    .lines()
                    .filter(|line| line.starts_with('-') && !line.starts_with("---"))
                    .count(),
            }),
            _ => None,
        }
    }

    fn mock_applied_edit(
        path: &str,
        before: &str,
        after: &str,
    ) -> crate::domain::model::AppliedEdit {
        let mut diff = vec![
            format!("--- a/{path}"),
            format!("+++ b/{path}"),
            "@@".to_string(),
        ];
        diff.extend(before.lines().map(|line| format!("-{line}")));
        diff.extend(after.lines().map(|line| format!("+{line}")));
        crate::domain::model::AppliedEdit {
            files: vec![path.to_string()],
            diff: diff.join("\n"),
            insertions: after.lines().count(),
            deletions: before.lines().count(),
        }
    }

    fn initial_action_decision(action: InitialAction, rationale: &str) -> InitialActionDecision {
        InitialActionDecision {
            action,
            rationale: rationale.to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        }
    }

    fn test_planner_loop_context(
        initial_edit: InitialEditInstruction,
    ) -> super::PlannerLoopContext {
        super::PlannerLoopContext {
            prepared: PreparedRuntimeLanes {
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
            },
            planner_engine: Arc::new(TestPlanner::new(
                initial_action_decision(InitialAction::Answer, "unused"),
                Vec::new(),
                Arc::new(Mutex::new(Vec::new())),
            )),
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
            resolver: Arc::new(NoopContextResolver),
            entity_resolver: Arc::new(WorkspaceEntityResolver::new()),
            interpretation: InterpretationContext::default(),
            recent_turns: Vec::new(),
            recent_thread_summary: None,
            instruction_frame: super::instruction_frame_from_initial_edit(&initial_edit),
            initial_edit,
            grounding: None,
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

    #[tokio::test]
    async fn prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(RecordingRegistry::default());
        let operator_memory = Arc::new(AgentMemory::load(workspace.path()));
        let captured_synthesizer_lane = Arc::new(Mutex::new(None::<PreparedModelLane>));
        let synthesizer_capture = Arc::clone(&captured_synthesizer_lane);
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
            Box::new(move |_, _lane| {
                Ok(Arc::new(TestPlanner::new(
                    initial_action_decision(InitialAction::Answer, "not used"),
                    Vec::new(),
                    Arc::new(Mutex::new(Vec::new())),
                )) as Arc<dyn RecursivePlanner>)
            }),
            Box::new(|_, _, _, _| Ok(None)),
        );
        let config = RuntimeLaneConfig::new("mercury-2".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Inception)
            .with_planner_provider(Some(ModelProvider::Inception))
            .with_planner_model_id(Some("mercury-2".to_string()));

        let prepared = service
            .prepare_runtime_lanes(&config)
            .await
            .expect("prepare runtime lanes");

        assert_eq!(prepared.synthesizer.provider, ModelProvider::Inception);
        assert_eq!(prepared.synthesizer.paths, None);
        assert_eq!(prepared.planner.provider, ModelProvider::Inception);
        assert_eq!(prepared.planner.paths, None);
        assert!(
            registry.requested_model_ids().is_empty(),
            "remote providers should not resolve sift model paths"
        );
        assert_eq!(
            captured_synthesizer_lane
                .lock()
                .expect("captured synthesizer lane lock")
                .clone()
                .expect("synthesizer lane")
                .provider,
            ModelProvider::Inception
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
    fn repository_grounding_heuristic_flags_our_generative_layer_followup() {
        assert!(super::prompt_requires_repository_grounding(
            "I think this is a perfect fit for our generative layer"
        ));
        assert!(!super::prompt_requires_repository_grounding(
            "What is the capital of France?"
        ));
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
    fn local_harness_enrichment_adds_tool_preferences_and_workspace_procedure() {
        let context = super::enrich_interpretation_with_local_harness_profile(
            InterpretationContext::default(),
            &super::LocalHarnessCapabilities {
                git: true,
                rg: true,
                ..Default::default()
            },
        );

        assert!(
            context
                .summary
                .contains("Paddles can execute local workspace actions")
        );
        assert!(context.tool_hints.iter().any(|hint| {
            matches!(
                hint.action,
                WorkspaceAction::Inspect { ref command } if command == "git status --short"
            )
        }));
        assert!(context.tool_hints.iter().any(|hint| {
            matches!(
                hint.action,
                WorkspaceAction::Inspect { ref command } if command == "rg --files"
            ) && hint.note.contains("prefer `rg` over `grep`")
        }));
        assert!(context.decision_framework.procedures.iter().any(|procedure| {
            procedure.label == "Inspect Local Workspace"
                && procedure
                    .steps
                    .iter()
                    .any(|step| matches!(
                        step.action,
                        WorkspaceAction::Inspect { ref command } if command == "rg -n \"pattern\" src"
                    ))
        }));
    }

    #[test]
    fn local_harness_enrichment_adds_general_github_hint_and_ci_procedure_when_available() {
        let context = super::enrich_interpretation_with_local_harness_profile(
            InterpretationContext::default(),
            &super::LocalHarnessCapabilities {
                gh: true,
                just: true,
                nix: true,
                ..Default::default()
            },
        );

        assert!(context.tool_hints.iter().any(|hint| {
            matches!(
                hint.action,
                WorkspaceAction::Inspect { ref command } if command == "gh run list --limit 10"
            )
        }));
        assert!(
            context
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| {
                    procedure.label == "Diagnose CI Or Actions"
                        && procedure.steps.iter().any(|step| {
                            matches!(
                                step.action,
                                WorkspaceAction::Shell { ref command }
                                    if command == "nix develop --command just test"
                            )
                        })
                })
        );
    }

    #[test]
    fn local_harness_enrichment_no_longer_uses_prompt_intent_to_gate_github_hints() {
        let context = super::enrich_interpretation_with_local_harness_profile(
            InterpretationContext::default(),
            &super::LocalHarnessCapabilities {
                gh: true,
                ..Default::default()
            },
        );

        assert!(context.tool_hints.iter().any(|hint| matches!(
            hint.action,
            WorkspaceAction::Inspect { ref command } if command == "gh run list --limit 10"
        )));
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
            vec!["AGENTS.md".to_string(), "paddles-harness".to_string()]
        );
        assert!(!requests[0].interpretation.tool_hints.is_empty());
        assert!(
            requests[0]
                .interpretation
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| procedure.label == "Inspect Local Workspace")
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
                .process_prompt_with_sink("Record this turn", sink.clone())
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
        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::HarnessState { snapshot }
                if snapshot.chamber == crate::domain::model::HarnessChamber::Planning
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::HarnessState { snapshot }
                if snapshot.chamber == crate::domain::model::HarnessChamber::Rendering
        )));
    }

    #[test]
    fn plain_turn_event_rendering_uses_hunting_language_for_gatherer_progress() {
        let rendered = render_turn_event(&TurnEvent::GathererSearchProgress {
            phase: "Indexing".to_string(),
            elapsed_seconds: 110,
            eta_seconds: Some(0),
            strategy: Some("bm25".to_string()),
            detail: Some("indexing 75914/75934 files".to_string()),
        });

        assert!(rendered.starts_with("• Hunting (Indexing)"));
        assert!(rendered.contains("strategy=bm25"));
        assert!(rendered.contains("indexing 75914/75934 files"));
    }

    #[test]
    fn structured_turn_trace_records_lineage_edges_for_model_calls_and_outputs() {
        let session = ConversationSession::new(TaskTraceId::new("task-lineage").expect("task id"));
        let turn_id = session.allocate_turn_id();
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            recorder.clone(),
            Vec::new(),
            session.clone(),
            turn_id.clone(),
            ConversationThreadRef::Mainline,
        ));
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

        trace.record_turn_start(
            "Fix the target file",
            &InterpretationContext::default(),
            &prepared,
        );
        trace.record_planner_action("read src/lib.rs", "act on the likeliest file first", None);
        let exchange_id = ForensicTraceSink::allocate_model_exchange_id(
            trace.as_ref(),
            TraceModelExchangeLane::Planner,
            TraceModelExchangeCategory::PlannerAction,
        );
        let assembled_context_id = ForensicTraceSink::record_forensic_artifact(
            trace.as_ref(),
            ForensicArtifactCapture {
                exchange_id: exchange_id.clone(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::AssembledContext,
                provider: "openai".to_string(),
                model: "gpt-test".to_string(),
                parent_artifact_id: None,
                summary: "planner assembled context".to_string(),
                content: "{\"user\":\"Fix the target file\"}".to_string(),
                mime_type: "application/json".to_string(),
                labels: Default::default(),
            },
        )
        .expect("assembled context artifact");
        let raw_response_id = ForensicTraceSink::record_forensic_artifact(
            trace.as_ref(),
            ForensicArtifactCapture {
                exchange_id: exchange_id.clone(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::RawProviderResponse,
                provider: "openai".to_string(),
                model: "gpt-test".to_string(),
                parent_artifact_id: Some(assembled_context_id.clone()),
                summary: "planner raw response".to_string(),
                content: "{\"action\":\"read\"}".to_string(),
                mime_type: "application/json".to_string(),
                labels: Default::default(),
            },
        )
        .expect("raw response artifact");
        ForensicTraceSink::record_forensic_artifact(
            trace.as_ref(),
            ForensicArtifactCapture {
                exchange_id: exchange_id.clone(),
                lane: TraceModelExchangeLane::Synthesizer,
                category: TraceModelExchangeCategory::TurnResponse,
                phase: TraceModelExchangePhase::RenderedResponse,
                provider: "openai".to_string(),
                model: "gpt-test".to_string(),
                parent_artifact_id: Some(raw_response_id.clone()),
                summary: "rendered response".to_string(),
                content: "Patched src/lib.rs".to_string(),
                mime_type: "text/plain".to_string(),
                labels: Default::default(),
            },
        )
        .expect("rendered response artifact");
        trace.record_completion(&AuthoredResponse::from_plain_text(
            ResponseMode::DirectAnswer,
            "Patched src/lib.rs",
        ));

        let replay = recorder.replay(&session.task_id()).expect("replay");
        let completion_output_id = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::CompletionCheckpoint(checkpoint) => checkpoint
                    .response
                    .as_ref()
                    .map(|artifact| artifact.artifact_id.as_str().to_string()),
                _ => None,
            })
            .expect("completion output id");
        let completion_mode = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::CompletionCheckpoint(checkpoint) => checkpoint
                    .response
                    .as_ref()
                    .and_then(|artifact| artifact.labels.get("paddles.response_mode").cloned()),
                _ => None,
            })
            .expect("completion response mode");
        let lineage_edges = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::LineageEdge(edge) => Some(edge),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(lineage_edges.iter().any(|edge| {
            edge.source.kind == TraceLineageNodeKind::Conversation
                && edge.target.kind == TraceLineageNodeKind::Turn
                && edge.relation == TraceLineageRelation::Contains
        }));
        assert!(lineage_edges.iter().any(|edge| {
            edge.source.kind == TraceLineageNodeKind::Turn
                && edge.target.kind == TraceLineageNodeKind::PlannerStep
                && edge.relation == TraceLineageRelation::Contains
        }));
        assert!(lineage_edges.iter().any(|edge| {
            edge.source.kind == TraceLineageNodeKind::PlannerStep
                && edge.target.kind == TraceLineageNodeKind::ModelCall
                && edge.target.label == exchange_id
                && edge.relation == TraceLineageRelation::Triggers
        }));
        assert!(lineage_edges.iter().any(|edge| {
            edge.source.kind == TraceLineageNodeKind::ModelCall
                && edge.source.label == exchange_id
                && edge.target.id == format!("artifact:{}", assembled_context_id.as_str())
                && edge.relation == TraceLineageRelation::Produces
        }));
        assert!(lineage_edges.iter().any(|edge| {
            edge.source.id == format!("artifact:{}", assembled_context_id.as_str())
                && edge.target.id == format!("artifact:{}", raw_response_id.as_str())
                && edge.relation == TraceLineageRelation::Transforms
        }));
        assert!(lineage_edges.iter().any(|edge| {
            edge.source.kind == TraceLineageNodeKind::ModelCall
                && edge.source.label == exchange_id
                && edge.target.id == format!("output:{completion_output_id}")
                && edge.relation == TraceLineageRelation::ResultsIn
        }));
        assert_eq!(completion_mode, "direct_answer");
    }

    #[test]
    fn structured_turn_trace_records_signal_snapshots_with_contribution_estimates() {
        let session = ConversationSession::new(TaskTraceId::new("task-force").expect("task id"));
        let turn_id = session.allocate_turn_id();
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            recorder.clone(),
            Vec::new(),
            session.clone(),
            turn_id,
            ConversationThreadRef::Mainline,
        ));
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

        trace.record_turn_start(
            "Investigate context strain",
            &InterpretationContext::default(),
            &prepared,
        );
        trace.emit(TurnEvent::ContextStrain {
            strain: ContextStrain::new(
                vec![
                    StrainFactor::MemoryTruncated,
                    StrainFactor::ArtifactTruncated,
                ],
                3,
            ),
        });
        trace.emit(TurnEvent::Fallback {
            stage: "action-bias".to_string(),
            reason: "acting on the likely file is more informative".to_string(),
        });
        trace.emit(TurnEvent::Fallback {
            stage: "planner-fallback".to_string(),
            reason: "planner response could not be parsed".to_string(),
        });
        trace.emit(TurnEvent::Fallback {
            stage: "premise-challenge".to_string(),
            reason:
                "Reviewed `inspect `gh run list --limit 10`` and kept the same action after judging the current sources."
                    .to_string(),
        });
        trace.emit(TurnEvent::Fallback {
            stage: "entity-resolution".to_string(),
            reason: "deterministic entity resolution remained ambiguous; safe workspace mutation is blocked until the target is narrowed. Candidates: src/application/mod.rs, src/domain/model/turns.rs. two authored files remained tied".to_string(),
        });
        trace.emit(TurnEvent::RefinementApplied {
            reason: "Archived deeper artifacts".to_string(),
            before_summary: "12 retained artifacts".to_string(),
            after_summary: "6 retained artifacts".to_string(),
        });
        trace.emit(TurnEvent::PlannerSummary {
            strategy: "bounded".to_string(),
            mode: "search".to_string(),
            turns: 1,
            steps: 4,
            stop_reason: Some("search-budget-exhausted".to_string()),
            active_branch_id: None,
            branch_count: None,
            frontier_count: None,
            node_count: None,
            edge_count: None,
            retained_artifact_count: None,
        });

        let replay = recorder.replay(&session.task_id()).expect("replay");
        let signal_snapshots = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::SignalSnapshot(snapshot) => Some(snapshot),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(signal_snapshots.iter().any(|snapshot| {
            snapshot.kind == TraceSignalKind::ContextStrain
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "operator_memory")
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "retained_artifacts")
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            snapshot.kind == TraceSignalKind::ActionBias
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "controller_policy")
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            snapshot.kind == TraceSignalKind::Fallback
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "provider_or_parser")
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            if snapshot.kind != TraceSignalKind::Fallback {
                return false;
            }
            let payload = snapshot
                .artifact
                .inline_content
                .as_deref()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(content).ok());
            matches!(
                payload,
                Some(serde_json::Value::Object(ref details))
                    if details.get("stage").and_then(serde_json::Value::as_str)
                        == Some("premise-challenge")
                        && snapshot
                            .contributions
                            .iter()
                            .any(|item| item.source == "premise_challenge")
                        && snapshot
                            .contributions
                            .iter()
                            .all(|item| item.source != "provider_or_parser")
            )
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            if snapshot.kind != TraceSignalKind::Fallback {
                return false;
            }
            let payload = snapshot
                .artifact
                .inline_content
                .as_deref()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(content).ok());
            matches!(
                payload,
                Some(serde_json::Value::Object(ref details))
                    if details.get("stage").and_then(serde_json::Value::as_str)
                        == Some("premise-challenge")
                        && snapshot.resolved_gate()
                            == crate::domain::model::SteeringGateKind::Convergence
                        && snapshot.resolved_phase()
                            == crate::domain::model::SteeringGatePhase::Narrowing
            )
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            if snapshot.kind != TraceSignalKind::Fallback {
                return false;
            }
            let payload = snapshot
                .artifact
                .inline_content
                .as_deref()
                .and_then(|content| serde_json::from_str::<serde_json::Value>(content).ok());
            matches!(
                payload,
                Some(serde_json::Value::Object(ref details))
                    if details.get("stage").and_then(serde_json::Value::as_str) == Some("entity-resolution")
                        && details.get("status").and_then(serde_json::Value::as_str) == Some("ambiguous")
            )
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            snapshot.kind == TraceSignalKind::CompactionCue
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "controller_policy")
        }));
        assert!(signal_snapshots.iter().any(|snapshot| {
            snapshot.kind == TraceSignalKind::BudgetBoundary
                && snapshot
                    .contributions
                    .iter()
                    .any(|item| item.source == "planner_budget")
        }));

        let ordered_kinds = signal_snapshots
            .iter()
            .map(|snapshot| snapshot.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_kinds,
            vec![
                TraceSignalKind::ContextStrain,
                TraceSignalKind::ActionBias,
                TraceSignalKind::Fallback,
                TraceSignalKind::Fallback,
                TraceSignalKind::Fallback,
                TraceSignalKind::CompactionCue,
                TraceSignalKind::BudgetBoundary,
            ]
        );
    }

    #[test]
    fn premise_challenge_fallback_reason_keeps_noop_review_readable() {
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "gh run list --limit 10".to_string(),
                },
            },
            rationale: "inspect recent CI runs".to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let reason = super::format_steering_review_fallback_reason(&decision, &decision);

        assert_eq!(
            reason,
            "Reviewed `inspect `gh run list --limit 10`` and kept the same action after judging the current sources."
        );
        assert!(!reason.contains("replaced"));
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

    #[tokio::test]
    async fn recent_turn_history_persists_across_service_processes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let history_store = Arc::new(ConversationHistoryStore::with_path(
            workspace.path().join("state/conversation-history.toml"),
        ));

        let service_one = test_service(workspace.path());
        service_one.set_conversation_history_store(Arc::clone(&history_store));
        let planner_requests_one = Arc::new(Mutex::new(Vec::new()));
        *service_one.runtime.write().await = Some(ActiveRuntimeState {
            prepared: PreparedRuntimeLanes {
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
            },
            planner_engine: Arc::new(TestPlanner::new(
                initial_action_decision(InitialAction::Answer, "answer directly"),
                Vec::new(),
                Arc::clone(&planner_requests_one),
            )),
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        service_one
            .process_prompt_in_session_with_sink(
                "First prompt",
                service_one.shared_conversation_session(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .await
            .expect("process first prompt");

        let service_two = test_service(workspace.path());
        service_two.set_conversation_history_store(Arc::clone(&history_store));
        let planner_requests_two = Arc::new(Mutex::new(Vec::new()));
        *service_two.runtime.write().await = Some(ActiveRuntimeState {
            prepared: PreparedRuntimeLanes {
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
            },
            planner_engine: Arc::new(TestPlanner::new(
                initial_action_decision(InitialAction::Answer, "answer directly"),
                Vec::new(),
                Arc::clone(&planner_requests_two),
            )),
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        service_two
            .process_prompt_in_session_with_sink(
                "Second prompt",
                service_two.shared_conversation_session(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .await
            .expect("process second prompt");

        let second_requests = planner_requests_two.lock().expect("planner requests lock");
        assert_eq!(second_requests.len(), 1);
        assert_eq!(
            second_requests[0].recent_turns,
            vec!["Q: First prompt A: Applied the bounded action.".to_string()]
        );

        assert_eq!(
            history_store.prompt_history().expect("prompt history"),
            vec!["First prompt".to_string(), "Second prompt".to_string()]
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
    fn replay_conversation_projection_packages_all_shared_session_surfaces() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nProject transcript, forensics, manifold, and trace graph from one canonical snapshot.\n",
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
                "Projection response.".to_string(),
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
                    "Project all shared session surfaces",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let projection = service
            .replay_conversation_projection(&session.task_id())
            .expect("projection replay");

        assert_eq!(projection.task_id, session.task_id());
        assert_eq!(projection.transcript.entries.len(), 2);
        assert_eq!(projection.forensics.turns.len(), 1);
        assert_eq!(projection.manifold.turns.len(), 1);
        assert!(!projection.trace_graph.nodes.is_empty());
        assert!(
            projection
                .trace_graph
                .nodes
                .iter()
                .any(|node| node.kind == "root")
        );
        assert!(
            projection
                .trace_graph
                .nodes
                .iter()
                .any(|node| node.kind == "action")
        );
    }

    #[test]
    fn conversation_projection_updates_are_derived_from_authoritative_replay_state() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nRebuild canonical projection updates from authoritative replay state.\n",
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
                "Projection update response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let transcript_updates = Arc::new(RecordingTranscriptUpdateSink::default());
        let forensic_updates = Arc::new(RecordingForensicUpdateSink::default());
        service.register_transcript_observer(transcript_updates.clone());
        service.register_forensic_observer(forensic_updates.clone());
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
                    "Emit projection updates",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let expected = service
            .replay_conversation_projection(&session.task_id())
            .expect("projection replay");
        let transcript_update = transcript_updates
            .recorded()
            .into_iter()
            .next()
            .expect("transcript update");
        let forensic_update = forensic_updates
            .recorded()
            .into_iter()
            .last()
            .expect("forensic update");

        let transcript_projection_update = service
            .projection_update_for_transcript(&transcript_update)
            .expect("projection transcript update");
        assert_eq!(
            transcript_projection_update.kind,
            crate::domain::model::ConversationProjectionUpdateKind::Transcript
        );
        assert_eq!(transcript_projection_update.snapshot, expected);
        assert_eq!(
            transcript_projection_update.transcript_update.as_ref(),
            Some(&transcript_update)
        );
        assert!(transcript_projection_update.forensic_update.is_none());

        let forensic_projection_update = service
            .projection_update_for_forensic(&forensic_update)
            .expect("projection forensic update");
        assert_eq!(
            forensic_projection_update.kind,
            crate::domain::model::ConversationProjectionUpdateKind::Forensic
        );
        assert_eq!(forensic_projection_update.snapshot, expected);
        assert_eq!(
            forensic_projection_update.forensic_update.as_ref(),
            Some(&forensic_update)
        );
        assert!(forensic_projection_update.transcript_update.is_none());
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
    fn process_prompt_emits_plan_updates_and_containment_notes_for_edit_turns() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nDrive edit turns to completion with explicit execution plans.\n",
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status --short".to_string(),
                    },
                },
                rationale: "inspect the current workspace state before editing".to_string(),
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["src/lib.rs".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::ApplyPatch {
                            patch: "*** Begin Patch\n*** Add File: src/lib.rs\n+pub fn plan_mode() {}\n*** End Patch\n"
                                .to_string(),
                        },
                    },
                    rationale: "apply the requested repository change".to_string(),
                    answer: None,
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["src/lib.rs".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "change applied and verified".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some("Patched the file and verified the result.".to_string()),
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["src/lib.rs".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(RecordingSynthesizer::default());
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
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
                .process_prompt_with_sink("Fix src/lib.rs and verify the result", sink.clone())
                .await
                .expect("process prompt")
        });

        let recorded = sink.recorded();
        let plan_updates = recorded
            .iter()
            .filter_map(|event| match event {
                TurnEvent::PlanUpdated { items } => Some(items.clone()),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(
            !plan_updates.is_empty(),
            "edit-oriented planned turns should emit at least one plan update"
        );
        assert!(
            plan_updates
                .iter()
                .all(|items| items.iter().all(|item| item.id != "initial-action")),
            "stream-visible plan updates should show remaining work, not restate the current planner step"
        );
        assert!(
            plan_updates.windows(2).all(|window| window[0] != window[1]),
            "plan updates should not replay identical visible checklists after the first planner step"
        );
        assert_eq!(
            plan_updates[0]
                .iter()
                .map(|item| item.status)
                .collect::<Vec<_>>(),
            vec![
                crate::domain::model::PlanChecklistItemStatus::Pending,
                crate::domain::model::PlanChecklistItemStatus::Pending,
            ]
        );
        assert!(
            plan_updates[0][0]
                .label
                .contains("Apply the requested repository change"),
            "edit turns should keep an explicit apply-change checklist item"
        );
        assert_eq!(
            plan_updates
                .last()
                .expect("final plan update")
                .iter()
                .map(|item| item.status)
                .collect::<Vec<_>>(),
            vec![
                crate::domain::model::PlanChecklistItemStatus::Completed,
                crate::domain::model::PlanChecklistItemStatus::Completed,
            ]
        );

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(
            requests
                .iter()
                .skip(1)
                .flat_map(|request| request.loop_state.notes.iter())
                .any(|note| note.contains("Execution checklist")),
            "follow-on planner requests should carry the containment checklist note"
        );
    }

    #[test]
    fn replay_conversation_forensics_projects_superseded_and_final_records() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let turn_id = session.allocate_turn_id();
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            recorder,
            Vec::new(),
            session.clone(),
            turn_id.clone(),
            ConversationThreadRef::Mainline,
        ));

        let first_artifact = ForensicTraceSink::record_forensic_artifact(
            trace.as_ref(),
            ForensicArtifactCapture {
                exchange_id: "exchange-1".to_string(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::AssembledContext,
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                parent_artifact_id: None,
                summary: "planner prompt".to_string(),
                content: "{\"step\":1}".to_string(),
                mime_type: "application/json".to_string(),
                labels: Default::default(),
            },
        )
        .expect("first artifact");

        ForensicTraceSink::record_forensic_artifact(
            trace.as_ref(),
            ForensicArtifactCapture {
                exchange_id: "exchange-2".to_string(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::AssembledContext,
                provider: "openai".to_string(),
                model: "gpt-5.4".to_string(),
                parent_artifact_id: Some(first_artifact),
                summary: "planner prompt refined".to_string(),
                content: "{\"step\":2}".to_string(),
                mime_type: "application/json".to_string(),
                labels: Default::default(),
            },
        )
        .expect("second artifact");
        trace.emit(TurnEvent::SynthesisReady {
            grounded: true,
            citations: Vec::new(),
            insufficient_evidence: false,
        });
        trace.record_completion(&AuthoredResponse::from_plain_text(
            ResponseMode::GroundedAnswer,
            "final answer",
        ));

        let projection = service
            .replay_conversation_forensics(&session.task_id())
            .expect("forensic replay");

        assert_eq!(projection.task_id, session.task_id());
        assert_eq!(projection.turns.len(), 1);
        assert_eq!(projection.turns[0].turn_id, turn_id);
        assert_eq!(projection.turns[0].lifecycle, ForensicLifecycle::Final);
        assert!(
            projection.turns[0]
                .records
                .iter()
                .any(|record| record.lifecycle == ForensicLifecycle::Superseded)
        );
        assert!(
            projection.turns[0]
                .records
                .iter()
                .any(|record| record.lifecycle == ForensicLifecycle::Final)
        );
    }

    #[test]
    fn process_prompt_emits_forensic_updates_for_recorded_trace_artifacts() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nEmit forensic updates whenever transit records change.\n",
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
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: None,
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
                "Forensic update response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let updates = Arc::new(RecordingForensicUpdateSink::default());
        service.register_forensic_observer(updates.clone());
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
                    "Record forensic updates",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let recorded = updates.recorded();
        assert!(!recorded.is_empty());
        assert!(
            recorded
                .iter()
                .all(|update| update.task_id == session.task_id())
        );
        assert!(recorded.iter().all(|update| {
            update
                .turn_id
                .as_str()
                .starts_with(session.task_id().as_str())
        }));
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
                        retrievers: Vec::new(),
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
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
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
    fn action_bias_targets_code_files_before_docs() {
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

        let ranked = super::likely_action_bias_targets(&loop_state, Path::new("/workspace"), 3);

        assert_eq!(
            ranked,
            vec![
                "src/application/mod.rs".to_string(),
                "README.md".to_string()
            ]
        );
    }

    #[test]
    fn action_bias_targets_ignore_non_authored_workspace_paths() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            evidence_items: vec![
                EvidenceItem {
                    source:
                        "apps/docs/node_modules/playwright-core/lib/server/utils/image_tools/compare.js"
                            .to_string(),
                    snippet: "vendored compare implementation".to_string(),
                    rationale: "dependency".to_string(),
                    rank: 1,
                },
                EvidenceItem {
                    source: "apps/web/dist/assets/index.js".to_string(),
                    snippet: "compiled bundle".to_string(),
                    rationale: "generated asset".to_string(),
                    rank: 2,
                },
                EvidenceItem {
                    source: "apps/web/src/runtime-app.tsx".to_string(),
                    snippet: "authored runtime app".to_string(),
                    rationale: "real edit target".to_string(),
                    rank: 3,
                },
            ],
            ..Default::default()
        };

        let ranked = super::likely_action_bias_targets(&loop_state, Path::new("/workspace"), 3);

        assert_eq!(ranked, vec!["apps/web/src/runtime-app.tsx".to_string()]);
    }

    #[test]
    fn action_bias_targets_ignore_gitignored_workspace_paths() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        let loop_state = crate::domain::ports::PlannerLoopState {
            evidence_items: vec![
                EvidenceItem {
                    source: "apps/docs/.docusaurus/client-modules.js".to_string(),
                    snippet: "generated docs module".to_string(),
                    rationale: "generated asset".to_string(),
                    rank: 1,
                },
                EvidenceItem {
                    source: "apps/web/src/runtime-app.tsx".to_string(),
                    snippet: "authored runtime app".to_string(),
                    rationale: "real edit target".to_string(),
                    rank: 2,
                },
            ],
            ..Default::default()
        };

        let ranked = super::likely_action_bias_targets(&loop_state, workspace.path(), 3);

        assert_eq!(ranked, vec!["apps/web/src/runtime-app.tsx".to_string()]);
    }

    #[test]
    fn action_bias_redirects_non_file_actions_before_any_search_step() {
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
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction {
                known_edit: true,
                candidate_files: vec!["src/application/mod.rs".to_string()],
                resolution: None,
            }),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Execution
                && note.note.contains("Steering review [action-bias]")
                && note.note.contains("src/application/mod.rs")
        }));
    }

    #[test]
    fn action_bias_rejects_repeated_list_files_in_edit_turn() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::ListFiles {
                        pattern: Some("*openai*".to_string()),
                    },
                },
                outcome: "broad file discovery".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "src/infrastructure/providers.rs".to_string(),
                snippet: "provider model registry".to_string(),
                rationale: "known edit candidate".to_string(),
                rank: 1,
            }],
            ..Default::default()
        };
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::ListFiles {
                    pattern: Some("*openai*".to_string()),
                },
            },
            rationale: "try another path match".to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction {
                known_edit: true,
                candidate_files: vec!["src/infrastructure/providers.rs".to_string()],
                resolution: None,
            }),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Execution
                && note.note.contains("Steering review [action-bias]")
                && note.note.contains("Likely target files:")
                && note.note.contains("src/infrastructure/providers.rs")
        }));
    }

    #[test]
    fn action_bias_escalates_to_exact_diff_after_target_file_has_been_read() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "apps/web/src/runtime-shell.css".to_string(),
                    },
                },
                outcome: "read the likely css target".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "apps/web/src/runtime-shell.css".to_string(),
                snippet: ".runtime-shell-host { padding: 8px; }".to_string(),
                rationale: "likely css target".to_string(),
                rank: 1,
            }],
            ..Default::default()
        };
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Read {
                    path: "apps/web/src/runtime-shell.css".to_string(),
                },
            },
            rationale: "read the CSS file again before editing".to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction {
                known_edit: true,
                candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                resolution: None,
            }),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        );

        let note = notes
            .iter()
            .find(|note| note.kind == super::SteeringReviewKind::Execution)
            .expect("action bias note should be present");
        assert!(note.note.contains("Workspace editor pressure: active."));
        assert!(note.note.contains("exact-diff state space"));
        assert!(note.note.contains("replace_in_file"));
        assert!(note.note.contains("apply_patch"));
        assert!(note.note.contains("apps/web/src/runtime-shell.css"));
        assert!(note.note.contains("already been read 1 time"));
    }

    #[test]
    fn action_bias_reads_best_candidate_after_initial_search() {
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
            InitialActionDecision {
                action: InitialAction::Workspace {
                    action: WorkspaceAction::Search {
                        query: "planner loop edit target".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                        retrievers: Vec::new(),
                        intent: Some("implementation search".to_string()),
                    },
                },
                rationale: "start with one bounded search".to_string(),
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["src/application/mod.rs".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Search {
                            query: "keep searching broadly".to_string(),
                            mode: RetrievalMode::Linear,
                            strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                            retrievers: Vec::new(),
                            intent: Some("implementation search".to_string()),
                        },
                    },
                    rationale: "continue exploring".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "src/application/mod.rs".to_string(),
                        },
                    },
                    rationale: "read the likely target file before more retrieval".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
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
                        snippet: "planner loop handles action bias".to_string(),
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
                .process_prompt("Fix the action-bias behavior")
                .await
                .expect("process prompt")
        });

        let gatherer_requests = gatherer_requests.lock().expect("gatherer requests");
        assert_eq!(gatherer_requests.len(), 1);
        let planner_requests = request_log.lock().expect("planner request log");
        let review_request = planner_requests
            .iter()
            .find(|request| {
                request
                    .loop_state
                    .notes
                    .iter()
                    .any(|note| note.contains("Steering review [action-bias]"))
            })
            .expect("action bias review request should be recorded");
        assert!(review_request.loop_state.notes.iter().any(|note| {
            note.contains("Likely target files") && note.contains("src/application/mod.rs")
        }));
        assert!(review_request.loop_state.notes.iter().any(|note| {
            note.contains("exact-diff state space")
                && note.contains("replace_in_file")
                && note.contains("apply_patch")
        }));
        assert!(review_request.loop_state.notes.iter().any(|note| {
            note.contains("Workspace editor pressure: active.") && note.contains("workspace editor")
        }));

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
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec![
                        "README.md".to_string(),
                        "src/application/mod.rs".to_string(),
                    ],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
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
    fn known_edit_bootstrap_discards_node_modules_hints_and_prefers_authored_files() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(
            workspace
                .path()
                .join("apps/docs/node_modules/playwright-core/lib/server/utils/image_tools"),
        )
        .expect("create vendored compare dir");
        fs::create_dir_all(workspace.path().join("src/image_tools"))
            .expect("create authored compare dir");
        fs::write(
            workspace.path().join(
                "apps/docs/node_modules/playwright-core/lib/server/utils/image_tools/compare.js",
            ),
            "export function compare() { return 'vendored'; }\n",
        )
        .expect("write vendored compare");
        fs::write(
            workspace.path().join("src/image_tools/compare.rs"),
            "pub fn compare() {}\n",
        )
        .expect("write authored compare");

        let candidates = super::known_edit_bootstrap_candidates(
            workspace.path(),
            &[
                "apps/docs/node_modules/playwright-core/lib/server/utils/image_tools/compare.js"
                    .to_string(),
            ],
            "Fix the compare image tool behavior",
            3,
        );

        assert_eq!(candidates, vec!["src/image_tools/compare.rs".to_string()]);
    }

    #[test]
    fn known_edit_bootstrap_discards_gitignored_generated_artifacts() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated docs dir");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create authored app dir");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        fs::write(
            workspace
                .path()
                .join("apps/docs/.docusaurus/client-modules.js"),
            "export default [];\n",
        )
        .expect("write generated docs module");
        fs::write(
            workspace.path().join("apps/web/src/runtime-app.tsx"),
            "export function RuntimeApp() { return null; }\n",
        )
        .expect("write authored runtime app");

        let candidates = super::known_edit_bootstrap_candidates(
            workspace.path(),
            &["apps/docs/.docusaurus/client-modules.js".to_string()],
            "Fix the runtime app shell behavior",
            3,
        );

        assert_eq!(candidates, vec!["apps/web/src/runtime-app.tsx".to_string()]);
    }

    #[test]
    fn controller_infers_known_edit_from_prompt_without_provider_signal() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create css dir");
        fs::write(
            workspace.path().join("apps/web/src/runtime-shell.css"),
            ".runtime-shell-host {\n  padding: 0;\n}\n",
        )
        .expect("write css");

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
            initial_action_decision(InitialAction::Answer, "provider omitted the edit envelope"),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
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
                .process_prompt(
                    "The .runtime-shell-host div needs some padding. Something around 8px",
                )
                .await
                .expect("process prompt")
        });

        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions lock");
        assert!(matches!(
            executed_actions.first(),
            Some(WorkspaceAction::Read { path }) if path == "apps/web/src/runtime-shell.css"
        ));
    }

    #[test]
    fn workspace_editor_boundary_budget_signal_credits_boundary_source() {
        let (_, _, contributions) =
            budget_signal_details("workspace-editor-boundary:planner-budget-exhausted");
        assert!(
            contributions
                .iter()
                .any(|item| item.source == "workspace_editor_boundary")
        );
        assert!(
            contributions
                .iter()
                .any(|item| item.source == "planner_budget")
        );
    }

    #[test]
    fn known_edit_planner_budget_preserves_workspace_editor_headroom() {
        let initial_edit = InitialEditInstruction {
            known_edit: true,
            candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
            resolution: None,
        };
        let budget = super::planner_budget_for_turn(
            super::instruction_frame_from_initial_edit(&initial_edit).as_ref(),
            &initial_edit,
        );

        assert_eq!(budget.max_steps, 10);
        assert_eq!(budget.max_reads, 4);
        assert_eq!(budget.max_inspects, 3);
        assert_eq!(budget.max_searches, 2);
        assert_eq!(budget.max_replans, 1);
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
                        retrievers: Vec::new(),
                        intent: Some("implementation search".to_string()),
                    },
                },
                rationale: "controller should still choose the file".to_string(),
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["src/one.rs".to_string(), "src/two.rs".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
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

    #[tokio::test]
    async fn known_edit_bootstrap_uses_deterministic_resolution() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src/components"))
            .expect("create components");
        fs::write(
            workspace
                .path()
                .join("apps/web/src/components/ManifoldVisualization.tsx"),
            "export function ManifoldVisualization() { return null; }\n",
        )
        .expect("write component");
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let mut service = test_service(workspace.path());
        service.entity_resolver = Arc::new(StaticEntityResolver {
            outcome: EntityResolutionOutcome::Resolved {
                target: EntityResolutionCandidate::new(
                    "apps/web/src/components/ManifoldVisualization.tsx",
                    EntityLookupMode::ExactPath,
                    1,
                ),
                alternatives: Vec::new(),
                explanation: "single authored target".to_string(),
            },
            recorded_requests: Arc::clone(&recorded_requests),
        });
        let decision = InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Search {
                    query: "find the manifold component".to_string(),
                    mode: RetrievalMode::Linear,
                    strategy: RetrievalStrategy::Lexical,
                    retrievers: Vec::new(),
                    intent: Some("implementation search".to_string()),
                },
            },
            rationale: "search first".to_string(),
            answer: None,
            edit: InitialEditInstruction {
                known_edit: true,
                candidate_files: vec![
                    "apps/web/src/components/ManifoldVisualization.tsx".to_string(),
                ],
                resolution: None,
            },
            grounding: None,
        };

        let bootstrapped = service
            .bootstrap_known_edit_initial_action(
                "Tighten the ManifoldVisualization copy in the web UI.",
                &InterpretationContext::default(),
                &[],
                None,
                &decision,
                &StructuredTurnTrace::new(
                    Arc::new(RecordingTurnEventSink::default()),
                    Arc::new(InMemoryTraceRecorder::default()),
                    Vec::new(),
                    ConversationSession::new(
                        TaskTraceId::new("task-bootstrap").expect("task trace id"),
                    ),
                    crate::domain::model::TurnTraceId::new("turn-bootstrap")
                        .expect("turn trace id"),
                    ConversationThreadRef::Mainline,
                ),
            )
            .await
            .expect("bootstrap should succeed")
            .expect("known edit should bootstrap");

        assert!(matches!(
            bootstrapped.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Read { ref path }
            } if path == "apps/web/src/components/ManifoldVisualization.tsx"
        ));
        assert_eq!(
            bootstrapped
                .edit
                .resolution
                .as_ref()
                .and_then(EntityResolutionOutcome::resolved_path),
            Some("apps/web/src/components/ManifoldVisualization.tsx")
        );
        assert_eq!(
            recorded_requests
                .lock()
                .expect("resolver requests lock")
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn execution_pressure_prefers_resolved_targets_over_repeated_search() {
        let resolution = EntityResolutionOutcome::Resolved {
            target: EntityResolutionCandidate::new(
                "src/application/mod.rs",
                EntityLookupMode::ExactPath,
                1,
            ),
            alternatives: Vec::new(),
            explanation: "deterministic bootstrap already resolved the authored target".to_string(),
        };
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(InitialAction::Answer, "unused"),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "src/application/mod.rs".to_string(),
                    },
                },
                rationale: "move into the resolved file".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let mut context = test_planner_loop_context(InitialEditInstruction {
            known_edit: true,
            candidate_files: vec!["src/application/mod.rs".to_string()],
            resolution: Some(resolution.clone()),
        });
        context.planner_engine = planner;

        let reviewed = super::review_decision_under_signals(
            "Fix the planner stream text in src/application/mod.rs",
            &context,
            &PlannerBudget::default(),
            &PlannerLoopState {
                evidence_items: vec![EvidenceItem {
                    source: "src/application/mod.rs".to_string(),
                    snippet: "fn planner_loop() {}".to_string(),
                    rationale: "known authored target".to_string(),
                    rank: 1,
                }],
                ..Default::default()
            },
            RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "cargo test".to_string(),
                    },
                },
                rationale: "search a bit more".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Path::new("/workspace"),
            Arc::new(StructuredTurnTrace::new(
                Arc::new(RecordingTurnEventSink::default()),
                Arc::new(InMemoryTraceRecorder::default()),
                Vec::new(),
                ConversationSession::new(TaskTraceId::new("task-review").expect("task trace id")),
                crate::domain::model::TurnTraceId::new("turn").expect("turn id"),
                ConversationThreadRef::Mainline,
            )),
        )
        .await
        .expect("steering review should succeed");

        assert!(matches!(
            reviewed.action,
            PlannerAction::Workspace {
                action: WorkspaceAction::Read { ref path }
            } if path == "src/application/mod.rs"
        ));
        assert_eq!(
            recorded_requests
                .lock()
                .expect("recorded requests lock")
                .last()
                .expect("review planner request")
                .loop_state
                .target_resolution,
            Some(resolution)
        );
    }

    #[test]
    fn unresolved_targets_fail_closed_before_workspace_mutation() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src/application")).expect("create app dir");
        fs::write(workspace.path().join("src/application/mod.rs"), "before\n").expect("write file");

        let service = test_service(workspace.path());
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let sink = Arc::new(RecordingTurnEventSink::default());
        let session = ConversationSession::new(TaskTraceId::new("task-ambiguous").expect("task"));
        let turn_id = session.allocate_turn_id();
        let trace = Arc::new(StructuredTurnTrace::new(
            sink.clone(),
            Arc::new(InMemoryTraceRecorder::default()),
            Vec::new(),
            session.clone(),
            turn_id,
            session.active_thread().thread_ref,
        ));
        let resolution = EntityResolutionOutcome::Ambiguous {
            candidates: vec![
                EntityResolutionCandidate::new(
                    "src/application/mod.rs",
                    EntityLookupMode::Basename,
                    1,
                ),
                EntityResolutionCandidate::new(
                    "src/domain/model/turns.rs",
                    EntityLookupMode::Basename,
                    2,
                ),
            ],
            explanation: "two authored files remained tied".to_string(),
        };
        let mut context = test_planner_loop_context(InitialEditInstruction {
            known_edit: true,
            candidate_files: vec!["src/application/mod.rs".to_string()],
            resolution: Some(resolution.clone()),
        });
        context.synthesizer_engine = synthesizer.clone();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            service
                .execute_recursive_planner_loop(
                    "Update the planner stream text",
                    context,
                    Some(RecursivePlannerDecision {
                        action: PlannerAction::Workspace {
                            action: WorkspaceAction::ReplaceInFile {
                                path: "src/application/mod.rs".to_string(),
                                old: "before".to_string(),
                                new: "after".to_string(),
                                replace_all: false,
                            },
                        },
                        rationale: "edit immediately".to_string(),
                        answer: None,
                        edit: InitialEditInstruction::default(),
                        grounding: None,
                    }),
                    None,
                    trace,
                )
                .await
                .expect("planner loop should succeed");
        });

        assert!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .is_empty()
        );
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::Fallback { stage, reason }
                if stage == "entity-resolution" && reason.contains("ambiguous")
        )));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::PlannerSummary {
                stop_reason: Some(reason),
                ..
            } if reason.contains("unresolved-entity-target:ambiguous")
        )));
    }

    #[tokio::test]
    async fn resolver_never_promotes_non_authored_targets() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src")).expect("create src");
        fs::create_dir_all(workspace.path().join("dist")).expect("create dist");
        fs::write(workspace.path().join(".gitignore"), "/dist/\n").expect("write gitignore");
        fs::write(workspace.path().join("src/runtime.rs"), "fn runtime() {}\n").expect("write src");
        fs::write(workspace.path().join("dist/runtime.js"), "export {};\n").expect("write dist");

        let resolver: Arc<dyn EntityResolver> = Arc::new(StaticEntityResolver {
            outcome: EntityResolutionOutcome::Ambiguous {
                candidates: vec![
                    EntityResolutionCandidate::new(
                        "dist/runtime.js",
                        EntityLookupMode::ExactPath,
                        1,
                    ),
                    EntityResolutionCandidate::new(
                        "src/runtime.rs",
                        EntityLookupMode::ExactPath,
                        2,
                    ),
                ],
                explanation: "unsafe candidate leaked into the resolver output".to_string(),
            },
            recorded_requests: Arc::new(Mutex::new(Vec::new())),
        });

        let outcome = super::resolve_known_edit_target(
            &resolver,
            workspace.path(),
            "Update runtime",
            &["dist/runtime.js".to_string(), "src/runtime.rs".to_string()],
            &[],
            None,
        )
        .await
        .expect("resolution outcome");

        assert_eq!(outcome.resolved_path(), Some("src/runtime.rs"));
        assert!(
            !outcome
                .candidate_paths()
                .iter()
                .any(|path| path == "dist/runtime.js")
        );
    }

    #[test]
    fn inspect_actions_emit_tool_execution_events() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "cat README.md".to_string(),
                    },
                },
                "inspect the local file before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "inspection was enough".to_string(),
                },
                rationale: "stop after the inspect".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink("inspect the workspace", sink.clone())
                .await
                .expect("process prompt")
        });

        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolCalled { tool_name, invocation, .. }
                if tool_name == "inspect" && invocation == "cat README.md"
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolOutput {
                tool_name,
                stream,
                output,
                ..
            } if tool_name == "inspect"
                && stream == "stdout"
                && output.contains("# Workspace")
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolFinished { tool_name, summary, .. }
                if tool_name == "inspect" && summary == "inspection completed"
        )));
    }

    #[test]
    fn shell_actions_stream_terminal_output_before_completion() {
        let workspace = tempfile::tempdir().expect("workspace");

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
                InitialAction::Workspace {
                    action: WorkspaceAction::Shell {
                        command: "printf 'alpha\\n'; printf 'warning\\n' >&2".to_string(),
                    },
                },
                "stream the terminal output before stopping",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "inspection was enough".to_string(),
                },
                rationale: "stop after the inspect".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink("run the shell command", sink.clone())
                .await
                .expect("process prompt")
        });

        let events = sink.recorded();
        let stdout_index = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    TurnEvent::ToolOutput {
                        tool_name,
                        stream,
                        output,
                        ..
                    } if tool_name == "shell"
                        && stream == "stdout"
                        && output.contains("alpha")
                )
            })
            .expect("stdout terminal output");
        let stderr_index = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    TurnEvent::ToolOutput {
                        tool_name,
                        stream,
                        output,
                        ..
                    } if tool_name == "shell"
                        && stream == "stderr"
                        && output.contains("warning")
                )
            })
            .expect("stderr terminal output");
        let finished_index = events
            .iter()
            .position(|event| {
                matches!(
                    event,
                    TurnEvent::ToolFinished { tool_name, .. } if tool_name == "shell"
                )
            })
            .expect("shell completion");
        let finished_summary = events
            .iter()
            .find_map(|event| match event {
                TurnEvent::ToolFinished {
                    tool_name, summary, ..
                } if tool_name == "shell" => Some(summary.as_str()),
                _ => None,
            })
            .expect("shell completion summary");

        assert!(stdout_index < finished_index);
        assert!(stderr_index < finished_index);
        assert_eq!(finished_summary, "command completed");
    }

    #[test]
    fn workspace_editor_edits_emit_applied_edit_events() {
        let workspace = tempfile::tempdir().expect("workspace");

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
                InitialAction::Workspace {
                    action: WorkspaceAction::WriteFile {
                        path: "note.txt".to_string(),
                        content: "hello\n".to_string(),
                    },
                },
                "apply the requested edit",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "edit complete".to_string(),
                },
                rationale: "stop after the edit".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink("write the local file", sink.clone())
                .await
                .expect("process prompt")
        });

        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::WorkspaceEditApplied { tool_name, edit, .. }
                if tool_name == "write_file"
                    && edit.files == vec!["note.txt".to_string()]
                    && edit.diff.contains("+++ b/note.txt")
        )));
        assert!(!events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolFinished { tool_name, .. } if tool_name == "write_file"
        )));
    }

    #[test]
    fn planner_stop_answers_render_directly_without_synthesizer_rewrite() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
        let answer = "I’m happy to help you troubleshoot your 1968 Chevy C20. Start with battery, fuel, spark, and starter checks.".to_string();
        let rationale = "the user asked for general troubleshooting advice, so the loop can end without synthesis"
            .to_string();
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status --short".to_string(),
                    },
                },
                "inspect the local workspace before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "model selected answer".to_string(),
                },
                rationale: rationale.clone(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("help me troubleshoot my 1968 Chevy C20")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        assert_ne!(reply, rationale);
        assert!(
            synthesizer
                .gathered_summaries
                .lock()
                .expect("gathered summaries lock")
                .is_empty(),
            "planner-authored stop answers should not be rewritten by the synthesizer"
        );
    }

    #[test]
    fn initial_refusals_render_policy_violation_without_synthesizer_rewrite() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
        let rationale = "the user requested disallowed automotive repair instructions".to_string();
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Stop {
                    reason: "refusal".to_string(),
                },
                rationale: rationale.clone(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("Where would I find the starter solenoid?")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, POLICY_VIOLATION_DIRECT_REPLY);
        assert_ne!(reply, rationale);
        assert!(
            synthesizer
                .handoffs
                .lock()
                .expect("handoffs lock")
                .is_empty(),
            "planner-authored refusals should bypass the synthesizer"
        );
    }

    #[test]
    fn recursive_refusals_render_policy_violation_without_synthesizer_rewrite() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
        let rationale = "the user requested disallowed automotive repair instructions".to_string();
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status --short".to_string(),
                    },
                },
                "inspect the local workspace before deciding",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "refusal".to_string(),
                },
                rationale: rationale.clone(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("Where would I find the starter solenoid?")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, POLICY_VIOLATION_DIRECT_REPLY);
        assert_ne!(reply, rationale);
        assert!(
            synthesizer
                .handoffs
                .lock()
                .expect("handoffs lock")
                .is_empty(),
            "recursive planner refusals should bypass the synthesizer"
        );
    }

    #[test]
    fn initial_answer_decisions_render_explicit_user_answer_without_synthesizer_rewrite() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
        let answer = "Starter circuit\n\n[ battery ]---(solenoid)---(starter )".to_string();
        let rationale =
            "the prompt is a direct diagram request, so no workspace action is required"
                .to_string();
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: rationale.clone(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt("Can you generate an ASCII diagram of the start circuit?")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        assert_ne!(reply, rationale);
        assert!(
            synthesizer
                .gathered_summaries
                .lock()
                .expect("gathered summaries lock")
                .is_empty(),
            "initial direct answers should bypass the synthesizer"
        );
    }

    #[test]
    fn edit_turns_do_not_complete_with_advice_only_stop_answers() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create css dir");
        fs::write(
            workspace.path().join("apps/web/src/runtime-shell.css"),
            ".runtime-shell-host {\n  padding: 0;\n}\n",
        )
        .expect("write css");

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
                rationale: "the file is obvious, so jump straight to the answer".to_string(),
                answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt(
                    "The .runtime-shell-host class needs some padding. Something around 8px",
                )
                .await
                .expect("process prompt")
        });

        assert!(
            reply.contains("haven't completed the requested repository edit yet"),
            "edit turns should stay open instead of completing with advice-only text: {reply}"
        );
        assert!(
            synthesizer
                .handoffs
                .lock()
                .expect("handoffs lock")
                .is_empty(),
            "unsatisfied edit turns should not fall through to the synthesizer"
        );
    }

    #[test]
    fn edit_turn_stop_answers_replan_into_workspace_editor_actions() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create css dir");
        let css_path = workspace.path().join("apps/web/src/runtime-shell.css");
        fs::write(&css_path, ".runtime-shell-host {\n  padding: 0;\n}\n").expect("write css");

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
        let answer =
            "Added 8px padding to `.runtime-shell-host` after handing the turn to the workspace editor."
                .to_string();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "the file is obvious, so answer directly".to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "apps/web/src/runtime-shell.css".to_string(),
                        },
                    },
                    rationale: "read the CSS file before editing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the action-bias review still returned an advice-only answer"
                        .to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::ReplaceInFile {
                            path: "apps/web/src/runtime-shell.css".to_string(),
                            old: "padding: 0;".to_string(),
                            new: "padding: 8px;".to_string(),
                            replace_all: false,
                        },
                    },
                    rationale: "the replan should now hand the turn to the workspace editor"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt(
                    "Add 8px padding to `.runtime-shell-host` in the runtime shell styles.",
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        assert!(matches!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .last(),
            Some(WorkspaceAction::ReplaceInFile { path, new, .. })
                if path == "apps/web/src/runtime-shell.css" && new == "padding: 8px;"
        ));

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(
            requests.iter().any(|request| request
                .loop_state
                .notes
                .iter()
                .any(|note| note
                    .contains("Replan from current evidence after instruction unsatisfied."))),
            "advice-only planner stops with an open applied-edit obligation should trigger a bounded replan into workspace editing"
        );
    }

    #[test]
    fn commit_turns_review_stop_answers_into_git_commit_actions() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("src/application"))
            .expect("create application dir");
        let app_path = workspace.path().join("src/application/mod.rs");
        fs::write(
            &app_path,
            "pub fn runtime_mode() -> &'static str { \"before\" }\n",
        )
        .expect("write tracked file");

        std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(workspace.path())
            .status()
            .expect("git init");
        std::process::Command::new("git")
            .args(["config", "user.email", "paddles@example.com"])
            .current_dir(workspace.path())
            .status()
            .expect("git config user.email");
        std::process::Command::new("git")
            .args(["config", "user.name", "Paddles Test"])
            .current_dir(workspace.path())
            .status()
            .expect("git config user.name");
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(workspace.path())
            .status()
            .expect("git add");
        std::process::Command::new("git")
            .args(["commit", "-qm", "Initial state"])
            .current_dir(workspace.path())
            .status()
            .expect("initial commit");

        fs::write(
            &app_path,
            "pub fn runtime_mode() -> &'static str { \"after\" }\n",
        )
        .expect("write working change");

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
        let answer =
            "Recorded a git commit for the current workspace changes after reviewing the diff."
                .to_string();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "reply with commit guidance immediately".to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "git diff -- src/application/mod.rs".to_string(),
                        },
                    },
                    rationale: "inspect the changed file before committing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "git diff -- src/application/mod.rs".to_string(),
                        },
                    },
                    rationale:
                        "premise review kept the same diff inspection because the workspace is dirty"
                            .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "advise the operator instead of committing".to_string(),
                    answer: Some(
                        "I need to inspect a little more before I can commit safely."
                            .to_string(),
                    ),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Shell {
                            command: "git commit -am \"Clarify premise challenge containment signals\""
                                .to_string(),
                        },
                    },
                    rationale:
                        "the commit-bias review should hand the turn to git commit now"
                            .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested commit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt("Make a git commit")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        let head_subject = std::process::Command::new("git")
            .args(["log", "-1", "--pretty=%s"])
            .current_dir(workspace.path())
            .output()
            .expect("git log");
        assert_eq!(
            String::from_utf8_lossy(&head_subject.stdout).trim(),
            "Clarify premise challenge containment signals"
        );

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(
            requests.iter().any(|request| request
                .loop_state
                .notes
                .iter()
                .any(|note| note.contains("Steering review [action-bias]")
                    && note.contains("git commit"))),
            "commit turns should steer advice-only stops back toward a git commit action"
        );
    }

    #[test]
    fn recursive_edit_signals_promote_turns_into_edit_obligations() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create css dir");
        fs::write(
            workspace.path().join("apps/web/src/runtime-shell.css"),
            ".runtime-shell-host {\n  padding: 0;\n}\n",
        )
        .expect("write css");

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
                action: InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "apps/web/src/runtime-shell.css".to_string(),
                    },
                },
                rationale: "inspect the likely CSS file before deciding".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested padding edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested padding edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested padding edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested padding edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction {
                        known_edit: true,
                        candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                        resolution: None,
                    },
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt(
                    "The .runtime-shell-host class needs some padding. Something around 8px",
                )
                .await
                .expect("process prompt")
        });

        assert!(
            reply.contains("haven't completed the requested repository edit yet"),
            "recursive edit signals should promote the turn into an applied-edit obligation: {reply}"
        );
        assert!(
            synthesizer
                .handoffs
                .lock()
                .expect("handoffs lock")
                .is_empty(),
            "late-discovered edit turns should still block advice-only completion"
        );
    }

    #[test]
    fn edit_turns_can_complete_after_a_successful_write() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create css dir");
        let css_path = workspace.path().join("apps/web/src/runtime-shell.css");
        fs::write(&css_path, ".runtime-shell-host {\n  padding: 0;\n}\n").expect("write css");

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
        let answer = "Added 8px padding to `.runtime-shell-host`.".to_string();
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "the file is obvious, so jump straight to the answer".to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["apps/web/src/runtime-shell.css".to_string()],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::ReplaceInFile {
                            path: "apps/web/src/runtime-shell.css".to_string(),
                            old: "padding: 0;".to_string(),
                            new: "padding: 8px;".to_string(),
                            replace_all: false,
                        },
                    },
                    rationale: "apply the requested padding change".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt(
                    "The .runtime-shell-host class needs some padding. Something around 8px",
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        assert!(matches!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .last(),
            Some(WorkspaceAction::ReplaceInFile { path, new, .. })
                if path == "apps/web/src/runtime-shell.css" && new == "padding: 8px;"
        ));
    }

    #[test]
    fn known_edit_turns_can_spend_multiple_candidate_reads_before_applying_a_patch() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create web src");
        let css_path = workspace.path().join("apps/web/src/runtime-shell.css");
        fs::write(&css_path, ".runtime-shell-host {\n  padding: 0;\n}\n").expect("write css");
        fs::write(
            workspace.path().join("apps/web/src/runtime-app.tsx"),
            "export function RuntimeApp() { return null; }\n",
        )
        .expect("write runtime app");
        fs::write(
            workspace.path().join("apps/web/src/runtime-store.tsx"),
            "export function useRuntimeStore() { return null; }\n",
        )
        .expect("write runtime store");

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
        let answer =
            "Added 8px padding to `.runtime-shell-host` after checking the related runtime files."
                .to_string();
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "the file is probably obvious, so answer directly".to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec![
                        "apps/web/src/runtime-shell.css".to_string(),
                        "apps/web/src/runtime-app.tsx".to_string(),
                        "apps/web/src/runtime-store.tsx".to_string(),
                    ],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "apps/web/src/runtime-app.tsx".to_string(),
                        },
                    },
                    rationale: "read the app shell before editing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "apps/web/src/runtime-app.tsx".to_string(),
                        },
                    },
                    rationale: "the action-bias review still allows reading the app shell"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "apps/web/src/runtime-store.tsx".to_string(),
                        },
                    },
                    rationale: "read the runtime store before patching".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "apps/web/src/runtime-store.tsx".to_string(),
                        },
                    },
                    rationale: "the action-bias review still allows reading the runtime store"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::ReplaceInFile {
                            path: "apps/web/src/runtime-shell.css".to_string(),
                            old: "padding: 0;".to_string(),
                            new: "padding: 8px;".to_string(),
                            replace_all: false,
                        },
                    },
                    rationale: "apply the requested padding change".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt(
                    "The runtime shell needs 8px padding. Check the neighboring runtime files before you patch it.",
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        let executed_actions = synthesizer
            .executed_actions
            .lock()
            .expect("executed actions lock");
        let read_count = executed_actions
            .iter()
            .filter(|action| matches!(action, WorkspaceAction::Read { .. }))
            .count();
        assert!(
            read_count >= 3,
            "edit turn should retain enough headroom to read multiple candidate files before patching"
        );
        assert!(executed_actions.iter().any(|action| matches!(
            action,
            WorkspaceAction::Read { path } if path == "apps/web/src/runtime-app.tsx"
        )));
        assert!(executed_actions.iter().any(|action| matches!(
            action,
            WorkspaceAction::Read { path } if path == "apps/web/src/runtime-store.tsx"
        )));
        assert!(matches!(
            executed_actions.last(),
            Some(WorkspaceAction::ReplaceInFile { path, new, .. })
                if path == "apps/web/src/runtime-shell.css" && new == "padding: 8px;"
        ));
    }

    #[test]
    fn known_edit_turns_can_replan_after_budget_exhaustion_and_still_apply_a_patch() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create web src");
        fs::write(
            workspace.path().join("apps/web/src/runtime-shell.css"),
            ".runtime-shell-host {\n  padding: 0;\n}\n",
        )
        .expect("write css");
        fs::write(
            workspace.path().join("apps/web/src/runtime-app.tsx"),
            "export function RuntimeApp() { return null; }\n",
        )
        .expect("write runtime app");

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
        let answer = "Replanned after exhausting inspect budget, then applied 8px padding to `.runtime-shell-host`.".to_string();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "the controller should bootstrap the likely file first".to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec![
                        "apps/web/src/runtime-shell.css".to_string(),
                        "apps/web/src/runtime-app.tsx".to_string(),
                    ],
                    resolution: None,
                },
                grounding: None,
            },
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "inspect the workspace root before editing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "the action-bias review still allows one inspect".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "inspect again before editing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "the action-bias review still allows another inspect".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "inspect a third time before editing".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "the action-bias review still allows a third inspect".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "ask for a fourth inspect, which should force replanning"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "pwd".to_string(),
                        },
                    },
                    rationale: "the action-bias review repeats the exhausted inspect".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::ReplaceInFile {
                            path: "apps/web/src/runtime-shell.css".to_string(),
                            old: "padding: 0;".to_string(),
                            new: "padding: 8px;".to_string(),
                            replace_all: false,
                        },
                    },
                    rationale: "replanning should now go straight to the patch".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_with_sink(
                    "Upgrade the runtime shell padding to 8px. Keep going if the first plan runs out of room.",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, answer);
        assert!(matches!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .last(),
            Some(WorkspaceAction::ReplaceInFile { path, new, .. })
                if path == "apps/web/src/runtime-shell.css" && new == "padding: 8px;"
        ));

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(
            requests
                .iter()
                .any(|request| request.budget.max_steps > 10 && request.budget.max_inspects > 3),
            "replanning should expand the known-edit planner budget"
        );
        assert!(
            requests
                .iter()
                .skip(1)
                .flat_map(|request| request.loop_state.notes.iter())
                .any(|note| note.contains("Replan from current evidence")),
            "follow-on planner requests should carry a replanning note after budget exhaustion"
        );

        let plan_updates = sink
            .recorded()
            .into_iter()
            .filter_map(|event| match event {
                TurnEvent::PlanUpdated { items } => Some(items),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert!(
            plan_updates
                .iter()
                .any(|items| items.iter().any(|item| item.id == "replan")),
            "replanning should surface through the shared execution checklist"
        );
    }

    #[test]
    fn out_of_domain_direct_turns_do_not_trigger_controller_workspace_inspects() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
                rationale: "conversational request can go straight to the answer lane".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink(
                    "Can you help me debug an issue starting my Chevy C20 truck. It's not turning over",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert!(!sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::ToolCalled { tool_name, invocation, .. }
                if tool_name == "inspect" && invocation == "git status --short"
        )));
    }

    #[test]
    fn synthesizer_only_turns_receive_recent_turns_and_thread_summary_handoff() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
                rationale: "answer lane should handle this directly".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        let history_store = Arc::new(ConversationHistoryStore::with_path(
            workspace.path().join("state/conversation-history.toml"),
        ));
        service.set_conversation_history_store(Arc::clone(&history_store));
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_sink(
                    "Can you help me debug an issue starting my Chevy C20 truck?",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("first prompt");
            service
                .process_prompt_in_session_with_sink(
                    "Sure you do. You helped me in a prior conversation",
                    session,
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("second prompt");
        });

        let handoffs = synthesizer.handoffs.lock().expect("handoffs lock").clone();
        assert!(
            handoffs.len() >= 2,
            "expected both turns to reach the synthesizer"
        );
        let second = handoffs.last().expect("second handoff");
        assert!(
            second
                .recent_turns
                .iter()
                .any(|turn| turn.contains("Chevy C20 truck")),
            "recent turns should include the active conversational subject"
        );
        assert!(
            second
                .recent_thread_summary
                .as_deref()
                .is_some_and(|summary| summary.contains("Chevy C20 truck")),
            "thread summary should carry the active conversational subject into the answer lane"
        );
    }

    #[test]
    fn synthesizer_only_turns_receive_planner_declared_grounding_handoff() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

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
                rationale: "external evidence should be required before synthesis".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: Some(GroundingRequirement {
                    domain: GroundingDomain::External,
                    reason: Some("need a verified web source before answering".to_string()),
                }),
            },
            Vec::new(),
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
                .process_prompt_with_sink(
                    "Can you give me the website where I can read about that crate?",
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt");
        });

        let handoffs = synthesizer.handoffs.lock().expect("handoffs lock").clone();
        let handoff = handoffs.last().expect("synthesis handoff");
        assert!(
            handoff
                .grounding
                .as_ref()
                .is_some_and(|grounding| grounding.requires_external()),
            "planner-declared external grounding should be preserved into synthesis"
        );
    }

    #[test]
    fn repo_scoped_followups_bootstrap_a_grounding_probe_before_direct_answer() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("ARCHITECTURE.md"),
            "The generative layer lives in src/domain/model/generative.rs.\n",
        )
        .expect("write architecture");
        fs::create_dir_all(workspace.path().join("src/domain/model")).expect("create src dir");
        fs::write(
            workspace.path().join("src/domain/model/generative.rs"),
            "pub enum ResponseMode { GroundedAnswer, DirectAnswer }\n",
        )
        .expect("write generative module");

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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "this follow-up can be answered directly".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "local grounding probe captured the relevant repository evidence"
                        .to_string(),
                },
                rationale: "the local rg probe already found the repository references".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_with_sink(
                    "I think this is a perfect fit for our generative layer",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Applied the bounded action.");
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::Fallback { stage, reason }
                if stage == "grounding-bootstrap"
                    && reason.contains("repo-scoped")
        )));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::ToolCalled { tool_name, invocation, .. }
                if tool_name == "inspect"
                    && invocation.contains("rg -n -i --hidden")
                    && invocation.contains("generative|layer")
        )));
        assert!(
            !synthesizer
                .gathered_summaries
                .lock()
                .expect("gathered summaries lock")
                .is_empty(),
            "the synthesizer should receive repository evidence after the grounding probe"
        );
    }

    #[test]
    fn premise_challenge_stops_redundant_ci_probe_after_non_failing_run_evidence() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "gh run list --limit 10 --json status,conclusion,name,headBranch,workflowName,url"
                            .to_string(),
                    },
                },
                outcome: "inspected recent CI runs".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "command: gh run list --limit 10 --json status,conclusion,name,headBranch,workflowName,url".to_string(),
                snippet: r#"[{"conclusion":"success","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23910509164","workflowName":"CI"},{"conclusion":"cancelled","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23902835068","workflowName":"CI"}]"#.to_string(),
                rationale: "recent CI status listing".to_string(),
                rank: 1,
            }],
            ..Default::default()
        };
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "gh run list --limit 10 --json databaseId,status,conclusion,name,headBranch,workflowName,url"
                        .to_string(),
                },
            },
            rationale: "get the run id for the failing job".to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
                grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Evidence
                && note.note.contains("Steering review [premise-challenge]")
                && note.note.contains("\"conclusion\":\"success\"")
        }));
    }

    #[test]
    fn diagnostic_turn_requests_recursive_evidence_review_before_stopping() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nVerify failures from first principles.\n",
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
        let inspect_command = r#"printf '%s' '[{"conclusion":"success","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23910509164","workflowName":"CI"},{"conclusion":"cancelled","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23902835068","workflowName":"CI"}]'"#.to_string();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: inspect_command.clone(),
                    },
                },
                "inspect the recent CI runs",
            ),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: inspect_command.clone(),
                        },
                    },
                    rationale: "repeat the same status probe".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "reviewed evidence: recent CI runs are success/cancelled"
                            .to_string(),
                    },
                    rationale: "the gathered sources weaken the premise, so stop and judge them"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::clone(&recorded_requests),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink(
                    "CI is failing. Can you debug it on this machine?",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        let review_request = requests
            .last()
            .expect("steering review request should be recorded");
        assert!(review_request.loop_state.notes.iter().any(|note| {
            note.contains("Steering review [premise-challenge]")
                && note.contains("Treat the reported failure as a hypothesis")
        }));
        let events = sink.recorded();
        let inspect_calls = events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    TurnEvent::ToolCalled { tool_name, .. } if tool_name == "inspect"
                )
            })
            .count();
        assert_eq!(inspect_calls, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerStepProgress { step_number, action, .. }
                if *step_number == 2
                    && action.contains("stop (reviewed evidence: recent CI runs are success/cancelled)")
        )));
    }

    #[test]
    fn premise_challenge_stops_redundant_plain_ci_run_list_probe() {
        let loop_state = crate::domain::ports::PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "gh run list --limit 20".to_string(),
                    },
                },
                outcome: "inspected recent CI runs".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "command: gh run list --limit 20".to_string(),
                snippet: "in_progress\t\tTreat reported failures as verifiable hypotheses\tCI\tmain\tpush\t23912848177\t1m45s\t2026-04-02T17:18:06Z\ncompleted\tsuccess\tPersist runtime lane preferences over config\tCI\tmain\tpush\t23910509164\t16m4s\t2026-04-02T16:21:22Z\ncompleted\tcancelled\tAutocomplete TUI provider and model commands\tCI\tmain\tpush\t23902835068\t23m45s\t2026-04-02T13:29:15Z".to_string(),
                rationale: "recent CI table output".to_string(),
                rank: 1,
            }],
            ..Default::default()
        };
        let decision = RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "gh run list --limit 10".to_string(),
                },
            },
            rationale: "check a smaller recent window".to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &loop_state,
            &decision,
            Path::new("/workspace"),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Evidence
                && note.note.contains("Steering review [premise-challenge]")
                && note.note.contains("completed\tsuccess")
        }));
    }

    #[test]
    fn diagnostic_turn_stops_after_sources_weaken_failure_premise() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nVerify failures from first principles.\n",
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
        let inspect_command = r#"printf '%s' '[{"conclusion":"success","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23910509164","workflowName":"CI"},{"conclusion":"cancelled","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23902835068","workflowName":"CI"}]'"#.to_string();
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: inspect_command.clone(),
                    },
                },
                "inspect the recent CI runs",
            ),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: inspect_command.clone(),
                        },
                    },
                    rationale: "repeat the same status probe".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "reviewed evidence: recent CI runs are success/cancelled"
                            .to_string(),
                    },
                    rationale: "the gathered sources weaken the premise, so stop and judge them"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink(
                    "CI is failing. Can you debug it on this machine?",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        let events = sink.recorded();
        let inspect_calls = events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    TurnEvent::ToolCalled { tool_name, .. } if tool_name == "inspect"
                )
            })
            .count();
        assert_eq!(
            inspect_calls, 1,
            "controller should stop after the first decisive source"
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerStepProgress { step_number, action, .. }
                if *step_number == 2
                    && action.contains("stop (reviewed evidence: recent CI runs are success/cancelled)")
        )));
    }

    #[test]
    fn diagnostic_turn_stops_after_plain_ci_listing_shows_no_failure() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nVerify failures from first principles.\n",
        )
        .expect("write AGENTS.md");
        fs::create_dir_all(workspace.path().join("bin")).expect("create bin");
        let gh_path = workspace.path().join("bin/gh");
        fs::write(
            &gh_path,
            "#!/bin/sh\nif [ \"$1\" = \"run\" ] && [ \"$2\" = \"list\" ]; then\ncat <<'EOF'\nin_progress\t\tTreat reported failures as verifiable hypotheses\tCI\tmain\tpush\t23912848177\t1m45s\t2026-04-02T17:18:06Z\ncompleted\tsuccess\tPersist runtime lane preferences over config\tCI\tmain\tpush\t23910509164\t16m4s\t2026-04-02T16:21:22Z\ncompleted\tcancelled\tAutocomplete TUI provider and model commands\tCI\tmain\tpush\t23902835068\t23m45s\t2026-04-02T13:29:15Z\nEOF\nelse\n  echo \"unexpected gh invocation: $*\" >&2\n  exit 1\nfi\n",
        )
        .expect("write fake gh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&gh_path)
                .expect("fake gh metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&gh_path, permissions).expect("chmod fake gh");
        }

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
        let first_command = "PATH=\"$PWD/bin:$PATH\" gh run list --limit 20".to_string();
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: first_command.clone(),
                    },
                },
                "inspect the recent CI runs",
            ),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "gh run list --limit 10".to_string(),
                        },
                    },
                    rationale: "narrow the list size".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "reviewed evidence: current CI listing does not confirm a failure"
                            .to_string(),
                    },
                    rationale: "the gathered sources weaken the premise, so stop and judge them"
                        .to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
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
                .process_prompt_with_sink(
                    "CI is failing. Can you debug it on this machine?",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        let events = sink.recorded();
        let inspect_calls = events
            .iter()
            .filter(|event| {
                matches!(
                    event,
                    TurnEvent::ToolCalled { tool_name, .. } if tool_name == "inspect"
                )
            })
            .count();
        assert_eq!(
            inspect_calls, 1,
            "controller should stop after the plain CI listing already weakens the failure premise"
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerStepProgress { step_number, action, .. }
                if *step_number == 2
                    && action.contains("stop (reviewed evidence: current CI listing does not confirm a failure)")
        )));
    }

    #[test]
    fn finalize_turn_response_records_shared_completion_side_effects() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let history_store = Arc::new(ConversationHistoryStore::with_path(
            workspace.path().join("state/conversation-history.toml"),
        ));
        service.set_conversation_history_store(Arc::clone(&history_store));
        let transcript_updates = Arc::new(RecordingTranscriptUpdateSink::default());
        service.register_transcript_observer(transcript_updates.clone());

        let session = service.shared_conversation_session();
        let active_thread = session.active_thread().thread_ref;
        let trace = StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            Arc::new(InMemoryTraceRecorder::default()),
            Vec::new(),
            session.clone(),
            session.allocate_turn_id(),
            active_thread.clone(),
        );
        let response = AuthoredResponse::from_plain_text(
            ResponseMode::DirectAnswer,
            "Shared completion helper reply.",
        );

        let reply = service.finalize_turn_response(
            &trace,
            &session,
            &active_thread,
            "What happened?",
            &response,
        );

        assert_eq!(reply, "Shared completion helper reply.");
        assert_eq!(transcript_updates.recorded().len(), 1);
        let thread_summary = session
            .recent_thread_summary(&active_thread)
            .expect("thread summary should be recorded");
        assert!(thread_summary.contains("What happened?"));
        assert!(thread_summary.contains("Shared completion helper reply."));
        let recent_turns = history_store
            .recent_turn_summaries()
            .expect("recent turn summaries");
        assert_eq!(
            recent_turns,
            vec!["Q: What happened? A: Shared completion helper reply.".to_string()]
        );
    }

    #[test]
    fn planner_requests_include_runtime_notes_when_retrieval_is_still_warming() {
        struct WarmingGatherer;

        #[async_trait]
        impl ContextGatherer for WarmingGatherer {
            fn capability(&self) -> crate::domain::ports::GathererCapability {
                crate::domain::ports::GathererCapability::Warming {
                    reason: "background lexical bootstrap is still running".to_string(),
                }
            }

            fn capability_for_planning(
                &self,
                planning: &crate::domain::ports::PlannerConfig,
            ) -> crate::domain::ports::GathererCapability {
                match planning.retrieval_strategy {
                    RetrievalStrategy::Lexical => {
                        crate::domain::ports::GathererCapability::Warming {
                            reason: "bm25 warmup in progress".to_string(),
                        }
                    }
                    RetrievalStrategy::Vector => {
                        crate::domain::ports::GathererCapability::Warming {
                            reason: "vector warmup in progress".to_string(),
                        }
                    }
                }
            }

            async fn gather_context(
                &self,
                _request: &ContextGatherRequest,
            ) -> Result<ContextGatherResult> {
                panic!("warming gatherer should not be invoked")
            }
        }

        let workspace = tempfile::tempdir().expect("workspace");
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "this can be answered directly".to_string(),
                answer: Some("direct answer".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::clone(&recorded_requests),
        ));
        let service = test_service(workspace.path());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
                gatherer: Some(Arc::new(WarmingGatherer) as Arc<dyn ContextGatherer>),
            });
            service
                .process_prompt("Should we search the workspace yet?")
                .await
                .expect("process prompt");
        });

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        let request = requests.first().expect("initial planner request");
        assert!(request.runtime_notes.iter().any(|note| {
            note.contains("Workspace retrieval readiness")
                && note.contains("bm25=warming")
                && note.contains("vector=warming")
        }));
    }

    #[test]
    fn execute_planner_gather_step_refuses_warming_retrieval_without_invoking_gatherer() {
        #[derive(Default)]
        struct StrategyCapabilityGatherer {
            recorded_requests: Arc<Mutex<Vec<ContextGatherRequest>>>,
        }

        #[async_trait]
        impl ContextGatherer for StrategyCapabilityGatherer {
            fn capability(&self) -> crate::domain::ports::GathererCapability {
                crate::domain::ports::GathererCapability::Warming {
                    reason: "boot warmup in progress".to_string(),
                }
            }

            fn capability_for_planning(
                &self,
                planning: &crate::domain::ports::PlannerConfig,
            ) -> crate::domain::ports::GathererCapability {
                match planning.retrieval_strategy {
                    RetrievalStrategy::Lexical => {
                        crate::domain::ports::GathererCapability::Available
                    }
                    RetrievalStrategy::Vector => {
                        crate::domain::ports::GathererCapability::Warming {
                            reason: "vector warmup in progress".to_string(),
                        }
                    }
                }
            }

            async fn gather_context(
                &self,
                request: &ContextGatherRequest,
            ) -> Result<ContextGatherResult> {
                self.recorded_requests
                    .lock()
                    .expect("gatherer requests lock")
                    .push(request.clone());
                Ok(ContextGatherResult::unsupported(
                    "should not run while warming",
                ))
            }
        }

        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());
        let gatherer = Arc::new(StrategyCapabilityGatherer::default());
        let mut context = test_planner_loop_context(InitialEditInstruction::default());
        context.gatherer = Some(gatherer.clone() as Arc<dyn ContextGatherer>);
        let session = service.shared_conversation_session();
        let sink = Arc::new(RecordingTurnEventSink::default());
        let trace = Arc::new(StructuredTurnTrace::new(
            sink.clone(),
            Arc::new(InMemoryTraceRecorder::default()),
            Vec::new(),
            session.clone(),
            session.allocate_turn_id(),
            session.active_thread().thread_ref,
        ));
        let mut loop_state = PlannerLoopState::default();
        let mut used_workspace_resources = false;

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let summary = runtime.block_on(async {
            service
                .execute_planner_gather_step(
                    &context,
                    &mut loop_state,
                    trace,
                    "sift-direct",
                    super::PlannerGatherSpec {
                        query: "semantic target".to_string(),
                        intent_reason: "planner-search".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: RetrievalStrategy::Vector,
                        retrievers: Vec::new(),
                        max_evidence_items: 8,
                        success_summary_override: None,
                        no_bundle_message: "planner search returned no evidence bundle",
                        failure_label: "planner search failed",
                        unavailable_label: "planner search backend unavailable",
                        missing_backend_message:
                            "no gatherer backend is configured for planner search",
                    },
                    &mut used_workspace_resources,
                )
                .await
        });

        assert_eq!(
            summary,
            "planner search backend unavailable: vector warmup in progress"
        );
        assert!(
            gatherer
                .recorded_requests
                .lock()
                .expect("gatherer requests lock")
                .is_empty(),
            "cold vector retrieval should be refused before gather_context runs"
        );
        assert!(!used_workspace_resources);
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::GathererCapability { capability, .. }
                if capability.contains("warming: vector warmup in progress")
        )));
    }

    #[test]
    fn execute_planner_gather_step_reuses_request_and_merge_path() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let gatherer = Arc::new(RecordingGatherer {
            recorded_requests: Arc::clone(&requests),
            bundle: EvidenceBundle::new(
                "Found the likely implementation file.".to_string(),
                vec![EvidenceItem {
                    source: "src/application/mod.rs".to_string(),
                    snippet: "planner gather path".to_string(),
                    rationale: "primary hit".to_string(),
                    rank: 1,
                }],
            ),
        });
        let mut context = test_planner_loop_context(InitialEditInstruction::default());
        context.gatherer = Some(gatherer);
        context.interpretation = InterpretationContext {
            summary: "Search the implementation path.".to_string(),
            ..Default::default()
        };
        context.recent_turns = vec!["Previous turn context".to_string()];
        let session = service.shared_conversation_session();
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            Arc::new(InMemoryTraceRecorder::default()),
            Vec::new(),
            session.clone(),
            session.allocate_turn_id(),
            session.active_thread().thread_ref,
        ));
        let mut loop_state = PlannerLoopState::default();
        let mut used_workspace_resources = false;

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let search_summary = runtime.block_on(async {
            service
                .execute_planner_gather_step(
                    &context,
                    &mut loop_state,
                    trace.clone(),
                    "sift-direct",
                    super::PlannerGatherSpec {
                        query: "runtime shell host".to_string(),
                        intent_reason: "planner-search".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: RetrievalStrategy::Lexical,
                        retrievers: vec![RetrieverOption::PathFuzzy],
                        max_evidence_items: 8,
                        success_summary_override: None,
                        no_bundle_message: "planner search returned no evidence bundle",
                        failure_label: "planner search failed",
                        unavailable_label: "planner search backend unavailable",
                        missing_backend_message:
                            "no gatherer backend is configured for planner search",
                    },
                    &mut used_workspace_resources,
                )
                .await
        });
        let refine_summary = runtime.block_on(async {
            service
                .execute_planner_gather_step(
                    &context,
                    &mut loop_state,
                    trace,
                    "sift-direct",
                    super::PlannerGatherSpec {
                        query: "runtime shell host".to_string(),
                        intent_reason: "planner-refine".to_string(),
                        mode: RetrievalMode::Graph,
                        strategy: RetrievalStrategy::Vector,
                        retrievers: vec![
                            RetrieverOption::PathFuzzy,
                            RetrieverOption::SegmentFuzzy,
                        ],
                        max_evidence_items: 8,
                        success_summary_override: Some(
                            "refined search toward `runtime shell host`".to_string(),
                        ),
                        no_bundle_message: "planner refine returned no evidence bundle",
                        failure_label: "planner refine failed",
                        unavailable_label: "planner refine backend unavailable",
                        missing_backend_message:
                            "no gatherer backend is configured for refined planner search",
                    },
                    &mut used_workspace_resources,
                )
                .await
        });

        assert_eq!(search_summary, "Found the likely implementation file.");
        assert_eq!(refine_summary, "refined search toward `runtime shell host`");
        assert!(used_workspace_resources);
        assert_eq!(loop_state.evidence_items.len(), 1);
        let recorded_requests = requests.lock().expect("gatherer requests lock");
        assert_eq!(recorded_requests.len(), 2);
        assert_eq!(recorded_requests[0].intent_reason, "planner-search");
        assert_eq!(recorded_requests[0].planning.mode, RetrievalMode::Linear);
        assert_eq!(
            recorded_requests[0].planning.retrieval_strategy,
            RetrievalStrategy::Lexical
        );
        assert_eq!(
            recorded_requests[0].planning.retrievers,
            vec![RetrieverOption::PathFuzzy]
        );
        assert_eq!(recorded_requests[1].intent_reason, "planner-refine");
        assert_eq!(recorded_requests[1].planning.mode, RetrievalMode::Graph);
        assert_eq!(
            recorded_requests[1].planning.retrieval_strategy,
            RetrievalStrategy::Vector
        );
        assert_eq!(
            recorded_requests[1].planning.retrievers,
            vec![RetrieverOption::PathFuzzy, RetrieverOption::SegmentFuzzy]
        );
    }

    #[test]
    fn action_bias_falls_back_to_prompt_derived_file_candidates() {
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
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction {
                known_edit: true,
                candidate_files: vec!["src/application/mod.rs".to_string()],
                resolution: None,
            }),
            &PlannerLoopState::default(),
            &decision,
            workspace.path(),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Execution
                && note.note.contains("Steering review [action-bias]")
                && note.note.contains("src/application/mod.rs")
        }));
    }

    #[test]
    fn planner_can_carry_resolver_outcomes_from_edit_signal_into_instruction_frame() {
        let resolution = crate::domain::ports::EntityResolutionOutcome::Resolved {
            target: crate::domain::ports::EntityResolutionCandidate::new(
                "apps/web/src/runtime-app.tsx",
                crate::domain::ports::EntityLookupMode::ExactPath,
                1,
            ),
            alternatives: vec![crate::domain::ports::EntityResolutionCandidate::new(
                "apps/web/src/runtime-shell.css",
                crate::domain::ports::EntityLookupMode::PathFragment,
                2,
            )],
            explanation: "exact authored path match".to_string(),
        };
        let edit = InitialEditInstruction {
            known_edit: true,
            candidate_files: vec!["apps/web/src/runtime-app.tsx".to_string()],
            resolution: Some(resolution.clone()),
        };

        let frame = super::instruction_frame_from_initial_edit(&edit)
            .expect("known edit should produce an instruction frame");

        assert_eq!(frame.candidate_files, edit.candidate_files);
        assert_eq!(frame.resolution, Some(resolution.clone()));

        let request = PlannerRequest::new(
            "tighten the manifold copy",
            "/workspace",
            InterpretationContext::default(),
            PlannerBudget::default(),
        )
        .with_loop_state(PlannerLoopState {
            target_resolution: Some(resolution.clone()),
            ..Default::default()
        });
        assert_eq!(request.loop_state.target_resolution, Some(resolution));
    }

    fn sample_model_paths(prefix: &str) -> ModelPaths {
        ModelPaths {
            weights: vec![PathBuf::from(format!("{prefix}-weights.safetensors"))],
            tokenizer: PathBuf::from(format!("{prefix}-tokenizer.json")),
            config: PathBuf::from(format!("{prefix}-config.json")),
            generation_config: Some(PathBuf::from(format!("{prefix}-generation-config.json"))),
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
