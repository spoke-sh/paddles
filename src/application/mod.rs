mod conversation_read_model;
mod deliberation;
mod evals;
mod interpretation_chamber;
pub mod read_model;
mod recursive_control;
mod synthesis_chamber;
mod turn_orchestration;

use self::conversation_read_model::ConversationReadModelChamber;
pub use self::deliberation::{
    DeliberationConfidence, DeliberationContinuation, DeliberationSignal, DeliberationSignals,
    extract_deliberation_signals,
};
pub use self::evals::{EvalRunner, recursive_harness_eval_corpus};
use self::interpretation_chamber::InterpretationChamber;
use self::recursive_control::RecursiveControlChamber;
use self::synthesis_chamber::SynthesisChamber;
use self::turn_orchestration::TurnOrchestrationChamber;
pub use crate::domain::model::{
    ConversationForensicProjection, ConversationForensicUpdate, ConversationManifoldProjection,
    ConversationProjectionReducer, ConversationProjectionSnapshot, ConversationProjectionUpdate,
    ConversationProjectionUpdateKind, ConversationTraceGraph, ConversationTraceGraphBranch,
    ConversationTraceGraphEdge, ConversationTraceGraphNode, ConversationTranscript,
    ConversationTranscriptEntry, ConversationTranscriptSpeaker, ConversationTranscriptUpdate,
    ForensicLifecycle, ForensicRecordProjection, ForensicTurnProjection, ForensicUpdateSink,
    ManifoldConduitState, ManifoldFrame, ManifoldGateState, ManifoldPrimitiveBasis,
    ManifoldPrimitiveKind, ManifoldPrimitiveState, ManifoldSignalState, ManifoldTurnProjection,
    NullForensicUpdateSink, NullTranscriptUpdateSink, TranscriptUpdateSink,
};

use crate::infrastructure::adapters::TransitContextResolver;
use crate::infrastructure::adapters::local_workspace_action_executor::LocalWorkspaceActionExecutor;
use crate::infrastructure::adapters::trace_recorders::default_trace_recorder_for_workspace;
use crate::infrastructure::adapters::workspace_entity_resolver::WorkspaceEntityResolver;
use crate::infrastructure::conversation_history::ConversationHistoryStore;
use crate::infrastructure::execution_governance::{
    ExecutionPermissionGate, GovernedTerminalCommandResult, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::external_capability::NoopExternalCapabilityBroker;
use crate::infrastructure::harness_profile::HarnessProfileSelection;
use crate::infrastructure::native_transport::NativeTransportRegistry;
use crate::infrastructure::providers::ModelProvider;
use crate::infrastructure::specialist_brains::SpecialistBrainRegistry;
use crate::infrastructure::terminal::run_background_terminal_command_with_execution_hand_registry;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use paddles_conversation::ConversationThreadStatus;
pub use paddles_conversation::{
    ContextLocator, ConversationSession, TraceArtifactId, TurnControlKind, TurnControlRequest,
};

use crate::domain::model::{
    ArtifactEnvelope, ArtifactKind, AuthoredResponse, BootContext, CollaborationMode,
    CollaborationModeRequest, CollaborationModeRequestTarget, CollaborationModeResult,
    CompactionDecision, CompactionPlan, ControlOperation, ControlResult, ControlResultStatus,
    ControlSubject, ConversationReplayView, ConversationThreadRef, ExecutionGovernanceDecision,
    ExecutionGovernanceOutcome, ExecutionGovernanceProfile, ExecutionHandDiagnostic,
    ExecutionPermissionRequest, ExternalCapabilityDescriptor, ExternalCapabilityInvocation,
    ExternalCapabilityResultStatus, ExternalCapabilitySourceRecord, ForensicArtifactCapture,
    ForensicTraceSink, InstructionFrame, InstructionIntent, MultiplexEventSink,
    NativeTransportDiagnostic, ResponseMode, SteeringGateKind, SteeringGatePhase, StrainFactor,
    StrainLevel, StructuredClarificationKind, StructuredClarificationOption,
    StructuredClarificationRequest, TaskTraceId, ThreadCandidate, ThreadDecision,
    ThreadDecisionKind, ThreadMergeMode, ThreadMergeRecord, TraceBranch, TraceBranchId,
    TraceBranchStatus, TraceCheckpointId, TraceCheckpointKind, TraceCompletionCheckpoint,
    TraceHarnessProfileSelection, TraceLineage, TraceLineageEdge, TraceLineageNodeKind,
    TraceLineageNodeRef, TraceLineageRelation, TraceModelExchangeArtifact, TraceModelExchangePhase,
    TraceRecord, TraceRecordId, TraceRecordKind, TraceReplay, TraceSelectionArtifact,
    TraceSelectionKind, TraceSignalContribution, TraceSignalKind, TraceSignalSnapshot,
    TraceTaskRoot, TraceToolCall, TraceTurnStarted, TurnControlOperation, TurnEvent, TurnEventSink,
    TurnIntent, TurnTraceId,
};
#[cfg(test)]
use crate::domain::model::{
    TraceWorkerArtifact, TraceWorkerIntegration, TraceWorkerLifecycle, WorkerArtifactKind,
    WorkerArtifactRecord, WorkerDelegationRequest, WorkerIntegrationStatus, WorkerLifecycleResult,
};
use crate::domain::ports::{
    ContextGatherRequest, ContextGatherer, ContextResolver, EntityLookupMode,
    EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest, EntityResolver,
    EvidenceBudget, EvidenceBundle, EvidenceItem, ExternalCapabilityBroker, GathererCapability,
    GroundingDomain, GroundingRequirement, InitialAction, InitialActionDecision,
    InitialEditInstruction, InterpretationContext, InterpretationProcedure,
    InterpretationProcedureStep, InterpretationRequest, InterpretationToolHint, ModelPaths,
    ModelRegistry, NormalizedEntityHint, OperatorMemory, PlannerAction, PlannerBudget,
    PlannerCapability, PlannerConfig, PlannerExecutionContract, PlannerLoopState, PlannerRequest,
    PlannerStepRecord, PlannerStrategyKind, PlannerTraceMetadata, PlannerTraceStep,
    RecursivePlanner, RecursivePlannerDecision, RetainedEvidence, RetrievalMode, RetrievalStrategy,
    RetrieverOption, SpecialistBrainRequest, SynthesisHandoff, SynthesizerEngine,
    ThreadDecisionRequest, TraceRecorder, TraceRecorderCapability, TraceSessionContextQuery,
    WorkspaceAction, WorkspaceActionCapability, WorkspaceActionExecutionFrame,
    WorkspaceActionExecutor, WorkspaceCapabilitySurface, WorkspaceToolCapability,
};
use anyhow::{Result, anyhow};
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
    execution_hand_registry: Mutex<Arc<ExecutionHandRegistry>>,
    workspace_action_executor: Mutex<Arc<dyn WorkspaceActionExecutor>>,
    external_capability_broker: Mutex<Arc<dyn ExternalCapabilityBroker>>,
    native_transport_registry: Mutex<Arc<NativeTransportRegistry>>,
    specialist_brain_registry: Mutex<Arc<SpecialistBrainRegistry>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeLaneRole {
    Planner,
    Synthesizer,
    Gatherer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResumableConversation {
    pub task_id: TaskTraceId,
    pub turn_count: usize,
    pub preview: String,
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
    synthesizer_thinking_mode: Option<String>,
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
            synthesizer_thinking_mode: None,
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

    pub fn with_synthesizer_thinking_mode(mut self, thinking_mode: Option<String>) -> Self {
        self.synthesizer_thinking_mode = thinking_mode;
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

    pub fn synthesizer_thinking_mode(&self) -> Option<&str> {
        self.synthesizer_thinking_mode.as_deref()
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

    pub fn harness_profile(&self) -> HarnessProfileSelection {
        HarnessProfileSelection::resolve(
            &self
                .planner
                .provider
                .capability_surface(&self.planner.model_id),
            &self
                .synthesizer
                .provider
                .capability_surface(&self.synthesizer.model_id),
        )
    }
}

struct ActiveRuntimeState {
    prepared: PreparedRuntimeLanes,
    planner_engine: Arc<dyn RecursivePlanner>,
    synthesizer_engine: Arc<dyn SynthesizerEngine>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
}

struct ActiveTurnGuard {
    session: ConversationSession,
    turn_id: TurnTraceId,
}

impl ActiveTurnGuard {
    fn new(session: ConversationSession, turn_id: TurnTraceId) -> Self {
        session.mark_turn_active(turn_id.clone());
        Self { session, turn_id }
    }
}

impl Drop for ActiveTurnGuard {
    fn drop(&mut self) {
        self.session.clear_turn_if_active(&self.turn_id);
    }
}

struct PlannerLoopContext {
    prepared: PreparedRuntimeLanes,
    planner_engine: Arc<dyn RecursivePlanner>,
    gatherer: Option<Arc<dyn ContextGatherer>>,
    resolver: Arc<dyn ContextResolver>,
    entity_resolver: Arc<dyn EntityResolver>,
    workspace_capability_surface: WorkspaceCapabilitySurface,
    execution_hands: Vec<ExecutionHandDiagnostic>,
    governance_profile: Option<ExecutionGovernanceProfile>,
    external_capabilities: Vec<ExternalCapabilityDescriptor>,
    interpretation: InterpretationContext,
    recent_turns: Vec<String>,
    recent_thread_summary: Option<String>,
    collaboration: CollaborationModeResult,
    specialist_runtime_notes: Vec<String>,
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
        let harness_profile = prepared.harness_profile();
        let trace_harness_profile = TraceHarnessProfileSelection {
            requested_profile_id: harness_profile.requested.id().to_string(),
            active_profile_id: harness_profile.active.id().to_string(),
            downgrade_reason: harness_profile.downgrade_reason.clone(),
        };

        if record_task_root {
            self.record_kind(
                None,
                TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                    prompt: prompt_artifact,
                    interpretation: interpretation_artifact,
                    planner_model: prepared.planner.model_id.clone(),
                    synthesizer_model: prepared.synthesizer.model_id.clone(),
                    harness_profile: trace_harness_profile,
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
                harness_profile: trace_harness_profile,
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
        signal_summary: Option<&str>,
        branch_id: Option<TraceBranchId>,
    ) {
        let branch_id = branch_id.or_else(|| self.default_branch_id());
        let record_id = self.record_kind(
            branch_id.clone(),
            TraceRecordKind::PlannerAction {
                action: action.to_string(),
                rationale: rationale.to_string(),
                signal_summary: signal_summary.map(str::to_string),
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

    fn record_control_result(&self, result: &ControlResult) {
        self.record_kind(
            result
                .subject
                .thread
                .as_ref()
                .and_then(ConversationThreadRef::branch_id)
                .or_else(|| self.default_branch_id()),
            TraceRecordKind::ControlResultRecorded(result.clone()),
        );
    }

    #[cfg(test)]
    pub(crate) fn record_worker_lifecycle(
        &self,
        parent_thread: &ConversationThreadRef,
        worker_thread: &ConversationThreadRef,
        request: WorkerDelegationRequest,
        result: WorkerLifecycleResult,
    ) {
        self.record_kind(
            parent_thread
                .branch_id()
                .or_else(|| self.default_branch_id()),
            TraceRecordKind::WorkerLifecycleRecorded(TraceWorkerLifecycle {
                request,
                result,
                parent_thread: parent_thread.clone(),
                worker_thread: worker_thread.clone(),
            }),
        );
    }

    #[cfg(test)]
    pub(crate) fn record_worker_artifact(
        &self,
        worker_thread: &ConversationThreadRef,
        record: WorkerArtifactRecord,
        content: impl Into<String>,
    ) -> TraceArtifactId {
        let mut artifact = self
            .text_artifact(
                worker_artifact_payload_kind(record.kind),
                format!("worker {} `{}`", record.kind.label(), record.label),
                content,
                1_000,
            )
            .with_label("paddles.worker_id", record.worker_id.clone())
            .with_label("paddles.worker_artifact_kind", record.kind.label());
        if !record.integration_hints.is_empty() {
            artifact = artifact.with_label(
                "paddles.worker_integration_hints",
                record.integration_hints.join(" | "),
            );
        }
        let artifact_id = artifact.artifact_id.clone();
        self.record_kind(
            worker_thread
                .branch_id()
                .or_else(|| self.default_branch_id()),
            TraceRecordKind::WorkerArtifactRecorded(TraceWorkerArtifact { record, artifact }),
        );
        artifact_id
    }

    #[cfg(test)]
    pub(crate) fn record_worker_integration(
        &self,
        parent_thread: &ConversationThreadRef,
        worker_thread: &ConversationThreadRef,
        worker_id: impl Into<String>,
        status: WorkerIntegrationStatus,
        detail: impl Into<String>,
        integrated_artifact_ids: Vec<TraceArtifactId>,
    ) {
        self.record_kind(
            parent_thread
                .branch_id()
                .or_else(|| self.default_branch_id()),
            TraceRecordKind::WorkerIntegrationRecorded(TraceWorkerIntegration {
                worker_id: worker_id.into(),
                parent_thread: parent_thread.clone(),
                worker_thread: worker_thread.clone(),
                status,
                detail: detail.into(),
                integrated_artifact_ids,
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
        let authored_response = response.clone();
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
                authored_response: Some(authored_response),
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

    fn record_checkpoint_without_response(
        &self,
        kind: TraceCheckpointKind,
        summary: impl Into<String>,
    ) {
        self.record_kind(
            self.default_branch_id(),
            TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                checkpoint_id: self.next_checkpoint_id(),
                kind,
                summary: summary.into(),
                response: None,
                authored_response: None,
                citations: Vec::new(),
                grounded: false,
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
            TurnEvent::ExecutionGovernanceProfileApplied { snapshot } => {
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::ExecutionGovernanceProfileDeclared(snapshot),
                );
            }
            TurnEvent::ExecutionGovernanceDecisionRecorded { decision } => {
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::ExecutionGovernanceDecisionRecorded(decision),
                );
            }
            TurnEvent::ControlStateChanged { result } => {
                self.record_control_result(&result);
            }
            TurnEvent::CollaborationModeChanged { result } => {
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::CollaborationModeDeclared(result),
                );
            }
            TurnEvent::StructuredClarificationChanged { result } => {
                self.record_kind(
                    self.default_branch_id(),
                    TraceRecordKind::StructuredClarificationRecorded(result),
                );
            }
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
                let success = if tool_name == "external_capability" {
                    summary.contains("status=succeeded")
                } else {
                    !summary.to_ascii_lowercase().contains("failed:")
                };
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

#[cfg(test)]
pub(crate) fn worker_artifact_payload_kind(kind: WorkerArtifactKind) -> ArtifactKind {
    match kind {
        WorkerArtifactKind::ToolCall => ArtifactKind::ToolInvocation,
        WorkerArtifactKind::ToolOutput => ArtifactKind::ToolOutput,
        WorkerArtifactKind::CompletionSummary => ArtifactKind::Selection,
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
            signal_summary,
        } => {
            let mut lines = vec![
                format!(
                    "• Planner step {sequence}: {}",
                    trim_event_detail(action, 1)
                ),
                format!("  └ Rationale: {}", trim_event_detail(rationale, 2)),
            ];
            if let Some(signal_summary) = signal_summary {
                lines.push(format!(
                    "    Signals: {}",
                    trim_event_detail(signal_summary, 2)
                ));
            }
            lines.join("\n")
        }
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
        TurnEvent::ControlStateChanged { result } => format!(
            "• Control: {}\n  └ {}",
            result.summary(),
            trim_event_detail(&result.detail, 2)
        ),
        TurnEvent::CollaborationModeChanged { result } => {
            let mut lines = vec![format!(
                "• Collaboration: {} {}",
                result.active.mode.label(),
                result.status.label()
            )];
            if let Some(request) = &result.request {
                lines.push(format!(
                    "  └ requested={} via {}",
                    request.target.label(),
                    request.source.label()
                ));
            }
            lines.push(format!(
                "  └ posture={} output={} clarification={}",
                result.active.mutation_posture.label(),
                result.active.output_contract.label(),
                result.active.clarification_policy.label()
            ));
            if !result.detail.trim().is_empty() {
                lines.push(format!("  └ {}", trim_event_detail(&result.detail, 2)));
            }
            lines.join("\n")
        }
        TurnEvent::StructuredClarificationChanged { result } => {
            let mut lines = vec![format!(
                "• Clarification: {} {}",
                result.request.kind.label(),
                result.status.label()
            )];
            lines.push(format!(
                "  └ {}",
                trim_event_detail(&result.request.prompt, 2)
            ));
            if !result.request.options.is_empty() {
                lines.push(format!(
                    "  └ options: {}",
                    trim_event_detail(
                        &result
                            .request
                            .options
                            .iter()
                            .map(|option| format!("{} ({})", option.option_id, option.label))
                            .collect::<Vec<_>>()
                            .join(", "),
                        2
                    )
                ));
            }
            if !result.detail.trim().is_empty() {
                lines.push(format!("  └ {}", trim_event_detail(&result.detail, 2)));
            }
            lines.join("\n")
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
        TurnEvent::ExecutionGovernanceProfileApplied { snapshot } => format!(
            "• {}\n  └ {}",
            snapshot.summary(),
            trim_event_detail(&snapshot.detail(), 3)
        ),
        TurnEvent::ExecutionGovernanceDecisionRecorded { decision } => format!(
            "• {}\n  └ {}",
            decision.summary(),
            trim_event_detail(&decision.detail(), 3)
        ),
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
        let workspace_root = workspace_root.into();
        Self::with_trace_recorder(
            workspace_root.clone(),
            registry,
            operator_memory,
            synthesizer_factory,
            planner_factory,
            gatherer_factory,
            default_trace_recorder_for_workspace(&workspace_root),
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
        let workspace_root = workspace_root.into();
        let execution_hand_registry = Arc::new(ExecutionHandRegistry::default());
        let next_task_sequence = next_task_sequence_from_trace_recorder(trace_recorder.as_ref());
        let workspace_action_executor: Arc<dyn WorkspaceActionExecutor> =
            Arc::new(LocalWorkspaceActionExecutor::with_execution_hand_registry(
                workspace_root.clone(),
                Arc::clone(&execution_hand_registry),
            ));
        Self {
            workspace_root,
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
            trace_counter: AtomicU64::new(next_task_sequence),
            sessions: Mutex::new(HashMap::new()),
            shared_session_id: Mutex::new(None),
            conversation_history_store: Mutex::new(None),
            execution_hand_registry: Mutex::new(execution_hand_registry),
            workspace_action_executor: Mutex::new(workspace_action_executor),
            external_capability_broker: Mutex::new(Arc::new(
                NoopExternalCapabilityBroker::default(),
            )),
            native_transport_registry: Mutex::new(Arc::new(NativeTransportRegistry::default())),
            specialist_brain_registry: Mutex::new(Arc::new(SpecialistBrainRegistry::new())),
        }
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    pub fn trace_recorder_capability(&self) -> TraceRecorderCapability {
        self.trace_recorder.capability()
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

    pub fn set_execution_hand_registry(&self, registry: Arc<ExecutionHandRegistry>) {
        *self
            .execution_hand_registry
            .lock()
            .expect("execution hand registry lock") = registry;
    }

    pub fn execution_hand_registry(&self) -> Arc<ExecutionHandRegistry> {
        Arc::clone(
            &self
                .execution_hand_registry
                .lock()
                .expect("execution hand registry lock"),
        )
    }

    pub fn execution_hand_diagnostics(&self) -> Vec<ExecutionHandDiagnostic> {
        self.execution_hand_registry().diagnostics()
    }

    pub fn set_workspace_action_executor(&self, executor: Arc<dyn WorkspaceActionExecutor>) {
        *self
            .workspace_action_executor
            .lock()
            .expect("workspace action executor lock") = executor;
    }

    pub fn workspace_action_executor(&self) -> Arc<dyn WorkspaceActionExecutor> {
        Arc::clone(
            &self
                .workspace_action_executor
                .lock()
                .expect("workspace action executor lock"),
        )
    }

    pub fn set_external_capability_broker(&self, broker: Arc<dyn ExternalCapabilityBroker>) {
        *self
            .external_capability_broker
            .lock()
            .expect("external capability broker lock") = broker;
    }

    pub fn external_capability_broker(&self) -> Arc<dyn ExternalCapabilityBroker> {
        Arc::clone(
            &self
                .external_capability_broker
                .lock()
                .expect("external capability broker lock"),
        )
    }

    pub fn external_capability_descriptors(&self) -> Vec<ExternalCapabilityDescriptor> {
        self.external_capability_broker().descriptors()
    }

    pub fn planner_execution_contract(
        &self,
        gatherer: Option<&Arc<dyn ContextGatherer>>,
        collaboration: &CollaborationModeResult,
        instruction_frame: Option<&InstructionFrame>,
        grounding: Option<&GroundingRequirement>,
    ) -> PlannerExecutionContract {
        let workspace_capability_surface = self.workspace_action_executor().capability_surface();
        let execution_hand_registry = self.execution_hand_registry();
        let execution_hands = execution_hand_registry.diagnostics();
        let governance_profile = execution_hand_registry.governance_profile();
        let external_capabilities = self.external_capability_descriptors();
        build_planner_execution_contract(PlannerExecutionContractContext {
            workspace_capability_surface: &workspace_capability_surface,
            execution_hands: &execution_hands,
            governance_profile: governance_profile.as_ref(),
            external_capabilities: &external_capabilities,
            gatherer,
            collaboration,
            instruction_frame,
            grounding,
        })
    }

    pub fn set_native_transport_registry(&self, registry: Arc<NativeTransportRegistry>) {
        *self
            .native_transport_registry
            .lock()
            .expect("native transport registry lock") = registry;
    }

    pub fn native_transport_registry(&self) -> Arc<NativeTransportRegistry> {
        Arc::clone(
            &self
                .native_transport_registry
                .lock()
                .expect("native transport registry lock"),
        )
    }

    pub fn native_transport_diagnostics(&self) -> Vec<NativeTransportDiagnostic> {
        self.native_transport_registry().diagnostics()
    }

    pub fn set_specialist_brain_registry(&self, registry: Arc<SpecialistBrainRegistry>) {
        *self
            .specialist_brain_registry
            .lock()
            .expect("specialist brain registry lock") = registry;
    }

    pub fn specialist_brain_registry(&self) -> Arc<SpecialistBrainRegistry> {
        Arc::clone(
            &self
                .specialist_brain_registry
                .lock()
                .expect("specialist brain registry lock"),
        )
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

    fn turn_orchestration(&self) -> TurnOrchestrationChamber<'_> {
        TurnOrchestrationChamber::new(self)
    }

    fn interpretation_chamber(&self) -> InterpretationChamber<'_> {
        InterpretationChamber::new(self)
    }

    fn recursive_control(&self) -> RecursiveControlChamber<'_> {
        RecursiveControlChamber::new(self)
    }

    fn synthesis_chamber(&self) -> SynthesisChamber<'_> {
        SynthesisChamber::new(self)
    }

    fn conversation_read_model(&self) -> ConversationReadModelChamber<'_> {
        ConversationReadModelChamber::new(self)
    }

    #[cfg(test)]
    fn replay_for_known_session(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<Option<crate::domain::model::TraceReplay>> {
        self.conversation_read_model()
            .replay_for_known_session(task_id)
    }

    pub fn replay_conversation_forensics(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationForensicProjection> {
        self.conversation_read_model()
            .replay_conversation_forensics(task_id)
    }

    pub fn replay_turn_forensics(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<ForensicTurnProjection>> {
        self.conversation_read_model()
            .replay_turn_forensics(task_id, turn_id)
    }

    pub fn replay_conversation_manifold(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationManifoldProjection> {
        self.conversation_read_model()
            .replay_conversation_manifold(task_id)
    }

    pub fn replay_turn_manifold(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<ManifoldTurnProjection>> {
        self.conversation_read_model()
            .replay_turn_manifold(task_id, turn_id)
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

    pub fn query_session_context_slice(
        &self,
        task_id: &TaskTraceId,
        query: TraceSessionContextQuery,
    ) -> Result<crate::domain::ports::TraceSessionContextSlice> {
        self.trace_recorder.query_session_context(task_id, &query)
    }

    #[cfg(test)]
    fn recent_turn_summaries(
        &self,
        session: &ConversationSession,
        synthesizer_engine: &dyn SynthesizerEngine,
    ) -> Result<Vec<String>> {
        self.synthesis_chamber()
            .recent_turn_summaries(session, synthesizer_engine)
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

    #[cfg(test)]
    fn finalize_turn_response(
        &self,
        trace: &StructuredTurnTrace,
        session: &ConversationSession,
        active_thread: &ConversationThreadRef,
        prompt: &str,
        response: &AuthoredResponse,
    ) -> String {
        self.synthesis_chamber().finalize_turn_response(
            trace,
            session,
            active_thread,
            prompt,
            response,
        )
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
            .with_profile(context.prepared.harness_profile().active_profile_id())
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

    fn load_persisted_conversation_session(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationSession> {
        if let Some(session) = self
            .sessions
            .lock()
            .expect("conversation sessions lock")
            .get(task_id.as_str())
            .cloned()
        {
            return Ok(session);
        }

        let known_task = self
            .trace_recorder
            .task_ids()
            .into_iter()
            .any(|candidate| candidate == *task_id);
        if !known_task {
            return Err(anyhow!("conversation '{}' was not found", task_id.as_str()));
        }

        let replay = self.trace_recorder.replay(task_id)?;
        let session = rehydrate_conversation_session(&replay);
        Ok(self.register_session(session))
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

    pub fn resumable_conversations(&self) -> Result<Vec<ResumableConversation>> {
        let current_shared_session_id = self
            .shared_session_id
            .lock()
            .expect("shared session lock")
            .clone();
        let mut task_ids = self.trace_recorder.task_ids();
        task_ids.sort_by(compare_task_ids_desc);

        let mut conversations = Vec::new();
        for task_id in task_ids {
            if current_shared_session_id.as_deref() == Some(task_id.as_str()) {
                continue;
            }

            let transcript = self.replay_conversation_transcript(&task_id)?;
            if transcript.entries.is_empty() {
                continue;
            }

            let turn_count = transcript
                .entries
                .iter()
                .filter(|entry| entry.speaker == ConversationTranscriptSpeaker::User)
                .count();
            let preview = transcript
                .entries
                .iter()
                .find(|entry| {
                    entry.speaker == ConversationTranscriptSpeaker::User
                        && !entry.content.trim().is_empty()
                })
                .or_else(|| transcript.entries.first())
                .map(|entry| trim_for_planner(&entry.content, 120))
                .unwrap_or_default();

            conversations.push(ResumableConversation {
                task_id,
                turn_count,
                preview,
            });
        }

        Ok(conversations)
    }

    pub fn restore_shared_conversation_session(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationSession> {
        let session = self.load_persisted_conversation_session(task_id)?;
        *self.shared_session_id.lock().expect("shared session lock") =
            Some(task_id.as_str().to_string());
        Ok(session)
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
        self.conversation_read_model()
            .replay_conversation_transcript(task_id)
    }

    pub fn replay_conversation_trace_graph(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationTraceGraph> {
        self.conversation_read_model()
            .replay_conversation_trace_graph(task_id)
    }

    pub fn replay_conversation_delegation(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<crate::domain::model::ConversationDelegationProjection> {
        self.conversation_read_model()
            .replay_conversation_delegation(task_id)
    }

    pub fn replay_conversation_projection(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationProjectionSnapshot> {
        self.conversation_read_model()
            .replay_conversation_projection(task_id)
    }

    pub fn projection_update_for_transcript(
        &self,
        update: &ConversationTranscriptUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        self.conversation_read_model()
            .projection_update_for_transcript(update)
    }

    pub fn projection_update_for_forensic(
        &self,
        update: &ConversationForensicUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        self.conversation_read_model()
            .projection_update_for_forensic(update)
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
        let synthesizer_prepared_model_id = config.synthesizer_provider().prepare_runtime_model_id(
            config.synthesizer_model_id(),
            config.synthesizer_thinking_mode(),
        );
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
        let planner_thinking_mode = (planner_provider == config.synthesizer_provider()
            && planner_model_id == config.synthesizer_model_id())
        .then(|| config.synthesizer_thinking_mode())
        .flatten();
        let planner_prepared_model_id =
            planner_provider.prepare_runtime_model_id(&planner_model_id, planner_thinking_mode);
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
            planner_prepared_model_id,
            planner_paths,
        );
        let synthesizer = Self::build_lane(
            RuntimeLaneRole::Synthesizer,
            config.synthesizer_provider(),
            synthesizer_prepared_model_id,
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
        self.execution_hand_registry().set_governance_profile(
            prepared
                .harness_profile()
                .active_execution_governance()
                .clone(),
        );

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
        self.turn_orchestration().process_prompt(prompt).await
    }

    pub async fn process_prompt_with_sink(
        &self,
        prompt: &str,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.turn_orchestration()
            .process_prompt_with_sink(prompt, event_sink)
            .await
    }

    pub async fn process_prompt_in_session_with_sink(
        &self,
        prompt: &str,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.turn_orchestration()
            .process_prompt_in_session_with_sink(prompt, session, event_sink)
            .await
    }

    pub async fn process_prompt_in_session_with_mode_request_and_sink(
        &self,
        prompt: &str,
        session: ConversationSession,
        mode_request: Option<CollaborationModeRequest>,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.turn_orchestration()
            .process_prompt_in_session_with_mode_request_and_sink(
                prompt,
                session,
                mode_request,
                event_sink,
            )
            .await
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

    async fn apply_turn_controls_at_safe_checkpoint(
        &self,
        context: &PlannerLoopContext,
        trace: &Arc<StructuredTurnTrace>,
    ) -> Result<Option<PlannerLoopOutcome>> {
        let requests = trace.session.take_turn_control_requests(&trace.turn_id);
        if requests.is_empty() {
            return Ok(None);
        }

        let effective_index = requests
            .iter()
            .enumerate()
            .max_by_key(|(_, request)| request.captured_sequence)
            .map(|(index, _)| index)
            .expect("non-empty turn control batch");
        let effective = requests[effective_index].clone();

        for (index, request) in requests.iter().enumerate() {
            if index == effective_index {
                continue;
            }
            trace.emit(TurnEvent::ControlStateChanged {
                result: turn_control_result_for_request(
                    request,
                    ControlResultStatus::Stale,
                    format!(
                        "Superseded by later `{}` control request on the active turn.",
                        effective.kind.label()
                    ),
                ),
            });
        }

        match effective.kind {
            TurnControlKind::Interrupt => {
                trace.emit(TurnEvent::ControlStateChanged {
                    result: turn_control_result_for_request(
                        &effective,
                        ControlResultStatus::Applied,
                        "Interrupted the active turn at a safe checkpoint.",
                    ),
                });
                Ok(Some(PlannerLoopOutcome {
                    evidence: None,
                    direct_answer: Some(AuthoredResponse::from_plain_text(
                        ResponseMode::DirectAnswer,
                        "Interrupted the active turn at a safe checkpoint.",
                    )),
                    instruction_frame: None,
                    grounding: None,
                    continuation: None,
                }))
            }
            TurnControlKind::Steer => {
                let Some(candidate) = trace
                    .session
                    .capture_candidate_from_turn_control(&effective)
                else {
                    trace.emit(TurnEvent::ControlStateChanged {
                        result: turn_control_result_for_request(
                            &effective,
                            ControlResultStatus::Rejected,
                            "Steering requires a non-empty prompt payload.",
                        ),
                    });
                    return Ok(None);
                };
                trace.emit(TurnEvent::ThreadCandidateCaptured {
                    candidate_id: candidate.candidate_id.as_str().to_string(),
                    active_thread: candidate.active_thread.stable_id(),
                    prompt: candidate.prompt.clone(),
                });
                trace.record_thread_candidate(&candidate);

                let interpretation = self
                    .derive_interpretation_context(
                        &candidate.prompt,
                        context.planner_engine.as_ref(),
                        trace.clone() as Arc<dyn TurnEventSink>,
                    )
                    .await;
                let active_thread = trace.session.active_thread();
                let thread_request = ThreadDecisionRequest::new(
                    self.workspace_root.clone(),
                    interpretation,
                    active_thread.clone(),
                    candidate.clone(),
                )
                .with_recent_turns(context.recent_turns.clone())
                .with_known_threads(trace.session.known_threads())
                .with_recent_thread_summary(
                    trace
                        .session
                        .recent_thread_summary(&active_thread.thread_ref),
                );

                let decision = context
                    .planner_engine
                    .select_thread_decision(
                        &thread_request,
                        trace.clone() as Arc<dyn TurnEventSink>,
                    )
                    .await?;
                trace.emit(TurnEvent::ThreadDecisionApplied {
                    candidate_id: candidate.candidate_id.as_str().to_string(),
                    decision: decision.kind.label().to_string(),
                    target_thread: decision.target_thread.stable_id(),
                    rationale: decision.rationale.clone(),
                });
                trace.record_thread_decision(&decision, &candidate.active_thread);

                let branch_id = if matches!(decision.kind, ThreadDecisionKind::OpenChildThread) {
                    let branch_id = trace.session.next_branch_id();
                    trace.declare_branch(
                        branch_id.clone(),
                        decision
                            .new_thread_label
                            .as_deref()
                            .unwrap_or(candidate.prompt.as_str()),
                        Some(decision.rationale.as_str()),
                        candidate.active_thread.branch_id(),
                    );
                    Some(branch_id)
                } else {
                    None
                };

                if matches!(decision.kind, ThreadDecisionKind::MergeIntoTarget) {
                    trace.emit(TurnEvent::ThreadMerged {
                        source_thread: candidate.active_thread.stable_id(),
                        target_thread: decision.target_thread.stable_id(),
                        mode: decision
                            .merge_mode
                            .unwrap_or(ThreadMergeMode::Summary)
                            .label()
                            .to_string(),
                        summary: decision.merge_summary.clone(),
                    });
                    trace.record_thread_merge(
                        &decision,
                        &candidate.active_thread,
                        &decision.target_thread,
                    );
                }

                trace
                    .session
                    .apply_thread_decision(&decision, branch_id, &candidate.prompt);
                trace.emit(TurnEvent::ControlStateChanged {
                    result: turn_control_result_for_request(
                        &effective,
                        ControlResultStatus::Applied,
                        format!("Steered the active turn via `{}`.", decision.kind.label()),
                    ),
                });

                Ok(Some(PlannerLoopOutcome {
                    evidence: None,
                    direct_answer: None,
                    instruction_frame: None,
                    grounding: None,
                    continuation: Some(PlannerLoopContinuation {
                        prompt: candidate.prompt,
                        summary: format!(
                            "turn handed off to steered prompt on {} via {}",
                            decision.target_thread.stable_id(),
                            decision.kind.label()
                        ),
                    }),
                }))
            }
        }
    }

    fn expire_turn_control_requests(&self, trace: &Arc<StructuredTurnTrace>, detail: &str) {
        for request in trace.session.take_turn_control_requests(&trace.turn_id) {
            trace.emit(TurnEvent::ControlStateChanged {
                result: turn_control_result_for_request(
                    &request,
                    ControlResultStatus::Unavailable,
                    detail.to_string(),
                ),
            });
        }
    }

    pub async fn process_thread_candidate_in_session_with_sink(
        &self,
        candidate: ThreadCandidate,
        session: ConversationSession,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.turn_orchestration()
            .process_thread_candidate_in_session_with_sink(candidate, session, event_sink)
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

        enrich_interpretation_with_external_capabilities(
            enrich_interpretation_with_workspace_capability_surface(
                interpretation,
                &self.workspace_action_executor().capability_surface(),
            ),
            &self.external_capability_descriptors(),
        )
    }

    async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        initial_decision: Option<RecursivePlannerDecision>,
        trace: Arc<StructuredTurnTrace>,
    ) -> Result<PlannerLoopOutcome> {
        let mut context = context;
        let base_budget =
            planner_budget_for_turn(context.instruction_frame.as_ref(), &context.initial_edit);
        let mut budget = planner_budget_for_replan_attempt(&base_budget, 0);
        let harness_profile = context.prepared.harness_profile();
        let mut loop_state = PlannerLoopState {
            target_resolution: context.initial_edit.resolution.clone(),
            refinement_policy: harness_profile.active_refinement_policy(),
            ..PlannerLoopState::default()
        };
        let mut used_workspace_resources = false;
        let mut stop_reason = None;
        let mut direct_answer = None;
        let mut instruction_frame = context.instruction_frame.clone();
        let mut pending_initial_decision = initial_decision;
        let mut pending_deliberation_signals = DeliberationSignals::default();
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
            if let Some(control_outcome) = self
                .apply_turn_controls_at_safe_checkpoint(&context, &trace)
                .await?
            {
                return Ok(control_outcome);
            }

            if sequence > budget.max_steps {
                if activate_replan(
                    "planner-budget-exhausted",
                    ReplanActivation {
                        instruction_frame: instruction_frame.as_ref(),
                        base_budget: &base_budget,
                        completed_replans: &mut replan_count,
                        budget: &mut budget,
                        loop_state: &mut loop_state,
                        trace: trace.as_ref(),
                    },
                ) {
                    continue;
                }
                break;
            }

            let evidence_count_before = loop_state.evidence_items.len();
            let planner_selected_this_step = pending_initial_decision.is_none();
            sync_deliberation_signal_note(&mut loop_state, &pending_deliberation_signals);
            let mut decision = if let Some(decision) = pending_initial_decision.take() {
                decision
            } else {
                context.workspace_capability_surface =
                    self.workspace_action_executor().capability_surface();
                let request = PlannerRequest::new(
                    prompt,
                    self.workspace_root.clone(),
                    context.interpretation.clone(),
                    budget.clone(),
                )
                .with_collaboration(context.collaboration.clone())
                .with_recent_turns(context.recent_turns.clone())
                .with_recent_thread_summary(context.recent_thread_summary.clone())
                .with_runtime_notes(planner_runtime_notes(
                    context.gatherer.as_ref(),
                    &context.specialist_runtime_notes,
                    &context.collaboration,
                ))
                .with_execution_contract(build_planner_execution_contract(
                    PlannerExecutionContractContext {
                        workspace_capability_surface: &context.workspace_capability_surface,
                        execution_hands: &context.execution_hands,
                        governance_profile: context.governance_profile.as_ref(),
                        external_capabilities: &context.external_capabilities,
                        gatherer: context.gatherer.as_ref(),
                        collaboration: &context.collaboration,
                        instruction_frame: instruction_frame.as_ref(),
                        grounding: context.grounding.as_ref(),
                    },
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
                    DecisionReviewFrame {
                        deliberation_signals: &pending_deliberation_signals,
                        workspace_root: &self.workspace_root,
                        trace: trace.clone(),
                    },
                )
                .await?;
            }

            decision = sanitize_recursive_planner_decision_for_collaboration(
                &context.collaboration,
                decision,
            );
            instruction_frame =
                merge_instruction_frame_with_edit_signal(instruction_frame, &decision.edit);
            if let Some(resolution) = decision.edit.resolution.clone() {
                loop_state.target_resolution = Some(resolution);
            }
            if planner_selected_this_step {
                let (compiled_rationale, signal_summary) = compile_recursive_paddles_rationale(
                    &decision.action,
                    &loop_state.evidence_items,
                    &pending_deliberation_signals,
                );
                decision.rationale = compiled_rationale;
                trace.emit(TurnEvent::PlannerActionSelected {
                    sequence,
                    action: decision.action.summary(),
                    rationale: decision.rationale.clone(),
                    signal_summary: signal_summary.clone(),
                });
                trace.record_planner_action(
                    &decision.action.summary(),
                    &decision.rationale,
                    signal_summary.as_deref(),
                    None,
                );
            }

            trace.emit(TurnEvent::PlannerStepProgress {
                step_number: sequence,
                step_limit: budget.max_steps,
                action: decision.action.summary(),
                query: decision.action.target_query(),
                evidence_count: loop_state.evidence_items.len(),
            });

            let mut accepted_stop = false;
            let outcome = if let Some(outcome) = collaboration_boundary_for_action(
                &context.collaboration,
                &decision.action,
                &decision.edit,
            ) {
                trace.emit(TurnEvent::Fallback {
                    stage: "collaboration-mode".to_string(),
                    reason: outcome.summary.clone(),
                });
                if let Some(clarification) = outcome.clarification.clone() {
                    trace.emit(TurnEvent::StructuredClarificationChanged {
                        result: clarification,
                    });
                }
                direct_answer = Some(outcome.response);
                stop_reason = Some(outcome.reason);
                accepted_stop = true;
                outcome.summary
            } else {
                match &decision.action {
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
                                    self.execution_hand_registry(),
                                    command,
                                    &call_id,
                                    trace.as_ref(),
                                ) {
                                    Ok(output) => {
                                        emit_execution_governance_decision(
                                            trace.as_ref(),
                                            Some(&call_id),
                                            Some("inspect"),
                                            output.governance_request,
                                            output.governance_outcome,
                                        );
                                        if !output.command_succeeded {
                                            let summary =
                                                format!("inspect failed: {}", output.summary);
                                            trace.emit(TurnEvent::ToolFinished {
                                                call_id,
                                                tool_name: "inspect".to_string(),
                                                summary: summary.clone(),
                                            });
                                            append_evidence_item(
                                                &mut loop_state.evidence_items,
                                                EvidenceItem {
                                                    source: format!("command: {command}"),
                                                    snippet: trim_for_planner(&summary, 800),
                                                    rationale: decision.rationale.clone(),
                                                    rank: 0,
                                                },
                                                budget.max_evidence_items,
                                            );
                                            used_workspace_resources = true;
                                            summary
                                        } else {
                                            let summary = planner_terminal_tool_success_summary(
                                                "inspect",
                                                &output.summary,
                                            );
                                            trace.emit(TurnEvent::ToolFinished {
                                                call_id,
                                                tool_name: "inspect".to_string(),
                                                summary,
                                            });
                                            append_evidence_item(
                                                &mut loop_state.evidence_items,
                                                EvidenceItem {
                                                    source: format!("command: {command}"),
                                                    snippet: trim_for_planner(&output.summary, 800),
                                                    rationale: decision.rationale.clone(),
                                                    rank: 0,
                                                },
                                                budget.max_evidence_items,
                                            );
                                            used_workspace_resources = true;
                                            format!("inspected {command}")
                                        }
                                    }
                                    Err(err) => {
                                        let summary = format!("inspect failed: {err:#}");
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: "inspect".to_string(),
                                            summary: summary.clone(),
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: format!("command: {command}"),
                                                snippet: trim_for_planner(&summary, 800),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        used_workspace_resources = true;
                                        summary
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
                                self.execution_hand_registry(),
                                command,
                                &call_id,
                                trace.as_ref(),
                            ) {
                                Ok(result) => {
                                    emit_execution_governance_decision(
                                        trace.as_ref(),
                                        Some(&call_id),
                                        Some("shell"),
                                        result.governance_request,
                                        result.governance_outcome,
                                    );
                                    if result.command_succeeded {
                                        let summary = planner_terminal_tool_success_summary(
                                            "shell",
                                            &result.summary,
                                        );
                                        trace.emit(TurnEvent::ToolFinished {
                                            call_id,
                                            tool_name: "shell".to_string(),
                                            summary,
                                        });
                                        append_evidence_item(
                                            &mut loop_state.evidence_items,
                                            EvidenceItem {
                                                source: format!("command: {command}"),
                                                snippet: trim_for_planner(&result.summary, 1_200),
                                                rationale: decision.rationale.clone(),
                                                rank: 0,
                                            },
                                            budget.max_evidence_items,
                                        );
                                        if let Some(frame) = instruction_frame.as_mut() {
                                            frame.note_successful_workspace_action(action);
                                        }
                                        used_workspace_resources = true;
                                        result.summary
                                    } else {
                                        let summary =
                                            format!("Tool `shell` failed: {}", result.summary);
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
                                        summary
                                    }
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
                                    summary
                                }
                            }
                        }
                        WorkspaceAction::ExternalCapability { invocation } => {
                            let call_id = format!("planner-tool-{sequence}");
                            let broker = self.external_capability_broker();
                            let descriptor = broker.descriptor(&invocation.capability_id);
                            trace.emit(TurnEvent::ToolCalled {
                                call_id: call_id.clone(),
                                tool_name: action.label().to_string(),
                                invocation: format_external_capability_invocation(
                                    descriptor.as_ref(),
                                    invocation,
                                ),
                            });
                            let summary = execute_external_capability_action(
                                broker,
                                context
                                    .prepared
                                    .harness_profile()
                                    .active_execution_governance(),
                                invocation,
                                ExternalCapabilityExecutionFrame {
                                    rationale: decision.rationale.as_str(),
                                    evidence_limit: budget.max_evidence_items,
                                    evidence_items: &mut loop_state.evidence_items,
                                    call_id: &call_id,
                                    event_sink: trace.as_ref(),
                                },
                            );
                            used_workspace_resources = true;
                            summary
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
                                trace.record_entity_resolution_outcome(
                                    outcome,
                                    "exact-mutation-path",
                                );
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
                                match self.workspace_action_executor().execute_workspace_action(
                                    action,
                                    WorkspaceActionExecutionFrame {
                                        call_id: &call_id,
                                        event_sink: trace.as_ref(),
                                    },
                                ) {
                                    Ok(result) => {
                                        if let (
                                            Some(governance_request),
                                            Some(governance_outcome),
                                        ) = (
                                            result.governance_request.clone(),
                                            result.governance_outcome.clone(),
                                        ) {
                                            emit_execution_governance_decision(
                                                trace.as_ref(),
                                                Some(&call_id),
                                                Some(action.label()),
                                                governance_request,
                                                governance_outcome,
                                            );
                                        }
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
                            direct_answer =
                                stop_reason_direct_answer(reason, decision.answer.clone());
                            stop_reason = Some(reason.clone());
                            accepted_stop = true;
                            format!("planner requested synthesis: {reason}")
                        }
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
            pending_deliberation_signals =
                extract_deliberation_signals(decision.deliberation_state.as_ref());
            sync_deliberation_signal_note(&mut loop_state, &pending_deliberation_signals);

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
                    &pending_deliberation_signals,
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
                continuation: None,
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
            continuation: None,
        })
    }

    fn mid_loop_refinement_reason(
        &self,
        sequence: usize,
        loop_state: &PlannerLoopState,
        steps_without_new_evidence: usize,
        deliberation_signals: &DeliberationSignals,
    ) -> Option<String> {
        let policy = &loop_state.refinement_policy;
        if !policy.enabled {
            return None;
        }

        if continuation_requires_tool_follow_up(deliberation_signals) {
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
    continuation: Option<PlannerLoopContinuation>,
}

struct PlannerLoopContinuation {
    prompt: String,
    summary: String,
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

fn turn_control_operation(kind: TurnControlKind) -> TurnControlOperation {
    match kind {
        TurnControlKind::Steer => TurnControlOperation::Steer,
        TurnControlKind::Interrupt => TurnControlOperation::Interrupt,
    }
}

fn turn_control_result_for_request(
    request: &TurnControlRequest,
    status: ControlResultStatus,
    detail: impl Into<String>,
) -> ControlResult {
    ControlResult {
        operation: ControlOperation::Turn(turn_control_operation(request.kind)),
        status,
        subject: ControlSubject {
            turn_id: Some(request.turn_id.clone()),
            thread: Some(request.active_thread.clone()),
        },
        detail: detail.into(),
    }
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
        direct_answer: None,
        instruction_frame: None,
        initial_edit: InitialEditInstruction::default(),
        grounding: None,
    }
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
    trace: &'a StructuredTurnTrace,
}

fn activate_replan(stop_reason: &str, activation: ReplanActivation<'_>) -> bool {
    let ReplanActivation {
        instruction_frame,
        base_budget,
        completed_replans,
        budget,
        loop_state,
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

                    deliberation_state: None,
                }),
                direct_answer: None,
                instruction_frame,
                initial_edit: edit,
                grounding,
            }
        }
    }
}

fn resolve_collaboration_mode_request(
    request: Option<CollaborationModeRequest>,
) -> CollaborationModeResult {
    match request {
        None => CollaborationModeResult::default(),
        Some(request) => match request.target.clone() {
            CollaborationModeRequestTarget::Known(mode) => {
                CollaborationModeResult::applied(request, mode.state())
            }
            CollaborationModeRequestTarget::Unsupported(label) => CollaborationModeResult::invalid(
                request,
                CollaborationMode::Execution.state(),
                format!("unsupported collaboration mode `{label}`; continuing in execution mode"),
            ),
        },
    }
}

fn collaboration_runtime_notes(collaboration: &CollaborationModeResult) -> Vec<String> {
    let mut notes = vec![format!(
        "Collaboration mode: {} (status={}, mutation_posture={}, output_contract={}, clarification_policy={}).",
        collaboration.active.mode.label(),
        collaboration.status.label(),
        collaboration.active.mutation_posture.label(),
        collaboration.active.output_contract.label(),
        collaboration.active.clarification_policy.label(),
    )];
    if !collaboration.detail.trim().is_empty() {
        notes.push(format!("Mode detail: {}", collaboration.detail.trim()));
    }

    match collaboration.active.mode {
        CollaborationMode::Planning => notes.push(
            "Planning mode is read-only. Prefer search, list_files, read, inspect, or diff. If progress would require shell or file mutation, stop and ask for bounded clarification instead."
                .to_string(),
        ),
        CollaborationMode::Execution => notes.push(
            "Execution mode is the default mutation lane. Continue to honor execution governance and explicit instruction boundaries."
                .to_string(),
        ),
        CollaborationMode::Review => notes.push(
            "Review mode is read-only. Inspect local changes first with diff-backed evidence. Final output must list findings first with grounded file or line references, then residual risks or gaps."
                .to_string(),
        ),
    }

    notes
}

fn sanitize_initial_edit_instruction_for_collaboration(
    collaboration: &CollaborationModeResult,
    edit: InitialEditInstruction,
) -> InitialEditInstruction {
    if collaboration.active.mutation_posture.allows_mutation() {
        edit
    } else {
        InitialEditInstruction {
            known_edit: false,
            candidate_files: edit.candidate_files,
            resolution: edit.resolution,
        }
    }
}

fn sanitize_initial_action_decision_for_collaboration(
    collaboration: &CollaborationModeResult,
    mut decision: InitialActionDecision,
) -> InitialActionDecision {
    decision.edit =
        sanitize_initial_edit_instruction_for_collaboration(collaboration, decision.edit);
    decision
}

fn sanitize_recursive_planner_decision_for_collaboration(
    collaboration: &CollaborationModeResult,
    mut decision: RecursivePlannerDecision,
) -> RecursivePlannerDecision {
    decision.edit =
        sanitize_initial_edit_instruction_for_collaboration(collaboration, decision.edit);
    decision
}

fn bootstrap_review_initial_action(
    decision: &InitialActionDecision,
) -> Option<InitialActionDecision> {
    if matches!(
        decision.action,
        InitialAction::Workspace {
            action: WorkspaceAction::Diff { .. }
        }
    ) {
        return None;
    }

    Some(InitialActionDecision {
        action: InitialAction::Workspace {
            action: WorkspaceAction::Diff { path: None },
        },
        rationale: "review mode requires local diff evidence before synthesis".to_string(),
        answer: None,
        edit: InitialEditInstruction::default(),
        grounding: Some(GroundingRequirement {
            domain: GroundingDomain::Repository,
            reason: Some("review mode starts by inspecting local changes".to_string()),
        }),
    })
}

fn collaboration_boundary_for_action(
    collaboration: &CollaborationModeResult,
    action: &PlannerAction,
    edit: &InitialEditInstruction,
) -> Option<CollaborationBoundaryOutcome> {
    let PlannerAction::Workspace { action } = action else {
        return None;
    };
    if collaboration.active.mutation_posture.allows_mutation() || !action.is_mutating() {
        return None;
    }

    let detail = format!(
        "{} mode blocked mutating action `{}` and kept the harness read-only.",
        collaboration.active.mode.label(),
        action.summary()
    );
    match collaboration.active.mode {
        CollaborationMode::Planning => {
            let clarification = planning_mode_mutation_clarification_request(
                action,
                edit.candidate_files.as_slice(),
            );
            Some(CollaborationBoundaryOutcome {
                reason: "collaboration-mode-blocked".to_string(),
                response: AuthoredResponse::from_plain_text(
                    ResponseMode::DirectAnswer,
                    &render_structured_clarification_request(&clarification, &detail),
                ),
                summary: detail.clone(),
                clarification: Some(clarification.requested(detail)),
            })
        }
        CollaborationMode::Review => Some(CollaborationBoundaryOutcome {
            reason: "collaboration-mode-blocked".to_string(),
            response: AuthoredResponse::from_plain_text(
                ResponseMode::DirectAnswer,
                &format!(
                    "Review mode is read-only, so I stopped before `{}`.\n\nIf you want changes applied, rerun this request in execution mode.",
                    action.summary()
                ),
            ),
            summary: detail,
            clarification: None,
        }),
        CollaborationMode::Execution => None,
    }
}

#[derive(Clone)]
struct CollaborationBoundaryOutcome {
    reason: String,
    response: AuthoredResponse,
    summary: String,
    clarification: Option<crate::domain::model::StructuredClarificationResult>,
}

fn planning_mode_mutation_clarification_request(
    action: &WorkspaceAction,
    candidate_files: &[String],
) -> StructuredClarificationRequest {
    let mut prompt = format!(
        "Planning mode is read-only, so I stopped before `{}`.",
        action.summary()
    );
    if !candidate_files.is_empty() {
        prompt.push_str("\nLikely targets: ");
        prompt.push_str(&candidate_files.join(", "));
    }
    StructuredClarificationRequest::new(
        "planning-mode-clarification",
        StructuredClarificationKind::Approval,
        prompt,
        vec![
            StructuredClarificationOption::new(
                "stay_in_planning",
                "Stay in planning",
                "Keep the turn read-only and return a plan or review of the required changes.",
            ),
            StructuredClarificationOption::new(
                "switch_to_execution",
                "Switch to execution",
                "Rerun in execution mode so Paddles can apply the requested change.",
            ),
        ],
        false,
    )
}

fn render_structured_clarification_request(
    request: &StructuredClarificationRequest,
    detail: &str,
) -> String {
    let mut lines = vec![
        "Need clarification before mutating.".to_string(),
        request.prompt.clone(),
        detail.to_string(),
        "Options:".to_string(),
    ];
    lines.extend(
        request
            .options
            .iter()
            .map(|option| format!("- {}: {}", option.option_id, option.description.trim())),
    );
    lines.join("\n")
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

fn planner_runtime_notes(
    gatherer: Option<&Arc<dyn ContextGatherer>>,
    specialist_notes: &[String],
    collaboration: &CollaborationModeResult,
) -> Vec<String> {
    let mut notes = collaboration_runtime_notes(collaboration);
    notes.extend(specialist_notes.iter().cloned());
    let Some(gatherer) = gatherer else {
        return notes;
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
        return notes;
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

    notes.push(format!(
        "Workspace retrieval readiness: bm25={}, vector={}. {}",
        gatherer_readiness_label(&lexical),
        gatherer_readiness_label(&vector),
        guidance
    ));
    notes
}

struct PlannerExecutionContractContext<'a> {
    workspace_capability_surface: &'a WorkspaceCapabilitySurface,
    execution_hands: &'a [ExecutionHandDiagnostic],
    governance_profile: Option<&'a ExecutionGovernanceProfile>,
    external_capabilities: &'a [ExternalCapabilityDescriptor],
    gatherer: Option<&'a Arc<dyn ContextGatherer>>,
    collaboration: &'a CollaborationModeResult,
    instruction_frame: Option<&'a InstructionFrame>,
    grounding: Option<&'a GroundingRequirement>,
}

fn build_planner_execution_contract(
    context: PlannerExecutionContractContext<'_>,
) -> PlannerExecutionContract {
    let PlannerExecutionContractContext {
        workspace_capability_surface,
        execution_hands,
        governance_profile,
        external_capabilities,
        gatherer,
        collaboration,
        instruction_frame,
        grounding,
    } = context;
    let mut capability_manifest = workspace_capability_surface
        .actions
        .iter()
        .map(|capability| format_workspace_action_capability(capability, collaboration))
        .collect::<Vec<_>>();
    capability_manifest.extend(
        workspace_capability_surface
            .tools
            .iter()
            .map(format_workspace_tool_capability),
    );
    capability_manifest.extend(
        workspace_capability_surface
            .notes
            .iter()
            .map(|note| format!("workspace note: {note}")),
    );
    capability_manifest.extend(
        execution_hands
            .iter()
            .map(format_execution_hand_capability_line),
    );
    capability_manifest.extend(retrieval_capability_lines(gatherer));
    capability_manifest.extend(external_capabilities.iter().map(|descriptor| {
        format!(
            "external capability {}: {}",
            descriptor.id,
            format_external_capability_catalog_entry(descriptor)
        )
    }));
    capability_manifest.push(match governance_profile {
        Some(profile) => {
            format!(
                "execution governance: {} {}",
                profile.summary(),
                profile.detail()
            )
        }
        None => "execution governance: unavailable; mutating or networked actions may fail closed"
            .to_string(),
    });

    let mut completion_contract = vec![
        "Choose only actions supported by the capability manifest. If a capability is blocked or unavailable, choose a different bounded action."
            .to_string(),
        "When a task depends on a local program that is not already observed in the capability manifest, choose a bounded single-step probe such as `inspect` `command -v <tool>` before depending on it."
            .to_string(),
        "`inspect` is only for single read-only probes. Do not chain commands or use redirection; use `shell` for broader governed workspace command execution."
            .to_string(),
    ];

    match collaboration.active.mode {
        CollaborationMode::Planning => completion_contract.push(
            "Planning mode is read-only. Do not choose mutating workspace actions or shell commands that could change the repository."
                .to_string(),
        ),
        CollaborationMode::Review => completion_contract.push(
            "Review mode is read-only. Inspect local evidence and stop at findings; do not choose mutating workspace actions."
                .to_string(),
        ),
        CollaborationMode::Execution => completion_contract.push(
            "Execution mode allows mutating workspace actions, but they still run through execution governance and may be denied or downgraded."
                .to_string(),
        ),
    }

    if let Some(frame) = instruction_frame {
        if frame.requires_applied_edit() {
            let mut line =
                "The turn is not complete until an applied workspace edit succeeds.".to_string();
            if let Some(candidates) = frame.candidate_summary() {
                line.push_str(&format!(" Current candidate files: {candidates}."));
            }
            completion_contract.push(line);
        }
        if frame.requires_applied_commit() {
            completion_contract.push(
                "The turn is not complete until the requested git commit has been recorded in the workspace."
                    .to_string(),
            );
        }
    }

    if let Some(grounding) = grounding {
        let mut line = format!(
            "Do not stop with a final answer until {} evidence has been assembled.",
            grounding_domain_label(grounding.domain)
        );
        if let Some(reason) = grounding
            .reason
            .as_deref()
            .filter(|reason| !reason.trim().is_empty())
        {
            line.push_str(&format!(" Reason: {}.", reason.trim()));
        }
        completion_contract.push(line);
    }

    PlannerExecutionContract {
        capability_manifest,
        completion_contract,
    }
}

fn format_workspace_action_capability(
    capability: &WorkspaceActionCapability,
    collaboration: &CollaborationModeResult,
) -> String {
    if capability.mutating && !collaboration.active.mutation_posture.allows_mutation() {
        return format!(
            "workspace action {}: blocked by {} mode read-only boundary — {}",
            capability.action,
            collaboration.active.mode.label(),
            capability.summary
        );
    }

    let posture = if capability.mutating {
        "mutating"
    } else {
        "read-only"
    };
    format!(
        "workspace action {}: available ({posture}) — {}",
        capability.action, capability.summary
    )
}

fn format_workspace_tool_capability(tool: &WorkspaceToolCapability) -> String {
    match tool.suggested_probe.as_ref() {
        Some(action) => format!(
            "workspace tool observation {}: {} — re-probe via {}",
            tool.tool,
            tool.summary,
            action.summary()
        ),
        None => format!("workspace tool observation {}: {}", tool.tool, tool.summary),
    }
}

fn format_execution_hand_capability_line(diagnostic: &ExecutionHandDiagnostic) -> String {
    let operations = diagnostic
        .supported_operations
        .iter()
        .map(|operation| operation.label())
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "execution hand {}: {}, authority={}, operations=[{}] — {}",
        diagnostic.hand.label(),
        diagnostic.phase.label(),
        diagnostic.authority.label(),
        operations,
        diagnostic.summary
    )
}

fn retrieval_capability_lines(gatherer: Option<&Arc<dyn ContextGatherer>>) -> Vec<String> {
    let Some(gatherer) = gatherer else {
        return vec![
            "search/refine via bm25: unavailable — no gatherer is configured".to_string(),
            "search/refine via vector: unavailable — no gatherer is configured".to_string(),
        ];
    };

    let lexical = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Lexical),
    );
    let vector = gatherer.capability_for_planning(
        &PlannerConfig::default().with_retrieval_strategy(RetrievalStrategy::Vector),
    );

    vec![
        format!(
            "search/refine via bm25: {}",
            format_gatherer_capability(&lexical)
        ),
        format!(
            "search/refine via vector: {}",
            format_gatherer_capability(&vector)
        ),
    ]
}

fn grounding_domain_label(domain: GroundingDomain) -> &'static str {
    match domain {
        GroundingDomain::Repository => "repository",
        GroundingDomain::External => "external",
        GroundingDomain::Mixed => "mixed repository and external",
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

fn enrich_interpretation_with_workspace_capability_surface(
    mut context: InterpretationContext,
    surface: &WorkspaceCapabilitySurface,
) -> InterpretationContext {
    if surface.actions.is_empty() && surface.tools.is_empty() && surface.notes.is_empty() {
        return context;
    }

    let source = "paddles-harness";
    let action_labels = surface
        .actions
        .iter()
        .map(|capability| capability.action.as_str())
        .collect::<Vec<_>>();
    let tool_labels = surface
        .tools
        .iter()
        .map(|capability| capability.tool.as_str())
        .collect::<Vec<_>>();
    let mut summary_parts = Vec::new();
    if !action_labels.is_empty() {
        summary_parts.push(format!("Workspace actions: {}.", action_labels.join(", ")));
    }
    if !tool_labels.is_empty() {
        summary_parts.push(format!("Observed local tools: {}.", tool_labels.join(", ")));
    }
    if !surface.notes.is_empty() {
        summary_parts.extend(surface.notes.iter().cloned());
    }
    let capability_summary = format!(
        "Paddles can execute local workspace actions through the harness. {}",
        summary_parts.join(" ")
    );
    if context.summary.trim().is_empty() {
        context.summary = capability_summary.clone();
    } else if !context
        .summary
        .contains("Paddles can execute local workspace actions through the harness")
    {
        context.summary = format!("{}\n\n{}", context.summary.trim(), capability_summary);
    }

    if surface
        .actions
        .iter()
        .any(|capability| capability.action == "inspect")
    {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::Inspect {
                    command: "command -v <tool>".to_string(),
                },
                note: "When the task depends on a local program, probe that exact tool first and let the harness cache the observation for later planning steps.".to_string(),
            },
        );
    }

    append_interpretation_procedure(
        &mut context,
        InterpretationProcedure {
            source: source.to_string(),
            label: "Probe Required Local Tools".to_string(),
            purpose: "Decide which local program the task actually needs, probe that exact tool through the harness, and then reuse cached observations instead of relying on a prebaked whitelist.".to_string(),
            steps: dynamic_tool_probe_procedure_steps(surface),
        },
    );

    context
}

fn enrich_interpretation_with_external_capabilities(
    mut context: InterpretationContext,
    descriptors: &[ExternalCapabilityDescriptor],
) -> InterpretationContext {
    if descriptors.is_empty() {
        return context;
    }

    let source = "external-capability-catalog";
    let capability_summary = descriptors
        .iter()
        .map(format_external_capability_catalog_entry)
        .collect::<Vec<_>>()
        .join("; ");
    let summary = format!(
        "Paddles can route external capability fabrics through the recursive harness. Catalog: {capability_summary}."
    );
    if context.summary.trim().is_empty() {
        context.summary = summary.clone();
    } else if !context.summary.contains("external capability fabrics") {
        context.summary = format!("{}\n\n{}", context.summary.trim(), summary);
    }

    for descriptor in descriptors {
        append_interpretation_tool_hint(
            &mut context,
            InterpretationToolHint {
                source: source.to_string(),
                action: WorkspaceAction::ExternalCapability {
                    invocation: sample_external_capability_invocation(descriptor),
                },
                note: format!(
                    "{}. {}.",
                    descriptor.summary,
                    format_external_capability_catalog_entry(descriptor)
                ),
            },
        );
    }

    append_interpretation_procedure(
        &mut context,
        InterpretationProcedure {
            source: source.to_string(),
            label: "Ground With External Capabilities".to_string(),
            purpose: "Use the typed external-capability lane when the turn needs current, connector-backed, or MCP-mediated evidence, and preserve degraded availability or governance outcomes as evidence.".to_string(),
            steps: descriptors
                .iter()
                .enumerate()
                .map(|(index, descriptor)| InterpretationProcedureStep {
                    index,
                    action: WorkspaceAction::ExternalCapability {
                        invocation: sample_external_capability_invocation(descriptor),
                    },
                    note: format!(
                        "{} [{}]",
                        descriptor.summary,
                        format_external_capability_catalog_entry(descriptor)
                    ),
                })
                .collect(),
        },
    );

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

fn sample_external_capability_invocation(
    descriptor: &ExternalCapabilityDescriptor,
) -> ExternalCapabilityInvocation {
    let payload = match descriptor.kind {
        crate::domain::model::ExternalCapabilityKind::WebSearch => {
            serde_json::json!({ "query": "current topic" })
        }
        crate::domain::model::ExternalCapabilityKind::McpTool => {
            serde_json::json!({ "tool": "tool_name", "arguments": {} })
        }
        crate::domain::model::ExternalCapabilityKind::ConnectorApp => {
            serde_json::json!({ "app": "app_name", "action": "operation", "arguments": {} })
        }
    };
    ExternalCapabilityInvocation::new(
        descriptor.id.clone(),
        format!("gather {} evidence", descriptor.label.to_lowercase()),
        payload,
    )
}

fn dynamic_tool_probe_procedure_steps(
    surface: &WorkspaceCapabilitySurface,
) -> Vec<InterpretationProcedureStep> {
    let mut steps = Vec::new();
    if surface
        .actions
        .iter()
        .any(|capability| capability.action == "inspect")
    {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "command -v <tool>".to_string(),
            },
            note: "Probe the exact tool you think the task needs and let the harness cache the result."
                .to_string(),
        });
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Inspect {
                command: "<tool> --version".to_string(),
            },
            note: "Use a narrow read-only probe to confirm syntax or version before depending on the tool."
                .to_string(),
        });
    }
    if surface
        .actions
        .iter()
        .any(|capability| capability.action == "shell")
    {
        steps.push(InterpretationProcedureStep {
            index: steps.len(),
            action: WorkspaceAction::Shell {
                command: "<tool> <args>".to_string(),
            },
            note: "Once the required tool is justified and available, run the narrowest governed command that advances the task."
                .to_string(),
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
    Deliberation,
}

impl SteeringReviewKind {
    fn stage(self) -> &'static str {
        match self {
            Self::Evidence => "premise-challenge",
            Self::Execution => "action-bias",
            Self::Deliberation => "deliberation-signals",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SteeringReviewNote {
    kind: SteeringReviewKind,
    note: String,
}

struct DecisionReviewFrame<'a> {
    deliberation_signals: &'a DeliberationSignals,
    workspace_root: &'a Path,
    trace: Arc<StructuredTurnTrace>,
}

async fn review_decision_under_signals(
    prompt: &str,
    context: &PlannerLoopContext,
    budget: &PlannerBudget,
    loop_state: &PlannerLoopState,
    decision: RecursivePlannerDecision,
    frame: DecisionReviewFrame<'_>,
) -> Result<RecursivePlannerDecision> {
    let mut review_loop_state = loop_state.clone();
    sync_deliberation_signal_note(&mut review_loop_state, frame.deliberation_signals);
    if context.initial_edit.known_edit {
        let likely_targets = likely_action_bias_targets(loop_state, frame.workspace_root, 3);
        if let Some(resolution) = resolve_known_edit_target(
            &context.entity_resolver,
            frame.workspace_root,
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

    let steering_notes = collect_steering_review_notes(
        context,
        &review_loop_state,
        &decision,
        frame.workspace_root,
        frame.deliberation_signals,
    );
    if steering_notes.is_empty() {
        return Ok(decision);
    }

    for note in &steering_notes {
        review_loop_state.notes.push(note.note.clone());
    }

    let request = PlannerRequest::new(
        prompt,
        frame.workspace_root.to_path_buf(),
        context.interpretation.clone(),
        budget.clone(),
    )
    .with_collaboration(context.collaboration.clone())
    .with_recent_turns(context.recent_turns.clone())
    .with_recent_thread_summary(context.recent_thread_summary.clone())
    .with_runtime_notes(planner_runtime_notes(
        context.gatherer.as_ref(),
        &context.specialist_runtime_notes,
        &context.collaboration,
    ))
    .with_execution_contract(build_planner_execution_contract(
        PlannerExecutionContractContext {
            workspace_capability_surface: &context.workspace_capability_surface,
            execution_hands: &context.execution_hands,
            governance_profile: context.governance_profile.as_ref(),
            external_capabilities: &context.external_capabilities,
            gatherer: context.gatherer.as_ref(),
            collaboration: &context.collaboration,
            instruction_frame: context.instruction_frame.as_ref(),
            grounding: context.grounding.as_ref(),
        },
    ))
    .with_loop_state(review_loop_state)
    .with_resolver(context.resolver.clone())
    .with_entity_resolver(context.entity_resolver.clone());

    let reviewed = context
        .planner_engine
        .select_next_action(&request, frame.trace.clone() as Arc<dyn TurnEventSink>)
        .await?;

    if steering_review_failed_closed(&reviewed) {
        return Ok(decision);
    }

    if reviewed != decision {
        let stage = steering_review_stage(&steering_notes, &reviewed);
        frame.trace.emit(TurnEvent::Fallback {
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
    deliberation_signals: &DeliberationSignals,
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

    if should_apply_deliberation_review(deliberation_signals, decision) {
        notes.push(SteeringReviewNote {
            kind: SteeringReviewKind::Deliberation,
            note: format_deliberation_review_note(decision, deliberation_signals),
        });
    }

    notes
}

fn continuation_requires_tool_follow_up(signals: &DeliberationSignals) -> bool {
    matches!(
        signals.continuation,
        DeliberationSignal::Present(DeliberationContinuation {
            tool_results_required: true,
            ..
        })
    )
}

fn has_opaque_deliberation_hints(signals: &DeliberationSignals) -> bool {
    !matches!(&signals.uncertainty, DeliberationSignal::None)
        || !matches!(&signals.evidence_gaps, DeliberationSignal::None)
        || !matches!(&signals.branch_candidates, DeliberationSignal::None)
        || !matches!(&signals.stop_confidence, DeliberationSignal::None)
        || !matches!(&signals.risk_hints, DeliberationSignal::None)
}

fn format_deliberation_signal_note(signals: &DeliberationSignals) -> Option<String> {
    let mut lines = Vec::new();
    if continuation_requires_tool_follow_up(signals) {
        lines.push(
            "Deliberation signals: prior provider turn left reusable continuation state. Judge the current tool-result path before branching, refining, or stopping."
                .to_string(),
        );
    }
    if has_opaque_deliberation_hints(signals) {
        lines.push(
            "Opaque provider hints are present but intentionally not surfaced directly. Prefer one bounded continuation step over speculative branching."
                .to_string(),
        );
    }
    (!lines.is_empty()).then(|| lines.join("\n"))
}

fn deliberation_confidence_label(confidence: DeliberationConfidence) -> &'static str {
    match confidence {
        DeliberationConfidence::Low => "low",
        DeliberationConfidence::Medium => "medium",
        DeliberationConfidence::High => "high",
    }
}

fn summarize_deliberation_signal_values(values: &[String]) -> Option<String> {
    let values = values
        .iter()
        .map(|value| trim_for_planner(value, 48))
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.is_empty() {
        return None;
    }

    let preview = values.iter().take(2).cloned().collect::<Vec<_>>();
    let mut summary = preview.join(", ");
    if values.len() > preview.len() {
        summary.push_str(&format!(" (+{} more)", values.len() - preview.len()));
    }
    Some(summary)
}

fn summarize_deliberation_signals(signals: &DeliberationSignals) -> Option<String> {
    let mut summaries = Vec::new();
    if continuation_requires_tool_follow_up(signals) {
        summaries.push("continuation=tool_follow_up".to_string());
    }

    match &signals.uncertainty {
        DeliberationSignal::Present(confidence) => summaries.push(format!(
            "uncertainty={}",
            deliberation_confidence_label(*confidence)
        )),
        DeliberationSignal::Unknown => summaries.push("uncertainty=opaque".to_string()),
        DeliberationSignal::None => {}
    }

    match &signals.evidence_gaps {
        DeliberationSignal::Present(gaps) => {
            if let Some(gaps) = summarize_deliberation_signal_values(gaps) {
                summaries.push(format!("evidence_gaps={gaps}"));
            }
        }
        DeliberationSignal::Unknown => summaries.push("evidence_gaps=opaque".to_string()),
        DeliberationSignal::None => {}
    }

    match &signals.branch_candidates {
        DeliberationSignal::Present(candidates) => {
            if let Some(candidates) = summarize_deliberation_signal_values(candidates) {
                summaries.push(format!("branch_candidates={candidates}"));
            }
        }
        DeliberationSignal::Unknown => summaries.push("branch_candidates=opaque".to_string()),
        DeliberationSignal::None => {}
    }

    match &signals.stop_confidence {
        DeliberationSignal::Present(confidence) => summaries.push(format!(
            "stop_confidence={}",
            deliberation_confidence_label(*confidence)
        )),
        DeliberationSignal::Unknown => summaries.push("stop_confidence=opaque".to_string()),
        DeliberationSignal::None => {}
    }

    match &signals.risk_hints {
        DeliberationSignal::Present(risks) => {
            if let Some(risks) = summarize_deliberation_signal_values(risks) {
                summaries.push(format!("risk_hints={risks}"));
            }
        }
        DeliberationSignal::Unknown => summaries.push("risk_hints=opaque".to_string()),
        DeliberationSignal::None => {}
    }

    (!summaries.is_empty()).then(|| summaries.join("; "))
}

fn summarize_rationale_evidence_sources(evidence_items: &[EvidenceItem]) -> Option<String> {
    let mut unique_sources = Vec::new();
    for item in evidence_items {
        let source = trim_for_planner(&item.source, 48);
        if source.is_empty() || unique_sources.contains(&source) {
            continue;
        }
        unique_sources.push(source);
    }

    match unique_sources.as_slice() {
        [] => None,
        [only] => Some(only.clone()),
        [first, second] => Some(format!("{first} and {second}")),
        [first, second, rest @ ..] => Some(format!(
            "{first}, {second}, and {} other source(s)",
            rest.len()
        )),
    }
}

fn compile_initial_paddles_rationale(
    action: &InitialAction,
    signals: &DeliberationSignals,
) -> (String, Option<String>) {
    let signal_summary = summarize_deliberation_signals(signals);
    let rationale = match action {
        InitialAction::Answer | InitialAction::Stop { .. } => format!(
            "Paddles chose `{}` because the turn could complete without additional evidence before responding.",
            action.summary()
        ),
        InitialAction::Workspace { .. }
        | InitialAction::Refine { .. }
        | InitialAction::Branch { .. } => format!(
            "Paddles chose `{}` as the first bounded step to gather or act on the most relevant evidence.",
            action.summary()
        ),
    };
    let rationale = if continuation_requires_tool_follow_up(signals) {
        format!(
            "{rationale} Normalized deliberation signals preserved the active tool-result path."
        )
    } else if has_opaque_deliberation_hints(signals) {
        format!(
            "{rationale} Normalized deliberation signals kept the first step conservative and grounded."
        )
    } else {
        rationale
    };

    (rationale, signal_summary)
}

fn compile_recursive_paddles_rationale(
    action: &PlannerAction,
    evidence_items: &[EvidenceItem],
    signals: &DeliberationSignals,
) -> (String, Option<String>) {
    let signal_summary = summarize_deliberation_signals(signals);
    let evidence_summary = summarize_rationale_evidence_sources(evidence_items);
    let base = match (action, evidence_summary.as_deref()) {
        (PlannerAction::Stop { .. }, Some(evidence_summary)) => format!(
            "Paddles chose `{}` because evidence from {evidence_summary} made the current path sufficient.",
            action.summary()
        ),
        (PlannerAction::Stop { .. }, None) => format!(
            "Paddles chose `{}` because the turn could close without additional evidence.",
            action.summary()
        ),
        (_, Some(evidence_summary)) => format!(
            "Paddles chose `{}` because evidence from {evidence_summary} narrowed the next bounded step.",
            action.summary()
        ),
        (_, None) => format!(
            "Paddles chose `{}` as the next bounded step to gather or act on the most relevant evidence.",
            action.summary()
        ),
    };
    let rationale = if continuation_requires_tool_follow_up(signals) {
        format!("{base} Normalized deliberation signals preserved the active tool-result path.")
    } else if has_opaque_deliberation_hints(signals) {
        format!(
            "{base} Normalized deliberation signals kept the follow-up conservative and grounded."
        )
    } else {
        base
    };

    (rationale, signal_summary)
}

fn sync_deliberation_signal_note(
    loop_state: &mut PlannerLoopState,
    deliberation_signals: &DeliberationSignals,
) {
    const DELIBERATION_SIGNAL_NOTE_PREFIX: &str = "Deliberation signals:";
    loop_state
        .notes
        .retain(|note| !note.starts_with(DELIBERATION_SIGNAL_NOTE_PREFIX));

    if let Some(note) = format_deliberation_signal_note(deliberation_signals) {
        loop_state.notes.push(note);
    }
}

fn should_apply_deliberation_review(
    deliberation_signals: &DeliberationSignals,
    decision: &RecursivePlannerDecision,
) -> bool {
    if continuation_requires_tool_follow_up(deliberation_signals)
        && matches!(
            decision.action,
            PlannerAction::Stop { .. }
                | PlannerAction::Branch { .. }
                | PlannerAction::Refine { .. }
        )
    {
        return true;
    }

    has_opaque_deliberation_hints(deliberation_signals) && decision.action.is_terminal()
}

fn format_deliberation_review_note(
    decision: &RecursivePlannerDecision,
    deliberation_signals: &DeliberationSignals,
) -> String {
    let mut lines = vec![
        "Steering review [deliberation-signals]".to_string(),
        format!(
            "Proposed action under review: {}",
            decision.action.summary()
        ),
    ];

    if continuation_requires_tool_follow_up(deliberation_signals) {
        lines.push(
            "The previous provider turn left reusable continuation state that expects the current tool result or evidence to be judged on the same path.".to_string(),
        );
        lines.push(
            "Continue the active path with one bounded action before branching, refining, or stopping unless the current evidence is already decisive."
                .to_string(),
        );
    }

    if has_opaque_deliberation_hints(deliberation_signals) {
        lines.push(
            "Opaque provider hints are present but intentionally hidden from canonical output. Stay conservative and grounded.".to_string(),
        );
    }

    lines.join("\n")
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
        | WorkspaceAction::Shell { .. }
        | WorkspaceAction::ExternalCapability { .. } => None,
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
    } else if notes
        .iter()
        .any(|note| note.kind == SteeringReviewKind::Deliberation)
    {
        SteeringReviewKind::Deliberation.stage()
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

struct GovernedPlannerCommandSummary {
    summary: String,
    command_succeeded: bool,
    governance_request: ExecutionPermissionRequest,
    governance_outcome: ExecutionGovernanceOutcome,
}

fn run_planner_inspect_command(
    workspace_root: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<GovernedPlannerCommandSummary> {
    validate_inspect_command(command)?;
    let output = run_background_terminal_command_with_execution_hand_registry(
        workspace_root,
        command,
        "inspect",
        call_id,
        event_sink,
        execution_hand_registry,
    )?;
    match output {
        GovernedTerminalCommandResult::Executed {
            output,
            governance_request,
            governance_outcome,
        } => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let rendered = if stderr.trim().is_empty() {
                stdout
            } else {
                format!("{stdout}\n{stderr}")
            };

            Ok(GovernedPlannerCommandSummary {
                summary: trim_for_planner(&rendered, 1_200),
                command_succeeded: output.status.success(),
                governance_request,
                governance_outcome,
            })
        }
        GovernedTerminalCommandResult::Blocked {
            governance_request,
            governance_outcome,
        } => Ok(GovernedPlannerCommandSummary {
            summary: summarize_governance_outcome(&governance_outcome),
            command_succeeded: false,
            governance_request,
            governance_outcome,
        }),
    }
}

fn run_planner_shell_command(
    workspace_root: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<GovernedPlannerCommandSummary> {
    let output = run_background_terminal_command_with_execution_hand_registry(
        workspace_root,
        command,
        "shell",
        call_id,
        event_sink,
        execution_hand_registry,
    )?;
    match output {
        GovernedTerminalCommandResult::Executed {
            output,
            governance_request,
            governance_outcome,
        } => {
            let summary = format_command_output_summary(command, &output);
            Ok(GovernedPlannerCommandSummary {
                summary,
                command_succeeded: output.status.success(),
                governance_request,
                governance_outcome,
            })
        }
        GovernedTerminalCommandResult::Blocked {
            governance_request,
            governance_outcome,
        } => Ok(GovernedPlannerCommandSummary {
            summary: summarize_governance_outcome(&governance_outcome),
            command_succeeded: false,
            governance_request,
            governance_outcome,
        }),
    }
}

struct ExternalCapabilityExecutionFrame<'a> {
    rationale: &'a str,
    evidence_limit: usize,
    evidence_items: &'a mut Vec<EvidenceItem>,
    call_id: &'a str,
    event_sink: &'a dyn TurnEventSink,
}

fn execute_external_capability_action(
    broker: Arc<dyn ExternalCapabilityBroker>,
    governance_profile: &crate::domain::model::ExecutionGovernanceProfile,
    invocation: &ExternalCapabilityInvocation,
    frame: ExternalCapabilityExecutionFrame<'_>,
) -> String {
    let Some(descriptor) = broker.descriptor(&invocation.capability_id) else {
        let summary = format_external_capability_outcome(
            None,
            invocation,
            ExternalCapabilityResultStatus::Unavailable,
            "External capability unavailable".to_string(),
            format!(
                "External capability `{}` is unknown to this runtime",
                invocation.capability_id
            ),
            &[],
        );
        frame.event_sink.emit(TurnEvent::ToolFinished {
            call_id: frame.call_id.to_string(),
            tool_name: "external_capability".to_string(),
            summary: summary.clone(),
        });
        append_evidence_item(
            frame.evidence_items,
            EvidenceItem {
                source: format!("external_capability:{}", invocation.capability_id),
                snippet: trim_for_planner(&summary, 1_200),
                rationale: frame.rationale.to_string(),
                rank: 0,
            },
            frame.evidence_limit,
        );
        return summary;
    };

    let governance_request = ExecutionPermissionRequest::new(
        descriptor.hand,
        descriptor.governance_requirement(format!(
            "invoke external capability `{}` for {}",
            descriptor.id, invocation.purpose
        )),
    );
    let governance_outcome =
        ExecutionPermissionGate::evaluate(Some(governance_profile), &governance_request);
    emit_execution_governance_decision(
        frame.event_sink,
        Some(frame.call_id),
        Some("external_capability"),
        governance_request.clone(),
        governance_outcome.clone(),
    );

    if governance_outcome.kind != crate::domain::model::ExecutionGovernanceOutcomeKind::Allowed {
        let summary = format_external_capability_outcome(
            Some(&descriptor),
            invocation,
            ExternalCapabilityResultStatus::Denied,
            "External capability denied".to_string(),
            summarize_governance_outcome(&governance_outcome),
            &[],
        );
        frame.event_sink.emit(TurnEvent::ToolFinished {
            call_id: frame.call_id.to_string(),
            tool_name: "external_capability".to_string(),
            summary: summary.clone(),
        });
        append_evidence_item(
            frame.evidence_items,
            EvidenceItem {
                source: format!("external_capability:{}", descriptor.id),
                snippet: trim_for_planner(&summary, 1_200),
                rationale: frame.rationale.to_string(),
                rank: 0,
            },
            frame.evidence_limit,
        );
        return summary;
    }

    match broker.invoke(invocation) {
        Ok(result) => {
            let summary = summarize_external_capability_result(&result);
            frame.event_sink.emit(TurnEvent::ToolFinished {
                call_id: frame.call_id.to_string(),
                tool_name: "external_capability".to_string(),
                summary: summary.clone(),
            });
            for item in external_capability_result_evidence_items(&result, frame.rationale) {
                append_evidence_item(frame.evidence_items, item, frame.evidence_limit);
            }
            summary
        }
        Err(err) => {
            let summary = format_external_capability_outcome(
                Some(&descriptor),
                invocation,
                ExternalCapabilityResultStatus::Failed,
                format!("{} failed", descriptor.label),
                format!("External capability `{}` failed: {err:#}", descriptor.id),
                &[],
            );
            frame.event_sink.emit(TurnEvent::ToolFinished {
                call_id: frame.call_id.to_string(),
                tool_name: "external_capability".to_string(),
                summary: summary.clone(),
            });
            append_evidence_item(
                frame.evidence_items,
                EvidenceItem {
                    source: format!("external_capability:{}", descriptor.id),
                    snippet: trim_for_planner(&summary, 1_200),
                    rationale: frame.rationale.to_string(),
                    rank: 0,
                },
                frame.evidence_limit,
            );
            summary
        }
    }
}

fn emit_execution_governance_decision(
    event_sink: &dyn TurnEventSink,
    call_id: Option<&str>,
    tool_name: Option<&str>,
    governance_request: ExecutionPermissionRequest,
    governance_outcome: ExecutionGovernanceOutcome,
) {
    event_sink.emit(TurnEvent::ExecutionGovernanceDecisionRecorded {
        decision: ExecutionGovernanceDecision::new(
            call_id.map(str::to_string),
            tool_name.map(str::to_string),
            governance_request,
            governance_outcome,
        ),
    });
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
        anyhow::bail!(
            "planner inspect command must be a single read-only probe without chaining or redirection"
        );
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

fn next_task_sequence_from_trace_recorder(trace_recorder: &dyn TraceRecorder) -> u64 {
    trace_recorder
        .task_ids()
        .iter()
        .filter_map(|task_id| parse_generated_sequence(task_id.as_str(), "task-"))
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(1)
}

fn compare_task_ids_desc(left: &TaskTraceId, right: &TaskTraceId) -> std::cmp::Ordering {
    parse_generated_sequence(right.as_str(), "task-")
        .cmp(&parse_generated_sequence(left.as_str(), "task-"))
        .then_with(|| right.as_str().cmp(left.as_str()))
}

fn parse_generated_sequence(value: &str, marker: &str) -> Option<u64> {
    value.rsplit_once(marker)?.1.parse().ok()
}

fn rehydrate_conversation_session(replay: &TraceReplay) -> ConversationSession {
    let session = ConversationSession::new(replay.task_id.clone());
    let replay_view = ConversationReplayView::from_trace_replay(replay);
    let next_turn_sequence = replay
        .records
        .iter()
        .filter_map(|record| parse_generated_sequence(record.lineage.turn_id.as_str(), ".turn-"))
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(1);
    let next_candidate_sequence = replay
        .records
        .iter()
        .filter_map(|record| match &record.kind {
            TraceRecordKind::ThreadCandidateCaptured(candidate) => {
                Some(candidate.captured_sequence)
            }
            _ => None,
        })
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(1);
    let next_branch_sequence = replay
        .records
        .iter()
        .filter_map(|record| match &record.kind {
            TraceRecordKind::PlannerBranchDeclared(branch) => {
                parse_generated_sequence(branch.branch_id.as_str(), ".thread-")
            }
            _ => None,
        })
        .max()
        .map(|sequence| sequence.saturating_add(1))
        .unwrap_or(1);
    let mut root_last_record_id = None;
    let mut branch_last_record_ids = HashMap::new();
    for record in &replay.records {
        if let Some(branch_id) = record.lineage.branch_id.clone() {
            branch_last_record_ids.insert(branch_id, record.record_id.clone());
        } else {
            root_last_record_id = Some(record.record_id.clone());
        }
    }

    let active_thread = replay_view
        .threads
        .iter()
        .find(|thread| thread.status == ConversationThreadStatus::Active)
        .map(|thread| thread.thread_ref.clone())
        .unwrap_or(ConversationThreadRef::Mainline);
    let threads = replay_view
        .threads
        .into_iter()
        .map(|thread| (thread.thread_ref.stable_id(), thread))
        .collect::<HashMap<_, _>>();
    let state = session.state();
    let mut state = state.lock().expect("conversation session lock");
    state.next_turn_sequence = next_turn_sequence;
    state.next_candidate_sequence = next_candidate_sequence;
    state.next_branch_sequence = next_branch_sequence;
    state.root_started = !replay.records.is_empty();
    state.root_last_record_id = root_last_record_id;
    state.branch_last_record_ids = branch_last_record_ids;
    state.active_thread = active_thread;
    if !threads.is_empty() {
        state.threads = threads;
    }
    drop(state);
    session
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

fn summarize_external_capability_result(
    result: &crate::domain::model::ExternalCapabilityResult,
) -> String {
    format_external_capability_outcome(
        Some(&result.descriptor),
        &result.invocation,
        result.status,
        result.summary.clone(),
        result.detail.clone(),
        &result.sources,
    )
}

fn external_capability_result_evidence_items(
    result: &crate::domain::model::ExternalCapabilityResult,
    rationale: &str,
) -> Vec<EvidenceItem> {
    let mut items = vec![EvidenceItem {
        source: format!("external_capability:{}", result.descriptor.id),
        snippet: trim_for_planner(
            &format!(
                "status={}\nsummary={}\ndetail={}",
                result.status.label(),
                result.summary,
                result.detail
            ),
            1_200,
        ),
        rationale: rationale.to_string(),
        rank: 0,
    }];
    items.extend(
        result
            .sources
            .iter()
            .enumerate()
            .map(|(index, source)| EvidenceItem {
                source: format!(
                    "external_capability:{}:{}",
                    result.descriptor.id, source.locator
                ),
                snippet: trim_for_planner(&format!("{}\n{}", source.label, source.snippet), 1_200),
                rationale: rationale.to_string(),
                rank: index + 1,
            }),
    );
    items
}

fn format_external_capability_invocation(
    descriptor: Option<&ExternalCapabilityDescriptor>,
    invocation: &ExternalCapabilityInvocation,
) -> String {
    let (fabric, availability, auth, effects, evidence) = descriptor
        .map(|descriptor| {
            (
                descriptor.id.as_str(),
                descriptor.availability.label(),
                descriptor.auth_posture.label(),
                descriptor.side_effect_posture.label(),
                descriptor
                    .evidence_shape
                    .kinds
                    .iter()
                    .map(|kind| kind.label())
                    .collect::<Vec<_>>()
                    .join(","),
            )
        })
        .unwrap_or((
            invocation.capability_id.as_str(),
            "unknown",
            "unknown",
            "unknown",
            "unknown".to_string(),
        ));
    let mut lines = vec![
        format!(
            "fabric={fabric} availability={availability} auth={auth} effects={effects} evidence={evidence}"
        ),
        format!("purpose={}", invocation.purpose),
    ];
    if !invocation.payload.is_null() {
        lines.push(format!("payload={}", invocation.payload));
    }
    lines.join("\n")
}

fn format_external_capability_catalog_entry(descriptor: &ExternalCapabilityDescriptor) -> String {
    let evidence = descriptor
        .evidence_shape
        .kinds
        .iter()
        .map(|kind| kind.label())
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "fabric={} availability={} auth={} effects={} evidence={evidence}",
        descriptor.id,
        descriptor.availability.label(),
        descriptor.auth_posture.label(),
        descriptor.side_effect_posture.label(),
    )
}

fn format_external_capability_outcome(
    descriptor: Option<&ExternalCapabilityDescriptor>,
    invocation: &ExternalCapabilityInvocation,
    status: ExternalCapabilityResultStatus,
    summary: String,
    detail: String,
    sources: &[ExternalCapabilitySourceRecord],
) -> String {
    let (fabric, availability, auth, effects) = descriptor
        .map(|descriptor| {
            (
                descriptor.id.as_str(),
                descriptor.availability.label(),
                descriptor.auth_posture.label(),
                descriptor.side_effect_posture.label(),
            )
        })
        .unwrap_or((
            invocation.capability_id.as_str(),
            "unavailable",
            "unknown",
            "unknown",
        ));
    let mut lines = vec![
        format!(
            "fabric={fabric} status={} availability={availability} auth={auth} effects={effects}",
            status.label()
        ),
        format!("purpose={}", invocation.purpose),
        format!("summary={summary}"),
        format!("detail={detail}"),
    ];
    if sources.is_empty() {
        lines.push("provenance=none".to_string());
    } else {
        lines.extend(
            sources
                .iter()
                .map(|source| format!("provenance={} -> {}", source.label, source.locator)),
        );
    }
    lines.join("\n")
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
        profile: Some(prepared.harness_profile().active_profile_id().to_string()),
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
            WorkspaceAction::ExternalCapability { invocation } => Some(invocation.summary()),
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
        WorkspaceAction::ExternalCapability { invocation } => {
            format!("external_capability:{}", invocation.capability_id)
        }
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
        ActiveRuntimeState, DeliberationContinuation, DeliberationSignal, DeliberationSignals,
        GathererProvider, MechSuitService, POLICY_VIOLATION_DIRECT_REPLY, PreparedGathererLane,
        PreparedModelLane, PreparedRuntimeLanes, RuntimeLaneConfig, RuntimeLaneRole,
        StructuredTurnTrace, TurnIntent, budget_signal_details, render_turn_event,
    };
    use crate::domain::model::DeliberationState;
    use crate::domain::model::{
        AuthoredResponse, CollaborationMode, CollaborationModeRequest,
        CollaborationModeRequestSource, CollaborationModeRequestTarget, CollaborationModeResult,
        CompactionPlan, CompactionRequest, ControlOperation, ControlResultStatus,
        ConversationReplayView, ResponseMode, StructuredClarificationStatus, TurnControlOperation,
    };
    use crate::domain::model::{
        ContextStrain, ConversationForensicUpdate, ConversationThreadRef,
        ConversationTranscriptSpeaker, ConversationTranscriptUpdate, ExecutionApprovalPolicy,
        ExecutionGovernanceDecision, ExecutionGovernanceOutcome, ExecutionGovernanceOutcomeKind,
        ExecutionGovernanceSnapshot, ExecutionHandAuthority, ExecutionHandKind, ExecutionHandPhase,
        ExecutionPermission, ExecutionPermissionRequest, ExecutionPermissionRequirement,
        ExecutionSandboxMode, ExternalCapabilityAuthPosture, ExternalCapabilityAvailability,
        ExternalCapabilityDescriptor, ExternalCapabilityDescriptorMetadata,
        ExternalCapabilityEvidenceKind, ExternalCapabilityEvidenceShape,
        ExternalCapabilityInvocation, ExternalCapabilityResult, ExternalCapabilityResultStatus,
        ExternalCapabilitySideEffectPosture, ExternalCapabilitySourceRecord,
        ForensicArtifactCapture, ForensicLifecycle, ForensicTraceSink, ForensicUpdateSink,
        NullTurnEventSink, StrainFactor, TaskTraceId, ThreadDecision, ThreadDecisionId,
        ThreadDecisionKind, ThreadMergeMode, TraceLineageNodeKind, TraceLineageRelation,
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase,
        TraceRecordKind, TraceSignalKind, TranscriptUpdateSink, TurnEvent, TurnEventSink,
    };
    use crate::domain::ports::{
        ContextGatherRequest, ContextGatherResult, ContextGatherer, EntityLookupMode,
        EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest,
        EntityResolver, EvidenceBundle, EvidenceItem, ExternalCapabilityBroker, GroundingDomain,
        GroundingRequirement, InitialAction, InitialActionDecision, InitialEditInstruction,
        InterpretationContext, InterpretationRequest, ModelPaths, ModelRegistry, PlannerAction,
        PlannerBudget, PlannerCapability, PlannerGraphBranch, PlannerGraphBranchStatus,
        PlannerGraphEpisode, PlannerLoopState, PlannerRequest, PlannerStepRecord,
        PlannerStrategyKind, PlannerTraceMetadata, RecursivePlanner, RecursivePlannerDecision,
        RetainedEvidence, RetrievalMode, RetrievalStrategy, RetrieverOption, SynthesisHandoff,
        SynthesizerEngine, ThreadDecisionRequest, TraceRecorder, TraceRecorderCapability,
        WorkspaceAction, WorkspaceActionCapability, WorkspaceActionExecutionFrame,
        WorkspaceActionExecutor, WorkspaceCapabilitySurface, WorkspaceToolCapability,
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
    use serde_json::json;
    use sift::Conversation;
    use std::collections::VecDeque;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::Ordering;
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

    fn bind_workspace_action_executor<T>(service: &MechSuitService, executor: Arc<T>)
    where
        T: WorkspaceActionExecutor + 'static,
    {
        service.set_workspace_action_executor(executor);
    }

    async fn install_direct_answer_runtime(service: &MechSuitService) {
        *service.runtime.write().await = Some(ActiveRuntimeState {
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
                Arc::new(Mutex::new(Vec::new())),
            )),
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });
    }

    #[test]
    fn service_new_uses_persistent_trace_recorder_posture_by_default() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());

        assert!(matches!(
            service.trace_recorder_capability(),
            TraceRecorderCapability::Persistent { ref backend, .. }
                if backend == "embedded_transit"
        ));
    }

    #[test]
    fn service_new_exposes_default_execution_hand_diagnostics_surface() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());

        let diagnostics = service.execution_hand_diagnostics();

        assert_eq!(diagnostics.len(), 3);
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.hand == ExecutionHandKind::WorkspaceEditor
                && diagnostic.phase == ExecutionHandPhase::Described
                && diagnostic.authority == ExecutionHandAuthority::WorkspaceScoped
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.hand == ExecutionHandKind::TerminalRunner
                && diagnostic.phase == ExecutionHandPhase::Described
                && diagnostic.authority == ExecutionHandAuthority::WorkspaceScoped
        }));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.hand == ExecutionHandKind::TransportMediator
                && diagnostic.phase == ExecutionHandPhase::Described
                && diagnostic.authority == ExecutionHandAuthority::CredentialMediated
        }));
    }

    #[test]
    fn service_new_exposes_default_external_capability_catalog_surface() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());

        let descriptors = service.external_capability_descriptors();

        assert_eq!(descriptors.len(), 3);
        assert!(descriptors.iter().any(|descriptor| {
            descriptor.kind == crate::domain::model::ExternalCapabilityKind::WebSearch
                && descriptor.availability
                    == crate::domain::model::ExternalCapabilityAvailability::Unavailable
        }));
        assert!(descriptors.iter().all(|descriptor| {
            descriptor.hand == crate::domain::model::ExecutionHandKind::TransportMediator
        }));
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

    #[derive(Clone)]
    struct ThreadDecisionPlan {
        kind: ThreadDecisionKind,
        rationale: String,
        new_thread_label: Option<String>,
        merge_mode: Option<ThreadMergeMode>,
        merge_summary: Option<String>,
        target_thread: Option<ConversationThreadRef>,
    }

    impl ThreadDecisionPlan {
        fn continue_current() -> Self {
            Self {
                kind: ThreadDecisionKind::ContinueCurrent,
                rationale: "test planner keeps steering on the active thread".to_string(),
                new_thread_label: None,
                merge_mode: None,
                merge_summary: None,
                target_thread: None,
            }
        }

        fn open_child(label: &str) -> Self {
            Self {
                kind: ThreadDecisionKind::OpenChildThread,
                rationale: "test planner opens a child thread for the steered path".to_string(),
                new_thread_label: Some(label.to_string()),
                merge_mode: None,
                merge_summary: None,
                target_thread: None,
            }
        }
    }

    struct BlockingTurnControlPlanner {
        initial_decision: InitialActionDecision,
        next_decisions: Mutex<VecDeque<RecursivePlannerDecision>>,
        recorded_requests: Arc<Mutex<Vec<PlannerRequest>>>,
        thread_decision: ThreadDecisionPlan,
        release_initial_once: Mutex<Option<Arc<tokio::sync::Notify>>>,
    }

    impl BlockingTurnControlPlanner {
        fn new(
            initial_decision: InitialActionDecision,
            next_decisions: Vec<RecursivePlannerDecision>,
            recorded_requests: Arc<Mutex<Vec<PlannerRequest>>>,
            thread_decision: ThreadDecisionPlan,
            release_initial_once: Arc<tokio::sync::Notify>,
        ) -> Self {
            Self {
                initial_decision,
                next_decisions: Mutex::new(VecDeque::from(next_decisions)),
                recorded_requests,
                thread_decision,
                release_initial_once: Mutex::new(Some(release_initial_once)),
            }
        }
    }

    #[async_trait]
    impl RecursivePlanner for BlockingTurnControlPlanner {
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
                    "blocking test interpretation assembled from {} operator-memory document(s).",
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
            let gate = self
                .release_initial_once
                .lock()
                .expect("initial gate lock")
                .take();
            if let Some(gate) = gate {
                gate.notified().await;
            }
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
                .ok_or_else(|| anyhow!("blocking test planner exhausted"))
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
                kind: self.thread_decision.kind,
                rationale: self.thread_decision.rationale.clone(),
                target_thread: self
                    .thread_decision
                    .target_thread
                    .clone()
                    .unwrap_or_else(|| request.active_thread.thread_ref.clone()),
                new_thread_label: self.thread_decision.new_thread_label.clone(),
                merge_mode: self.thread_decision.merge_mode,
                merge_summary: self.thread_decision.merge_summary.clone(),
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
    struct RecordingExternalCapabilityBroker {
        descriptors: Vec<ExternalCapabilityDescriptor>,
        results: Vec<ExternalCapabilityResult>,
        invocations: Mutex<Vec<ExternalCapabilityInvocation>>,
    }

    impl RecordingExternalCapabilityBroker {
        fn new(
            descriptors: Vec<ExternalCapabilityDescriptor>,
            results: Vec<ExternalCapabilityResult>,
        ) -> Self {
            Self {
                descriptors,
                results,
                invocations: Mutex::new(Vec::new()),
            }
        }

        fn recorded_invocations(&self) -> Vec<ExternalCapabilityInvocation> {
            self.invocations.lock().expect("invocations lock").clone()
        }
    }

    impl ExternalCapabilityBroker for RecordingExternalCapabilityBroker {
        fn descriptors(&self) -> Vec<ExternalCapabilityDescriptor> {
            self.descriptors.clone()
        }

        fn invoke(
            &self,
            invocation: &ExternalCapabilityInvocation,
        ) -> Result<ExternalCapabilityResult> {
            self.invocations
                .lock()
                .expect("invocations lock")
                .push(invocation.clone());
            self.results
                .iter()
                .find(|result| result.descriptor.id == invocation.capability_id)
                .cloned()
                .ok_or_else(|| {
                    anyhow!(
                        "missing external capability result for `{}`",
                        invocation.capability_id
                    )
                })
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
        gathered_bundles: Mutex<Vec<EvidenceBundle>>,
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
                self.gathered_bundles
                    .lock()
                    .expect("gathered bundles lock")
                    .push(bundle.clone());
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
    }

    impl WorkspaceActionExecutor for RecordingSynthesizer {
        fn execute_workspace_action(
            &self,
            action: &WorkspaceAction,
            _frame: WorkspaceActionExecutionFrame<'_>,
        ) -> Result<crate::domain::ports::WorkspaceActionResult> {
            self.executed_actions
                .lock()
                .expect("executed actions lock")
                .push(action.clone());
            Ok(crate::domain::ports::WorkspaceActionResult {
                name: action.label().to_string(),
                summary: format!("executed {}", action.summary()),
                applied_edit: mock_applied_edit_for_action(action),
                governance_request: None,
                governance_outcome: None,
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

    fn planner_decision(action: PlannerAction, rationale: &str) -> RecursivePlannerDecision {
        RecursivePlannerDecision {
            action,
            rationale: rationale.to_string(),
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: None,
            deliberation_state: None,
        }
    }

    fn moonshot_continuation_state() -> DeliberationState {
        DeliberationState::new(
            ModelProvider::Moonshot.name(),
            "kimi-k2.6",
            json!({
                "kind": "moonshot_openai_chat_completion",
                "assistant": {
                    "role": "assistant",
                    "reasoning_content": "inspect the current tool path before stopping",
                    "tool_calls": [
                        {
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "inspect",
                                "arguments": "{\"command\":\"pwd\"}"
                            }
                        }
                    ]
                }
            }),
        )
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
            gatherer: None,
            resolver: Arc::new(NoopContextResolver),
            entity_resolver: Arc::new(WorkspaceEntityResolver::new()),
            workspace_capability_surface: WorkspaceCapabilitySurface::default(),
            execution_hands: Vec::new(),
            governance_profile: None,
            external_capabilities: Vec::new(),
            interpretation: InterpretationContext::default(),
            recent_turns: Vec::new(),
            recent_thread_summary: None,
            collaboration: CollaborationModeResult::default(),
            specialist_runtime_notes: Vec::new(),
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

    #[tokio::test]
    async fn prepare_runtime_lanes_resolves_structured_harness_profile_without_downgrade() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(RecordingRegistry::default());
        let operator_memory = Arc::new(AgentMemory::load(workspace.path()));
        let service = MechSuitService::new(
            workspace.path(),
            registry,
            operator_memory,
            Box::new(|_, _lane| {
                Ok(Arc::new(RecordingSynthesizer::default()) as Arc<dyn SynthesizerEngine>)
            }),
            Box::new(|_, _lane| {
                Ok(Arc::new(TestPlanner::new(
                    initial_action_decision(InitialAction::Answer, "not used"),
                    Vec::new(),
                    Arc::new(Mutex::new(Vec::new())),
                )) as Arc<dyn RecursivePlanner>)
            }),
            Box::new(|_, _, _, _| Ok(None)),
        );
        let config = RuntimeLaneConfig::new("gpt-5.4".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_planner_provider(Some(ModelProvider::Google))
            .with_planner_model_id(Some("gemini-2.5-flash".to_string()));

        let prepared = service
            .prepare_runtime_lanes(&config)
            .await
            .expect("prepare runtime lanes");

        let selection = prepared.harness_profile();
        assert_eq!(selection.requested.id(), "recursive-structured-v1");
        assert_eq!(selection.active.id(), "recursive-structured-v1");
        assert_eq!(selection.downgrade_reason, None);
    }

    #[tokio::test]
    async fn prepare_runtime_lanes_downgrades_harness_profile_when_prompt_envelopes_are_required() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(RecordingRegistry::default());
        let operator_memory = Arc::new(AgentMemory::load(workspace.path()));
        let service = MechSuitService::new(
            workspace.path(),
            registry,
            operator_memory,
            Box::new(|_, _lane| {
                Ok(Arc::new(RecordingSynthesizer::default()) as Arc<dyn SynthesizerEngine>)
            }),
            Box::new(|_, _lane| {
                Ok(Arc::new(TestPlanner::new(
                    initial_action_decision(InitialAction::Answer, "not used"),
                    Vec::new(),
                    Arc::new(Mutex::new(Vec::new())),
                )) as Arc<dyn RecursivePlanner>)
            }),
            Box::new(|_, _, _, _| Ok(None)),
        );
        let config = RuntimeLaneConfig::new("claude-sonnet-4-20250514".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Anthropic)
            .with_planner_provider(Some(ModelProvider::Anthropic))
            .with_planner_model_id(Some("claude-sonnet-4-20250514".to_string()));

        let prepared = service
            .prepare_runtime_lanes(&config)
            .await
            .expect("prepare runtime lanes");

        let selection = prepared.harness_profile();
        assert_eq!(selection.requested.id(), "recursive-structured-v1");
        assert_eq!(selection.active.id(), "prompt-envelope-safe-v1");
        assert_eq!(
            selection.downgrade_reason.as_deref(),
            Some("planner next-action transport requires prompt-envelope recovery")
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
    fn workspace_capability_enrichment_adds_generic_probe_guidance() {
        let context = super::enrich_interpretation_with_workspace_capability_surface(
            InterpretationContext::default(),
            &WorkspaceCapabilitySurface {
                actions: vec![
                    WorkspaceActionCapability::new(
                        "inspect",
                        "run a single read-only shell probe through the terminal hand",
                        false,
                    ),
                    WorkspaceActionCapability::new(
                        "shell",
                        "run a governed workspace command when a command should execute now",
                        true,
                    ),
                ],
                tools: Vec::new(),
                notes: Vec::new(),
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
                WorkspaceAction::Inspect { ref command } if command == "command -v <tool>"
            ) && hint.note.contains("probe that exact tool first")
        }));
        assert!(
            context
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| {
                    procedure.label == "Probe Required Local Tools"
                && procedure
                    .steps
                    .iter()
                    .any(|step| matches!(
                        step.action,
                        WorkspaceAction::Inspect { ref command } if command == "command -v <tool>"
                    ))
                })
        );
    }

    #[test]
    fn workspace_capability_enrichment_surfaces_observed_tools_without_prebaked_tool_hints() {
        let context = super::enrich_interpretation_with_workspace_capability_surface(
            InterpretationContext::default(),
            &WorkspaceCapabilitySurface {
                actions: vec![WorkspaceActionCapability::new(
                    "inspect",
                    "run a single read-only shell probe through the terminal hand",
                    false,
                )],
                tools: vec![WorkspaceToolCapability::new(
                    "cargo",
                    "observed available from prior tool probe `command -v cargo`",
                    Some(WorkspaceAction::Inspect {
                        command: "command -v cargo".to_string(),
                    }),
                )],
                notes: Vec::new(),
            },
        );

        assert!(context.summary.contains("Observed local tools: cargo."));
        assert!(!context.tool_hints.iter().any(|hint| matches!(
            hint.action,
            WorkspaceAction::Inspect { ref command } if command == "gh run list --limit 10"
        )));
        assert!(!context.tool_hints.iter().any(|hint| matches!(
            hint.action,
            WorkspaceAction::Inspect { ref command } if command == "git status --short"
        )));
    }

    #[test]
    fn workspace_capability_enrichment_does_not_emit_tool_specific_ci_procedures() {
        let context = super::enrich_interpretation_with_workspace_capability_surface(
            InterpretationContext::default(),
            &WorkspaceCapabilitySurface {
                actions: vec![
                    WorkspaceActionCapability::new(
                        "inspect",
                        "run a single read-only shell probe through the terminal hand",
                        false,
                    ),
                    WorkspaceActionCapability::new(
                        "shell",
                        "run a governed workspace command when a command should execute now",
                        true,
                    ),
                ],
                tools: vec![
                    WorkspaceToolCapability::new(
                        "nix",
                        "observed available from prior shell command `nix --version`",
                        Some(WorkspaceAction::Inspect {
                            command: "command -v nix".to_string(),
                        }),
                    ),
                    WorkspaceToolCapability::new(
                        "just",
                        "observed available from prior shell command `just --version`",
                        Some(WorkspaceAction::Inspect {
                            command: "command -v just".to_string(),
                        }),
                    ),
                ],
                notes: Vec::new(),
            },
        );

        assert!(
            !context
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| { procedure.label == "Diagnose CI Or Actions" })
        );
    }

    #[test]
    fn planner_execution_contract_describes_inspect_as_single_step() {
        let contract =
            super::build_planner_execution_contract(super::PlannerExecutionContractContext {
                workspace_capability_surface: &WorkspaceCapabilitySurface {
                    actions: vec![
                        WorkspaceActionCapability::new(
                            "inspect",
                            "run a single read-only shell probe through the terminal hand",
                            false,
                        ),
                        WorkspaceActionCapability::new(
                            "shell",
                            "run a governed workspace command when a command should execute now",
                            true,
                        ),
                    ],
                    tools: Vec::new(),
                    notes: Vec::new(),
                },
                execution_hands: &[],
                governance_profile: None,
                external_capabilities: &[],
                gatherer: None,
                collaboration: &CollaborationModeResult::default(),
                instruction_frame: None,
                grounding: None,
            });

        assert!(contract.capability_manifest.iter().any(|line| line.contains(
            "workspace action inspect: available (read-only) — run a single read-only shell probe through the terminal hand"
        )));
        assert!(contract.completion_contract.iter().any(|line| {
            line.contains("bounded single-step probe such as `inspect` `command -v <tool>`")
        }));
        assert!(
            contract
                .completion_contract
                .iter()
                .any(|line| line.contains("`inspect` is only for single read-only probes"))
        );
    }

    #[test]
    fn external_capability_enrichment_adds_discovery_hints_and_grounding_procedure() {
        let descriptor = ExternalCapabilityDescriptor::new(
            "web.search",
            crate::domain::model::ExternalCapabilityKind::WebSearch,
            "Web Search",
            "Search the public web for current documentation and return citations.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Available,
                ExternalCapabilityAuthPosture::NoneRequired,
                ExternalCapabilitySideEffectPosture::ReadOnly,
                ExecutionHandKind::TransportMediator,
                Vec::new(),
                ExternalCapabilityEvidenceShape::new(
                    "current web answers should produce citations and a runtime summary",
                    vec![
                        ExternalCapabilityEvidenceKind::Citation,
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                    ],
                ),
            ),
        );

        let context = super::enrich_interpretation_with_external_capabilities(
            InterpretationContext::default(),
            &[descriptor],
        );

        assert!(context.summary.contains("external capability fabrics"));
        assert!(context.tool_hints.iter().any(|hint| matches!(
            hint.action,
            WorkspaceAction::ExternalCapability { ref invocation }
                if invocation.capability_id == "web.search"
        )));
        assert!(
            context
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| {
                    procedure.label == "Ground With External Capabilities"
                        && procedure.steps.iter().any(|step| {
                            matches!(
                                step.action,
                                WorkspaceAction::ExternalCapability { ref invocation }
                                    if invocation.capability_id == "web.search"
                            )
                        })
                })
        );
    }

    #[test]
    fn external_capability_actions_route_through_discovery_governance_and_evidence() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("README.md"), "# Workspace\n").expect("write readme");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let invocation = ExternalCapabilityInvocation::new(
            "web.search",
            "confirm the latest external docs",
            json!({ "query": "paddles external capability docs" }),
        );
        let descriptor = ExternalCapabilityDescriptor::new(
            "web.search",
            crate::domain::model::ExternalCapabilityKind::WebSearch,
            "Web Search",
            "Search the public web for current documentation and return citations.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Available,
                ExternalCapabilityAuthPosture::NoneRequired,
                ExternalCapabilitySideEffectPosture::ReadOnly,
                ExecutionHandKind::TransportMediator,
                Vec::new(),
                ExternalCapabilityEvidenceShape::new(
                    "current web answers should produce citations and a runtime summary",
                    vec![
                        ExternalCapabilityEvidenceKind::Citation,
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                        ExternalCapabilityEvidenceKind::SourceLineage,
                    ],
                ),
            ),
        );
        let result = ExternalCapabilityResult {
            descriptor: descriptor.clone(),
            invocation: invocation.clone(),
            status: ExternalCapabilityResultStatus::Succeeded,
            summary: "Web Search succeeded".to_string(),
            detail: "Found the latest external capability docs with a cited source.".to_string(),
            sources: vec![ExternalCapabilitySourceRecord {
                label: "OpenAI docs".to_string(),
                locator: "https://example.com/docs".to_string(),
                snippet: "External capability calls should return citation-backed evidence."
                    .to_string(),
            }],
        };
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::ExternalCapability {
                        invocation: invocation.clone(),
                    },
                },
                "use the external evidence lane before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "external evidence captured the answer".to_string(),
                },
                rationale: "the cited web result is enough".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&request_log),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let broker = Arc::new(RecordingExternalCapabilityBroker::new(
            vec![descriptor.clone()],
            vec![result],
        ));
        let service = test_service(workspace.path());
        service.set_external_capability_broker(broker.clone());
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
                .process_prompt_with_sink("look up the latest docs", sink.clone())
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Applied the bounded action.");
        let requests = request_log.lock().expect("request log");
        assert!(
            requests[0]
                .interpretation
                .tool_hints
                .iter()
                .any(|hint| matches!(
                    hint.action,
                    WorkspaceAction::ExternalCapability { ref invocation }
                        if invocation.capability_id == "web.search"
                ))
        );
        assert!(
            requests[0]
                .interpretation
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| procedure.label == "Ground With External Capabilities")
        );
        drop(requests);

        assert_eq!(broker.recorded_invocations(), vec![invocation]);
        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolCalled { tool_name, invocation, .. }
                if tool_name == "external_capability"
                    && invocation.contains("web.search")
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ExecutionGovernanceDecisionRecorded { decision }
                if decision.tool_name.as_deref() == Some("external_capability")
                    && decision.request.hand == ExecutionHandKind::TransportMediator
                    && decision.outcome.kind == ExecutionGovernanceOutcomeKind::Allowed
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolFinished { tool_name, summary, .. }
                if tool_name == "external_capability"
                    && summary.contains("Web Search succeeded")
        )));

        let bundles = synthesizer
            .gathered_bundles
            .lock()
            .expect("gathered bundles lock");
        let bundle = bundles.last().expect("gathered evidence bundle");
        assert!(bundle.items.iter().any(|item| {
            item.source.contains("web.search")
                && item.snippet.contains("latest external capability docs")
        }));
        assert!(bundle.items.iter().any(|item| {
            item.source.contains("https://example.com/docs")
                && item.snippet.contains("citation-backed evidence")
        }));
    }

    #[test]
    fn external_capability_actions_fail_closed_when_governance_blocks_network_access() {
        let workspace = tempfile::tempdir().expect("workspace");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let invocation = ExternalCapabilityInvocation::new(
            "web.search",
            "look up the latest release notes",
            json!({ "query": "paddles release notes" }),
        );
        let descriptor = ExternalCapabilityDescriptor::new(
            "web.search",
            crate::domain::model::ExternalCapabilityKind::WebSearch,
            "Web Search",
            "Search the public web for current documentation and return citations.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Available,
                ExternalCapabilityAuthPosture::NoneRequired,
                ExternalCapabilitySideEffectPosture::ReadOnly,
                ExecutionHandKind::TransportMediator,
                vec![ExecutionPermission::AccessNetwork],
                ExternalCapabilityEvidenceShape::new(
                    "network-backed search should yield citations when allowed",
                    vec![ExternalCapabilityEvidenceKind::Citation],
                ),
            ),
        );
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::ExternalCapability {
                        invocation: invocation.clone(),
                    },
                },
                "use web search before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "governance outcome was recorded".to_string(),
                },
                rationale: "stop after the governance boundary is exercised".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&request_log),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let broker = Arc::new(RecordingExternalCapabilityBroker::new(
            vec![descriptor.clone()],
            vec![ExternalCapabilityResult {
                descriptor: descriptor.clone(),
                invocation: invocation.clone(),
                status: ExternalCapabilityResultStatus::Succeeded,
                summary: "unexpected".to_string(),
                detail: "broker should not be invoked when governance blocks the call".to_string(),
                sources: Vec::new(),
            }],
        ));
        let service = test_service(workspace.path());
        service.set_external_capability_broker(broker.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_with_sink("look up the latest release notes", sink.clone())
                .await
                .expect("process prompt");
        });

        assert!(
            broker.recorded_invocations().is_empty(),
            "blocked transport calls must not invoke the broker"
        );
        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ExecutionGovernanceDecisionRecorded { decision }
                if decision.tool_name.as_deref() == Some("external_capability")
                    && decision.request.hand == ExecutionHandKind::TransportMediator
                    && decision.outcome.kind == ExecutionGovernanceOutcomeKind::EscalationRequired
                    && decision.request.requirement.permissions.contains(&ExecutionPermission::AccessNetwork)
        )));

        let bundles = synthesizer
            .gathered_bundles
            .lock()
            .expect("gathered bundles lock");
        let bundle = bundles.last().expect("gathered evidence bundle");
        assert!(bundle.items.iter().any(|item| {
            item.source.contains("web.search") && item.snippet.contains("requires approval")
        }));
    }

    #[test]
    fn external_capability_actions_remain_useful_when_the_fabric_is_disabled() {
        let workspace = tempfile::tempdir().expect("workspace");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let invocation = ExternalCapabilityInvocation::new(
            "connector.app_action",
            "query the connector fabric",
            json!({ "app": "gmail", "action": "search" }),
        );
        let descriptor = ExternalCapabilityDescriptor::new(
            "connector.app_action",
            crate::domain::model::ExternalCapabilityKind::ConnectorApp,
            "Connector App Action",
            "Invoke a connector-backed application action through the transport boundary.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Disabled,
                ExternalCapabilityAuthPosture::Required,
                ExternalCapabilitySideEffectPosture::PotentiallyMutating,
                ExecutionHandKind::TransportMediator,
                Vec::new(),
                ExternalCapabilityEvidenceShape::new(
                    "connector actions should explain degraded availability",
                    vec![ExternalCapabilityEvidenceKind::RuntimeSummary],
                ),
            ),
        );
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::ExternalCapability {
                        invocation: invocation.clone(),
                    },
                },
                "probe the connector fabric before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "the degraded connector state is enough evidence".to_string(),
                },
                rationale: "stop after capturing the disabled state".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&request_log),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let broker = Arc::new(RecordingExternalCapabilityBroker::new(
            vec![descriptor.clone()],
            vec![ExternalCapabilityResult::unavailable(
                descriptor,
                invocation.clone(),
                "Connector App Action is currently disabled in this runtime",
            )],
        ));
        let service = test_service(workspace.path());
        service.set_external_capability_broker(broker.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());

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
                    "check whether the connector fabric is available",
                    sink.clone(),
                )
                .await
                .expect("process prompt");
        });

        assert_eq!(broker.recorded_invocations(), vec![invocation]);
        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolFinished { tool_name, summary, .. }
                if tool_name == "external_capability"
                    && summary.contains("currently disabled")
        )));

        let bundles = synthesizer
            .gathered_bundles
            .lock()
            .expect("gathered bundles lock");
        let bundle = bundles.last().expect("gathered evidence bundle");
        assert!(bundle.items.iter().any(|item| {
            item.source.contains("connector.app_action")
                && item.snippet.contains("currently disabled")
        }));
    }

    #[test]
    fn degraded_external_capability_results_project_honest_state_across_trace_and_transcript() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nProject external capability degradation into transcript and trace surfaces.\n",
        )
        .expect("write AGENTS.md");

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("planner")),
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let invocation = ExternalCapabilityInvocation::new(
            "web.search",
            "confirm the latest release notes",
            json!({ "query": "paddles release notes" }),
        );
        let descriptor = ExternalCapabilityDescriptor::new(
            "web.search",
            crate::domain::model::ExternalCapabilityKind::WebSearch,
            "Web Search",
            "Search the public web for current release notes and return citations.",
            ExternalCapabilityDescriptorMetadata::new(
                ExternalCapabilityAvailability::Stale,
                ExternalCapabilityAuthPosture::NoneRequired,
                ExternalCapabilitySideEffectPosture::ReadOnly,
                ExecutionHandKind::TransportMediator,
                Vec::new(),
                ExternalCapabilityEvidenceShape::new(
                    "release-note lookups should yield citations and runtime summaries",
                    vec![
                        ExternalCapabilityEvidenceKind::Citation,
                        ExternalCapabilityEvidenceKind::RuntimeSummary,
                        ExternalCapabilityEvidenceKind::SourceLineage,
                    ],
                ),
            ),
        );
        let result = ExternalCapabilityResult {
            descriptor: descriptor.clone(),
            invocation: invocation.clone(),
            status: ExternalCapabilityResultStatus::Degraded,
            summary: "Web Search degraded".to_string(),
            detail: "Capability metadata is stale; using cached release notes.".to_string(),
            sources: vec![ExternalCapabilitySourceRecord {
                label: "Release notes".to_string(),
                locator: "https://example.com/releases".to_string(),
                snippet: "Cached release notes still describe the current external fabric posture."
                    .to_string(),
            }],
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::ExternalCapability {
                        invocation: invocation.clone(),
                    },
                },
                "ground the answer with the external capability lane first",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "degraded external result recorded".to_string(),
                },
                rationale: "the degraded external result is sufficient evidence".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let broker = Arc::new(RecordingExternalCapabilityBroker::new(
            vec![descriptor],
            vec![result],
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        service.set_external_capability_broker(broker);
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
                    "look up the latest release notes",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt");
        });

        let projection = service
            .replay_conversation_projection(&session.task_id())
            .expect("projection replay");
        assert!(projection.transcript.entries.iter().any(|entry| {
            entry.speaker == ConversationTranscriptSpeaker::System
                && entry.content.contains("fabric=web.search")
                && entry.content.contains("status=degraded")
                && entry.content.contains("availability=stale")
                && entry
                    .content
                    .contains("provenance=Release notes -> https://example.com/releases")
        }));
        assert!(projection.trace_graph.nodes.iter().any(|node| {
            node.kind == "tool_done"
                && node.label.contains("web.search")
                && node.label.contains("degraded")
        }));

        let replay = service
            .replay_for_known_session(&session.task_id())
            .expect("known replay query")
            .expect("stored replay");
        let completed_tool_call = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::ToolCallCompleted(tool)
                    if tool.tool_name == "external_capability" =>
                {
                    Some(tool)
                }
                _ => None,
            })
            .expect("external capability tool completion");
        assert_eq!(completed_tool_call.success, Some(false));
        assert!(
            completed_tool_call
                .payload
                .inline_content
                .as_deref()
                .expect("tool payload")
                .contains("status=degraded")
        );
        assert!(
            completed_tool_call
                .payload
                .inline_content
                .as_deref()
                .expect("tool payload")
                .contains("provenance=Release notes -> https://example.com/releases")
        );
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
        let sources = requests[0].interpretation.sources();
        assert!(sources.contains(&"AGENTS.md".to_string()));
        assert!(sources.contains(&"paddles-harness".to_string()));
        assert!(sources.contains(&"external-capability-catalog".to_string()));
        assert!(!requests[0].interpretation.tool_hints.is_empty());
        assert!(
            requests[0]
                .interpretation
                .decision_framework
                .procedures
                .iter()
                .any(|procedure| procedure.label == "Probe Required Local Tools")
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
        let _reply = runtime.block_on(async {
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
        let planner_action = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::PlannerAction {
                    action,
                    rationale,
                    signal_summary,
                } => Some((action, rationale, signal_summary)),
                _ => None,
            })
            .expect("planner action record");
        assert_eq!(planner_action.0, "answer directly");
        assert_ne!(planner_action.1, "answer directly");
        assert!(planner_action.1.contains("answer directly"));
        assert_eq!(planner_action.2, &None);
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
    fn task_root_trace_records_resolved_harness_profile_selection() {
        let task_id = TaskTraceId::new("task-harness-profile").expect("task");
        let session = ConversationSession::new(task_id.clone());
        let turn_id = session.allocate_turn_id();
        let active_thread = session.active_thread().thread_ref;
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let prepared = PreparedRuntimeLanes {
            planner: MechSuitService::build_lane(
                RuntimeLaneRole::Planner,
                ModelProvider::Anthropic,
                "claude-sonnet-4-20250514",
                None,
            ),
            synthesizer: MechSuitService::build_lane(
                RuntimeLaneRole::Synthesizer,
                ModelProvider::Anthropic,
                "claude-sonnet-4-20250514",
                None,
            ),
            gatherer: None,
        };
        let trace = StructuredTurnTrace::new(
            Arc::new(NullTurnEventSink),
            recorder.clone(),
            Vec::new(),
            session,
            turn_id,
            active_thread,
        );

        trace.record_turn_start(
            "Record this turn",
            &InterpretationContext::default(),
            &prepared,
        );

        let replay = recorder.replay(&task_id).expect("replay");
        let Some(TraceRecordKind::TaskRootStarted(root)) =
            replay.records.first().map(|record| &record.kind)
        else {
            panic!("expected task root trace record");
        };
        assert_eq!(
            root.harness_profile.active_profile_id,
            "prompt-envelope-safe-v1"
        );
        assert_eq!(
            root.harness_profile.downgrade_reason.as_deref(),
            Some("planner next-action transport requires prompt-envelope recovery")
        );
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
    fn plain_turn_event_rendering_includes_planner_signal_summaries() {
        let rendered = render_turn_event(&TurnEvent::PlannerActionSelected {
            sequence: 3,
            action: "inspect `ls`".to_string(),
            rationale: "Paddles chose `inspect `ls`` because evidence from command: pwd narrowed the next bounded step.".to_string(),
            signal_summary: Some(
                "continuation=tool_follow_up; uncertainty=opaque".to_string(),
            ),
        });

        assert!(rendered.contains("Planner step 3: inspect `ls`"));
        assert!(rendered.contains("Rationale: Paddles chose `inspect `ls``"));
        assert!(rendered.contains("Signals: continuation=tool_follow_up; uncertainty=opaque"));
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
        trace.record_planner_action(
            "read src/lib.rs",
            "act on the likeliest file first",
            None,
            None,
        );
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
                    .authored_response
                    .as_ref()
                    .map(|response| response.mode.label().to_string()),
                _ => None,
            })
            .expect("completion response mode");
        let persisted_document = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::CompletionCheckpoint(checkpoint) => checkpoint
                    .authored_response
                    .as_ref()
                    .map(|response| response.document.clone()),
                _ => None,
            })
            .expect("persisted response document");
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
        assert_eq!(persisted_document.to_plain_text(), "Patched src/lib.rs");
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

            deliberation_state: None,
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
    async fn shared_conversation_session_starts_after_highest_persisted_task_on_restart() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service_one = test_service(workspace.path());
        install_direct_answer_runtime(&service_one).await;

        service_one
            .process_prompt_in_session_with_sink(
                "First prompt",
                service_one.shared_conversation_session(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .await
            .expect("process first prompt");

        let service_two = test_service(workspace.path());
        let shared = service_two.shared_conversation_session();

        assert_eq!(shared.task_id().as_str(), "task-000002");
        assert_eq!(shared.allocate_turn_id().as_str(), "task-000002.turn-0001");
    }

    #[tokio::test]
    async fn resumable_conversations_list_prior_persisted_tasks() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service_one = test_service(workspace.path());
        install_direct_answer_runtime(&service_one).await;

        service_one
            .process_prompt_in_session_with_sink(
                "First prompt",
                service_one.shared_conversation_session(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .await
            .expect("process first prompt");

        let service_two = test_service(workspace.path());
        let resumable = service_two
            .resumable_conversations()
            .expect("resumable conversations");

        assert_eq!(resumable.len(), 1);
        assert_eq!(resumable[0].task_id.as_str(), "task-000001");
        assert_eq!(resumable[0].turn_count, 1);
        assert_eq!(resumable[0].preview, "First prompt");
    }

    #[tokio::test]
    async fn shared_conversation_session_can_restore_persisted_task_on_demand() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service_one = test_service(workspace.path());
        install_direct_answer_runtime(&service_one).await;
        let original = service_one.shared_conversation_session();
        let original_task_id = original.task_id();

        service_one
            .process_prompt_in_session_with_sink(
                "First prompt",
                original,
                Arc::new(RecordingTurnEventSink::default()),
            )
            .await
            .expect("process first prompt");

        let service_two = test_service(workspace.path());
        let fresh = service_two.shared_conversation_session();
        assert_eq!(fresh.task_id().as_str(), "task-000002");

        let restored = service_two
            .restore_shared_conversation_session(&original_task_id)
            .expect("restore persisted conversation");

        assert_eq!(restored.task_id(), original_task_id);
        assert_eq!(
            service_two.shared_conversation_session().task_id(),
            original_task_id
        );
        assert_eq!(
            restored.allocate_turn_id().as_str(),
            "task-000001.turn-0002"
        );
    }

    #[tokio::test]
    async fn service_restarts_keep_prompt_history_but_start_fresh_recent_turn_context() {
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
            vec!["Q: Second prompt".to_string()]
        );

        assert_eq!(
            history_store.prompt_history().expect("prompt history"),
            vec!["First prompt".to_string(), "Second prompt".to_string()]
        );
    }

    #[test]
    fn chamber_services_compose_turn_execution_and_projection_replay() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nCompose chamber seams around turn execution and projection replay.\n",
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
                "Chamber response.".to_string(),
            ])),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .turn_orchestration()
                .process_prompt_in_session_with_mode_request_and_sink(
                    "Execute through chamber seams",
                    session.clone(),
                    None,
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Chamber response.");

        let projection = service
            .conversation_read_model()
            .replay_conversation_projection(&session.task_id())
            .expect("projection replay");

        assert_eq!(projection.task_id, session.task_id());
        assert_eq!(projection.transcript.entries.len(), 3);
        assert_eq!(
            projection.transcript.entries[2].content,
            "Chamber response."
        );
    }

    #[test]
    fn application_read_model_exports_projection_types() {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let replay = crate::domain::model::TraceReplay {
            task_id: task_id.clone(),
            records: Vec::new(),
        };

        let transcript = crate::application::ConversationTranscript::from_trace_replay(&replay);
        let snapshot =
            crate::application::ConversationProjectionSnapshot::from_trace_replay(&replay);

        assert_eq!(transcript.task_id, task_id);
        assert_eq!(snapshot.task_id, task_id);
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
        assert_eq!(transcript.entries.len(), 3);
        assert_eq!(transcript.entries[0].content, "Project this conversation");
        assert_eq!(
            transcript.entries[1].speaker,
            ConversationTranscriptSpeaker::System
        );
        assert!(transcript.entries[1].content.contains("execution posture"));
        assert_eq!(transcript.entries[2].content, "Transcript response.");
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
        assert_eq!(projection.transcript.entries.len(), 3);
        assert_eq!(
            projection.transcript.entries[1].speaker,
            ConversationTranscriptSpeaker::System
        );
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
        assert!(
            projection
                .trace_graph
                .nodes
                .iter()
                .any(|node| node.kind == "governance")
        );
    }

    #[test]
    fn replay_conversation_projection_includes_the_shared_delegation_surface() {
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
            turn_id,
            ConversationThreadRef::Mainline,
        ));
        let worker_branch = session.next_branch_id();
        let worker_thread = ConversationThreadRef::Branch(worker_branch.clone());

        trace.declare_branch(
            worker_branch,
            "delegated projection worker",
            Some("project shared delegation state"),
            None,
        );
        trace.record_worker_lifecycle(
            &ConversationThreadRef::Mainline,
            &worker_thread,
            crate::domain::model::WorkerDelegationRequest::spawn(
                "Project shared delegation state",
                crate::domain::model::WorkerDelegationContract::new(
                    crate::domain::model::WorkerRole::new(
                        "worker",
                        "Worker",
                        "Project delegated worker state across shared surfaces.",
                    ),
                    crate::domain::model::WorkerOwnership::new(
                        "Own src/domain/model/projection.rs",
                        vec!["src/domain/model".to_string()],
                        vec!["src/domain/model/projection.rs".to_string()],
                        crate::domain::model::DelegationIntegrationOwner::Parent,
                    ),
                    crate::domain::model::DelegationGovernancePolicy::inherit_from_parent(
                        &ExecutionGovernanceSnapshot::new(
                            "recursive-structured-v1",
                            "recursive-structured-v1",
                            crate::domain::model::ExecutionGovernanceProfile::new(
                                ExecutionSandboxMode::WorkspaceWrite,
                                ExecutionApprovalPolicy::OnRequest,
                                vec![
                                    crate::domain::model::ExecutionPermissionReuseScope::Turn,
                                    crate::domain::model::ExecutionPermissionReuseScope::Hand,
                                ],
                                None,
                            ),
                        ),
                        crate::domain::model::DelegationEvidencePolicy::new(
                            "Worker state stays visible to the parent.",
                            vec![
                                crate::domain::model::WorkerArtifactKind::ToolCall,
                                crate::domain::model::WorkerArtifactKind::CompletionSummary,
                            ],
                        ),
                    ),
                ),
            ),
            crate::domain::model::WorkerLifecycleResult::new(
                crate::domain::model::WorkerLifecycleOperation::Spawn,
                crate::domain::model::WorkerLifecycleResultStatus::Accepted,
                Some("worker-1".to_string()),
                "Spawned worker-1 on a child thread.",
            ),
        );
        trace.record_worker_artifact(
            &worker_thread,
            crate::domain::model::WorkerArtifactRecord::tool_call(
                "worker-1",
                "shell",
                "rg delegation apps/web/src",
            ),
            "rg delegation apps/web/src",
        );

        let projection = service
            .replay_conversation_projection(&session.task_id())
            .expect("projection replay");

        assert_eq!(projection.delegation.harness_identity, "recursive-harness");
        assert_eq!(projection.delegation.active_worker_count, 1);
        assert_eq!(projection.delegation.workers.len(), 1);
        assert_eq!(projection.delegation.workers[0].role_label, "Worker");
        assert_eq!(projection.delegation.workers[0].parent_thread, "mainline");
        assert!(
            projection.delegation.workers[0]
                .progress_summary
                .contains("parent may continue")
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
            crate::application::ConversationProjectionUpdateKind::Transcript
        );
        assert_eq!(
            transcript_projection_update.reducer,
            crate::application::ConversationProjectionReducer::ReplaceSnapshot
        );
        assert_eq!(transcript_projection_update.version, expected.version());
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
            crate::application::ConversationProjectionUpdateKind::Forensic
        );
        assert_eq!(
            forensic_projection_update.reducer,
            crate::application::ConversationProjectionReducer::ReplaceSnapshot
        );
        assert_eq!(forensic_projection_update.version, expected.version());
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
    fn live_projection_updates_converge_with_replayed_transcript_render_state() {
        struct ConvergenceSynthesizer {
            reply: String,
            citations: Vec<String>,
        }

        impl SynthesizerEngine for ConvergenceSynthesizer {
            fn set_verbose(&self, _level: u8) {}

            fn respond_for_turn(
                &self,
                _prompt: &str,
                _turn_intent: TurnIntent,
                _gathered_evidence: Option<&EvidenceBundle>,
                _handoff: &SynthesisHandoff,
                event_sink: Arc<dyn TurnEventSink>,
            ) -> Result<String> {
                event_sink.emit(TurnEvent::SynthesisReady {
                    grounded: true,
                    citations: self.citations.clone(),
                    insufficient_evidence: false,
                });
                Ok(self.reply.clone())
            }

            fn recent_turn_summaries(&self) -> Result<Vec<String>> {
                Ok(Vec::new())
            }
        }

        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nCompare live projection snapshots against replayed transcript state.\n",
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
        let reply =
            "**Summary**\n\nProjection and replay stay aligned.\n\nSources: README.md".to_string();
        let expected = AuthoredResponse::from_plain_text(ResponseMode::GroundedAnswer, &reply);
        let synthesizer: Arc<dyn SynthesizerEngine> = Arc::new(ConvergenceSynthesizer {
            reply,
            citations: vec!["README.md".to_string()],
        });
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
                    "Prove live and replay convergence.",
                    session.clone(),
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        let live_update = updates
            .recorded()
            .into_iter()
            .last()
            .expect("completion transcript update");
        let live_projection = service
            .projection_update_for_transcript(&live_update)
            .expect("live projection update");
        let replayed_projection = service
            .replay_conversation_projection(&session.task_id())
            .expect("replayed projection");
        let live_assistant = live_projection
            .snapshot
            .transcript
            .entries
            .iter()
            .rev()
            .find(|entry| entry.speaker == ConversationTranscriptSpeaker::Assistant)
            .cloned()
            .expect("live assistant entry");
        let replayed_assistant = replayed_projection
            .transcript
            .entries
            .iter()
            .rev()
            .find(|entry| entry.speaker == ConversationTranscriptSpeaker::Assistant)
            .cloned()
            .expect("replayed assistant entry");

        assert_eq!(
            live_projection.snapshot.transcript,
            replayed_projection.transcript
        );
        assert_eq!(live_assistant, replayed_assistant);
        assert_eq!(
            live_assistant.response_mode,
            Some(ResponseMode::GroundedAnswer)
        );
        assert_eq!(live_assistant.render, Some(expected.document));
        assert_eq!(live_assistant.citations, vec!["README.md".to_string()]);
        assert_eq!(live_assistant.grounded, Some(true));
    }

    #[test]
    fn process_prompt_does_not_emit_generic_plan_updates_for_edit_turns() {
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

                    deliberation_state: None,                },
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

                    deliberation_state: None,                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder);
        bind_workspace_action_executor(&service, synthesizer.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer as Arc<dyn SynthesizerEngine>,
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
            plan_updates.is_empty(),
            "edit turns should not emit generic plan updates when the planner did not author any concrete remaining work"
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
                .all(|note| !note.contains("Execution checklist")),
            "follow-on planner requests should not receive generic execution checklist notes"
        );
    }

    #[tokio::test]
    async fn process_prompt_records_steering_control_and_hands_off_to_the_steered_prompt() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nTurn controls should stay attached to the active runtime.\n",
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
        let release_initial = Arc::new(tokio::sync::Notify::new());
        let planner = Arc::new(BlockingTurnControlPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "AGENTS.md".to_string(),
                    },
                },
                "read the operator contract before replying",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "steered path complete".to_string(),
                },
                rationale: "the steered prompt is complete".to_string(),
                answer: Some("Steered reply.".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            recorded_requests.clone(),
            ThreadDecisionPlan::continue_current(),
            release_initial.clone(),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let sink = Arc::new(RecordingTurnEventSink::default());

        *service.runtime.write().await = Some(ActiveRuntimeState {
            prepared,
            planner_engine: planner,
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        let process = service.process_prompt_in_session_with_sink(
            "original prompt",
            session.clone(),
            sink.clone(),
        );
        tokio::pin!(process);

        let mut steer_requested = false;
        loop {
            tokio::select! {
                result = &mut process, if !steer_requested => {
                    panic!("process prompt completed before steering could attach: {result:?}");
                }
                _ = tokio::task::yield_now(), if !steer_requested => {
                    if session.active_turn_id().is_some() {
                        session
                            .request_turn_steer("steer harder")
                            .expect("active turn should accept steering");
                        release_initial.notify_waiters();
                        steer_requested = true;
                    }
                }
            }
            if steer_requested {
                break;
            }
        }

        let reply = process.await.expect("process prompt");
        assert_eq!(reply, "Steered reply.");
        assert_eq!(session.active_turn_id(), None);

        let prompts = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .iter()
            .map(|request| request.user_prompt.clone())
            .collect::<Vec<_>>();
        assert_eq!(prompts.first().map(String::as_str), Some("original prompt"));
        assert!(
            prompts
                .iter()
                .skip(1)
                .all(|prompt| prompt == "steer harder"),
            "all follow-on planner requests should use the steered prompt: {prompts:?}"
        );

        let replay = recorder.replay(&session.task_id()).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ControlResultRecorded(result)
                if result.operation == ControlOperation::Turn(TurnControlOperation::Steer)
                    && result.status == ControlResultStatus::Applied
        )));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ThreadDecisionSelected(decision)
                if decision.kind == ThreadDecisionKind::ContinueCurrent
        )));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::CompletionCheckpoint(checkpoint)
                if checkpoint.response.is_none()
                    && checkpoint.summary.contains("turn handed off to steered prompt")
        )));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::ControlStateChanged { result }
                if result.operation == ControlOperation::Turn(TurnControlOperation::Steer)
                    && result.status == ControlResultStatus::Applied
        )));
    }

    #[tokio::test]
    async fn process_prompt_interrupts_at_a_safe_checkpoint_and_records_the_control_result() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nInterrupts should stop planned turns at safe checkpoints.\n",
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
        let release_initial = Arc::new(tokio::sync::Notify::new());
        let planner = Arc::new(BlockingTurnControlPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "AGENTS.md".to_string(),
                    },
                },
                "read the operator contract before replying",
            ),
            Vec::new(),
            recorded_requests.clone(),
            ThreadDecisionPlan::continue_current(),
            release_initial.clone(),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();

        *service.runtime.write().await = Some(ActiveRuntimeState {
            prepared,
            planner_engine: planner,
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        let process = service.process_prompt_in_session_with_sink(
            "original prompt",
            session.clone(),
            Arc::new(RecordingTurnEventSink::default()),
        );
        tokio::pin!(process);

        let mut interrupt_requested = false;
        loop {
            tokio::select! {
                result = &mut process, if !interrupt_requested => {
                    panic!("process prompt completed before interrupt could attach: {result:?}");
                }
                _ = tokio::task::yield_now(), if !interrupt_requested => {
                    if session.active_turn_id().is_some() {
                        session
                            .request_turn_interrupt()
                            .expect("active turn should accept interrupt");
                        release_initial.notify_waiters();
                        interrupt_requested = true;
                    }
                }
            }
            if interrupt_requested {
                break;
            }
        }

        let reply = process.await.expect("process prompt");
        assert_eq!(reply, "Interrupted the active turn at a safe checkpoint.");
        assert_eq!(session.active_turn_id(), None);
        assert_eq!(
            recorded_requests
                .lock()
                .expect("recorded requests lock")
                .iter()
                .map(|request| request.user_prompt.clone())
                .collect::<Vec<_>>(),
            vec!["original prompt".to_string()]
        );

        let replay = recorder.replay(&session.task_id()).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ControlResultRecorded(result)
                if result.operation == ControlOperation::Turn(TurnControlOperation::Interrupt)
                    && result.status == ControlResultStatus::Applied
        )));
        assert!(
            !replay
                .records
                .iter()
                .any(|record| matches!(&record.kind, TraceRecordKind::ThreadDecisionSelected(_)))
        );
    }

    #[tokio::test]
    async fn later_interrupts_mark_older_turn_controls_stale_instead_of_mutating_hidden_state() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nSuperseded turn controls must degrade honestly.\n",
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
        let release_initial = Arc::new(tokio::sync::Notify::new());
        let planner = Arc::new(BlockingTurnControlPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "AGENTS.md".to_string(),
                    },
                },
                "read the operator contract before replying",
            ),
            Vec::new(),
            Arc::new(Mutex::new(Vec::new())),
            ThreadDecisionPlan::continue_current(),
            release_initial.clone(),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();

        *service.runtime.write().await = Some(ActiveRuntimeState {
            prepared,
            planner_engine: planner,
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        let process = service.process_prompt_in_session_with_sink(
            "original prompt",
            session.clone(),
            Arc::new(RecordingTurnEventSink::default()),
        );
        tokio::pin!(process);

        let mut controls_sent = false;
        loop {
            tokio::select! {
                result = &mut process, if !controls_sent => {
                    panic!("process prompt completed before stale-control proof could attach: {result:?}");
                }
                _ = tokio::task::yield_now(), if !controls_sent => {
                    if session.active_turn_id().is_some() {
                        session
                            .request_turn_steer("steer harder")
                            .expect("active turn should accept steering");
                        session
                            .request_turn_interrupt()
                            .expect("active turn should accept interrupt");
                        release_initial.notify_waiters();
                        controls_sent = true;
                    }
                }
            }
            if controls_sent {
                break;
            }
        }

        let reply = process.await.expect("process prompt");
        assert_eq!(reply, "Interrupted the active turn at a safe checkpoint.");

        let replay = recorder.replay(&session.task_id()).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ControlResultRecorded(result)
                if result.operation == ControlOperation::Turn(TurnControlOperation::Steer)
                    && result.status == ControlResultStatus::Stale
        )));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ControlResultRecorded(result)
                if result.operation == ControlOperation::Turn(TurnControlOperation::Interrupt)
                    && result.status == ControlResultStatus::Applied
        )));
    }

    #[tokio::test]
    async fn steering_into_a_child_thread_preserves_replayable_thread_lineage() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nChild-thread handoffs should stay replayable.\n",
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
        let release_initial = Arc::new(tokio::sync::Notify::new());
        let planner = Arc::new(BlockingTurnControlPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "AGENTS.md".to_string(),
                    },
                },
                "read the operator contract before replying",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "child thread complete".to_string(),
                },
                rationale: "the child-thread steer completed".to_string(),
                answer: Some("Child thread reply.".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
            ThreadDecisionPlan::open_child("investigate"),
            release_initial.clone(),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();

        *service.runtime.write().await = Some(ActiveRuntimeState {
            prepared,
            planner_engine: planner,
            synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
            gatherer: None,
        });

        let process = service.process_prompt_in_session_with_sink(
            "original prompt",
            session.clone(),
            Arc::new(RecordingTurnEventSink::default()),
        );
        tokio::pin!(process);

        let mut steer_requested = false;
        loop {
            tokio::select! {
                result = &mut process, if !steer_requested => {
                    panic!("process prompt completed before child-thread steering could attach: {result:?}");
                }
                _ = tokio::task::yield_now(), if !steer_requested => {
                    if session.active_turn_id().is_some() {
                        session
                            .request_turn_steer("follow the branch")
                            .expect("active turn should accept steering");
                        release_initial.notify_waiters();
                        steer_requested = true;
                    }
                }
            }
            if steer_requested {
                break;
            }
        }

        let reply = process.await.expect("process prompt");
        assert_eq!(reply, "Child thread reply.");

        let replay = recorder.replay(&session.task_id()).expect("replay");
        let child_branch_id = replay
            .records
            .iter()
            .find_map(|record| match &record.kind {
                TraceRecordKind::PlannerBranchDeclared(branch) => Some(branch.branch_id.clone()),
                _ => None,
            })
            .expect("child branch should be declared");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::ThreadDecisionSelected(decision)
                if decision.kind == ThreadDecisionKind::OpenChildThread
        )));
        assert!(replay.records.iter().any(|record| {
            record.lineage.branch_id.as_ref() == Some(&child_branch_id)
                && matches!(record.kind, TraceRecordKind::TurnStarted(_))
        }));
        let replay_view = ConversationReplayView::from_trace_replay(&replay);
        assert_eq!(replay_view.threads.len(), 2);
    }

    #[test]
    fn structured_turn_trace_records_worker_coordination_records_across_parent_and_worker_threads()
    {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let turn_id = session.allocate_turn_id();
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            recorder.clone(),
            Vec::new(),
            session.clone(),
            turn_id,
            ConversationThreadRef::Mainline,
        ));
        let worker_branch = session.next_branch_id();
        let worker_thread = ConversationThreadRef::Branch(worker_branch.clone());
        trace.declare_branch(
            worker_branch.clone(),
            "delegated parser audit",
            Some("parallel bounded work"),
            None,
        );

        let contract = crate::domain::model::WorkerDelegationContract::new(
            crate::domain::model::WorkerRole::new(
                "worker",
                "Worker",
                "Audit parser lineage and report bounded findings.",
            ),
            crate::domain::model::WorkerOwnership::new(
                "Own parser lineage traces",
                vec!["src/domain/model".to_string()],
                vec!["src/domain/model/delegation.rs".to_string()],
                crate::domain::model::DelegationIntegrationOwner::Parent,
            ),
            crate::domain::model::DelegationGovernancePolicy::inherit_from_parent(
                &ExecutionGovernanceSnapshot::new(
                    "recursive-structured-v1",
                    "recursive-structured-v1",
                    crate::domain::model::ExecutionGovernanceProfile::new(
                        ExecutionSandboxMode::WorkspaceWrite,
                        ExecutionApprovalPolicy::OnRequest,
                        vec![
                            crate::domain::model::ExecutionPermissionReuseScope::Turn,
                            crate::domain::model::ExecutionPermissionReuseScope::Hand,
                        ],
                        None,
                    ),
                ),
                crate::domain::model::DelegationEvidencePolicy::new(
                    "Worker execution stays parent-visible.",
                    vec![
                        crate::domain::model::WorkerArtifactKind::ToolCall,
                        crate::domain::model::WorkerArtifactKind::ToolOutput,
                        crate::domain::model::WorkerArtifactKind::CompletionSummary,
                    ],
                ),
            ),
        );

        trace.record_worker_lifecycle(
            &ConversationThreadRef::Mainline,
            &worker_thread,
            crate::domain::model::WorkerDelegationRequest::spawn("Audit parser lineage", contract),
            crate::domain::model::WorkerLifecycleResult::new(
                crate::domain::model::WorkerLifecycleOperation::Spawn,
                crate::domain::model::WorkerLifecycleResultStatus::Accepted,
                Some("worker-1".to_string()),
                "Spawned worker-1 on a child thread.",
            ),
        );
        trace.record_worker_lifecycle(
            &ConversationThreadRef::Mainline,
            &worker_thread,
            crate::domain::model::WorkerDelegationRequest::wait("worker-1"),
            crate::domain::model::WorkerLifecycleResult::new(
                crate::domain::model::WorkerLifecycleOperation::Wait,
                crate::domain::model::WorkerLifecycleResultStatus::Accepted,
                Some("worker-1".to_string()),
                "Parent waited for worker-1 to reach a checkpoint.",
            ),
        );
        let _tool_call = trace.record_worker_artifact(
            &worker_thread,
            crate::domain::model::WorkerArtifactRecord::tool_call(
                "worker-1",
                "shell",
                "rg parser src/domain/model",
            ),
            "rg parser src/domain/model",
        );
        let _tool_output = trace.record_worker_artifact(
            &worker_thread,
            crate::domain::model::WorkerArtifactRecord::tool_output(
                "worker-1",
                "shell",
                "Found parser lineage records.",
            ),
            "Found parser lineage records.",
        );
        trace.record_worker_lifecycle(
            &ConversationThreadRef::Mainline,
            &worker_thread,
            crate::domain::model::WorkerDelegationRequest::resume("worker-1"),
            crate::domain::model::WorkerLifecycleResult::new(
                crate::domain::model::WorkerLifecycleOperation::Resume,
                crate::domain::model::WorkerLifecycleResultStatus::Accepted,
                Some("worker-1".to_string()),
                "Parent resumed after the worker checkpoint.",
            ),
        );
        let completion = trace.record_worker_artifact(
            &worker_thread,
            crate::domain::model::WorkerArtifactRecord::completion_summary(
                "worker-1",
                "Parser lineage audit complete",
                vec!["Integrate the parser findings into the main thread.".to_string()],
            ),
            "Parser lineage audit complete",
        );
        trace.record_worker_integration(
            &ConversationThreadRef::Mainline,
            &worker_thread,
            "worker-1",
            crate::domain::model::WorkerIntegrationStatus::Integrated,
            "Integrated the worker findings into the parent turn.",
            vec![completion],
        );

        let replay = recorder.replay(&session.task_id()).expect("replay");
        let delegation = crate::domain::model::DelegationReplayView::from_trace_replay(&replay);
        let worker = delegation.workers.first().expect("worker");

        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::WorkerLifecycleRecorded(_)
        ) && record.lineage.branch_id.is_none()));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::WorkerArtifactRecorded(_)
        ) && record.lineage.branch_id.as_ref()
            == Some(&worker_branch)));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::WorkerIntegrationRecorded(_)
        ) && record.lineage.branch_id.is_none()));
        assert_eq!(
            worker.status,
            crate::domain::model::DelegatedWorkerStatus::Integrated
        );
        assert_eq!(worker.artifacts.len(), 3);
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

                deliberation_state: None,
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

            deliberation_state: None,
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
            &DeliberationSignals::default(),
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

            deliberation_state: None,
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
            &DeliberationSignals::default(),
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

            deliberation_state: None,
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
            &DeliberationSignals::default(),
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

                    deliberation_state: None,
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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "enough information".to_string(),
                    },
                    rationale: "stop after acting".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
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
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "the file was enough".to_string(),
                    },
                    rationale: "stop after the read".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "done".to_string(),
                    },
                    rationale: "stop".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
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
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                deliberation_state: None,
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

                deliberation_state: None,
            },
            super::DecisionReviewFrame {
                deliberation_signals: &DeliberationSignals::default(),
                workspace_root: Path::new("/workspace"),
                trace: Arc::new(StructuredTurnTrace::new(
                    Arc::new(RecordingTurnEventSink::default()),
                    Arc::new(InMemoryTraceRecorder::default()),
                    Vec::new(),
                    ConversationSession::new(
                        TaskTraceId::new("task-review").expect("task trace id"),
                    ),
                    crate::domain::model::TurnTraceId::new("turn").expect("turn id"),
                    ConversationThreadRef::Mainline,
                )),
            },
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
        let context = test_planner_loop_context(InitialEditInstruction {
            known_edit: true,
            candidate_files: vec!["src/application/mod.rs".to_string()],
            resolution: Some(resolution.clone()),
        });
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                        deliberation_state: None,
                    }),
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

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());
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

                deliberation_state: None,
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
    fn failed_shell_actions_stay_in_the_loop_as_evidence_for_recovery() {
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
        let answer =
            "The shell command failed; inspect the hook output and adjust the commit step instead of ending the turn."
                .to_string();
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Shell {
                        command: "false".to_string(),
                    },
                },
                "run the shell command first so the harness can reason from the exact failure",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "failure captured and next steps are clear".to_string(),
                },
                rationale: "the failed shell output is enough to decide the next bounded step"
                    .to_string(),
                answer: Some(answer.clone()),
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        let sink = Arc::new(RecordingTurnEventSink::default());

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let _reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_with_sink(
                    "Run the shell command and keep reasoning if it fails.",
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(
            requests.len() >= 2,
            "a failed shell action should still permit a follow-on planner decision"
        );
        assert!(
            requests
                .iter()
                .skip(1)
                .flat_map(|request| request.loop_state.evidence_items.iter())
                .any(|item| item.snippet.contains("Tool `shell` failed")
                    && item.snippet.contains("Exit status")),
            "the failed shell action should remain available as evidence for the next decision"
        );

        let events = sink.recorded();
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::ToolFinished { tool_name, summary, .. }
                if tool_name == "shell" && summary.contains("Tool `shell` failed")
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerSummary {
                stop_reason: Some(reason),
                ..
            } if reason == "failure captured and next steps are clear"
        )));
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

                deliberation_state: None,
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
    fn planner_workspace_actions_emit_governance_decision_events() {
        #[derive(Default)]
        struct GovernedWriteSynthesizer;

        impl SynthesizerEngine for GovernedWriteSynthesizer {
            fn set_verbose(&self, _level: u8) {}

            fn respond_for_turn(
                &self,
                _prompt: &str,
                _turn_intent: TurnIntent,
                _gathered_evidence: Option<&EvidenceBundle>,
                _handoff: &SynthesisHandoff,
                _event_sink: Arc<dyn TurnEventSink>,
            ) -> Result<String> {
                Ok("done".to_string())
            }

            fn recent_turn_summaries(&self) -> Result<Vec<String>> {
                Ok(Vec::new())
            }
        }

        impl WorkspaceActionExecutor for GovernedWriteSynthesizer {
            fn execute_workspace_action(
                &self,
                action: &WorkspaceAction,
                _frame: WorkspaceActionExecutionFrame<'_>,
            ) -> Result<crate::domain::ports::WorkspaceActionResult> {
                Ok(crate::domain::ports::WorkspaceActionResult {
                    name: action.label().to_string(),
                    summary: format!("executed {}", action.summary()),
                    applied_edit: None,
                    governance_request: Some(ExecutionPermissionRequest::new(
                        ExecutionHandKind::WorkspaceEditor,
                        ExecutionPermissionRequirement::new(
                            "write file",
                            vec![
                                ExecutionPermission::ReadWorkspace,
                                ExecutionPermission::WriteWorkspace,
                            ],
                        ),
                    )),
                    governance_outcome: Some(ExecutionGovernanceOutcome::allowed(
                        "workspace write is allowed under the active sandbox",
                        ExecutionPermissionRequirement::new(
                            "write file",
                            vec![
                                ExecutionPermission::ReadWorkspace,
                                ExecutionPermission::WriteWorkspace,
                            ],
                        ),
                        vec![
                            ExecutionPermission::ReadWorkspace,
                            ExecutionPermission::WriteWorkspace,
                        ],
                    )),
                })
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

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(GovernedWriteSynthesizer);
        let service = test_service(workspace.path());
        service
            .set_workspace_action_executor(synthesizer.clone() as Arc<dyn WorkspaceActionExecutor>);
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
            TurnEvent::ExecutionGovernanceDecisionRecorded { decision }
                if decision.tool_name.as_deref() == Some("write_file")
                    && decision.request.hand == ExecutionHandKind::WorkspaceEditor
                    && decision.outcome.kind == ExecutionGovernanceOutcomeKind::Allowed
        )));
    }

    #[test]
    fn planner_workspace_actions_route_through_application_owned_executor_boundary() {
        #[derive(Default)]
        struct PanicOnWorkspaceActionSynthesizer;

        impl SynthesizerEngine for PanicOnWorkspaceActionSynthesizer {
            fn set_verbose(&self, _level: u8) {}

            fn respond_for_turn(
                &self,
                _prompt: &str,
                _turn_intent: TurnIntent,
                _gathered_evidence: Option<&EvidenceBundle>,
                _handoff: &SynthesisHandoff,
                _event_sink: Arc<dyn TurnEventSink>,
            ) -> Result<String> {
                Ok("executor boundary preserved".to_string())
            }

            fn recent_turn_summaries(&self) -> Result<Vec<String>> {
                Ok(Vec::new())
            }
        }

        #[derive(Default)]
        struct RecordingWorkspaceActionExecutor {
            actions: Mutex<Vec<WorkspaceAction>>,
        }

        impl WorkspaceActionExecutor for RecordingWorkspaceActionExecutor {
            fn execute_workspace_action(
                &self,
                action: &WorkspaceAction,
                _frame: WorkspaceActionExecutionFrame<'_>,
            ) -> Result<crate::domain::ports::WorkspaceActionResult> {
                self.actions
                    .lock()
                    .expect("workspace action executor actions lock")
                    .push(action.clone());
                Ok(crate::domain::ports::WorkspaceActionResult {
                    name: action.label().to_string(),
                    summary: format!("executed {}", action.summary()),
                    applied_edit: None,
                    governance_request: None,
                    governance_outcome: None,
                })
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
            gatherer: None,
        };
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "README.md".to_string(),
                    },
                },
                "read the workspace artifact before answering",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "workspace read was enough".to_string(),
                },
                rationale: "stop after the bounded action".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(PanicOnWorkspaceActionSynthesizer);
        let executor = Arc::new(RecordingWorkspaceActionExecutor::default());
        let service = test_service(workspace.path());
        service.set_workspace_action_executor(executor.clone() as Arc<dyn WorkspaceActionExecutor>);

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt("read the repo state first")
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "executor boundary preserved");
        assert_eq!(
            executor
                .actions
                .lock()
                .expect("workspace action executor actions lock")
                .as_slice(),
            [WorkspaceAction::Read {
                path: "README.md".to_string(),
            }]
        );
    }

    #[test]
    fn structured_turn_trace_records_governance_profile_and_decision_artifacts() {
        let task_id = TaskTraceId::new("task-governance").expect("task");
        let session = ConversationSession::new(task_id.clone());
        let turn_id = session.allocate_turn_id();
        let active_thread = session.active_thread().thread_ref;
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let trace = Arc::new(StructuredTurnTrace::new(
            Arc::new(RecordingTurnEventSink::default()),
            recorder.clone(),
            Vec::new(),
            session,
            turn_id,
            active_thread,
        ));

        trace.emit(TurnEvent::ExecutionGovernanceProfileApplied {
            snapshot: ExecutionGovernanceSnapshot::new(
                "recursive-structured-v1",
                "recursive-structured-v1",
                crate::domain::model::ExecutionGovernanceProfile::new(
                    ExecutionSandboxMode::WorkspaceWrite,
                    ExecutionApprovalPolicy::OnRequest,
                    Vec::new(),
                    None,
                ),
            ),
        });
        trace.emit(TurnEvent::ExecutionGovernanceDecisionRecorded {
            decision: ExecutionGovernanceDecision::new(
                Some("tool-1".to_string()),
                Some("write_file".to_string()),
                ExecutionPermissionRequest::new(
                    ExecutionHandKind::WorkspaceEditor,
                    ExecutionPermissionRequirement::new(
                        "write file",
                        vec![
                            ExecutionPermission::ReadWorkspace,
                            ExecutionPermission::WriteWorkspace,
                        ],
                    ),
                ),
                ExecutionGovernanceOutcome::allowed(
                    "workspace write is allowed under the active sandbox",
                    ExecutionPermissionRequirement::new(
                        "write file",
                        vec![
                            ExecutionPermission::ReadWorkspace,
                            ExecutionPermission::WriteWorkspace,
                        ],
                    ),
                    vec![
                        ExecutionPermission::ReadWorkspace,
                        ExecutionPermission::WriteWorkspace,
                    ],
                ),
            ),
        });

        let replay = recorder.replay(&task_id).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            record.kind,
            TraceRecordKind::ExecutionGovernanceProfileDeclared(_)
        )));
        assert!(replay.records.iter().any(|record| matches!(
            record.kind,
            TraceRecordKind::ExecutionGovernanceDecisionRecorded(_)
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

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "describe the requested edit".to_string(),
                    answer: Some("Add `padding: 8px;` to `.runtime-shell-host`.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,                },
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

                    deliberation_state: None,                },
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

                    deliberation_state: None,                },
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

                    deliberation_state: None,                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested commit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the requested edit is complete".to_string(),
                    answer: Some(answer.clone()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,

                    deliberation_state: None,
                },
            ],
            recorded_requests.clone(),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());
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
            plan_updates.is_empty(),
            "replanning should not reintroduce synthetic checklist plan updates"
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
    fn planning_mode_explores_read_only_then_requests_bounded_clarification_before_mutation() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "README.md".to_string(),
                    },
                },
                "inspect the target file before changing it",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::WriteFile {
                        path: "README.md".to_string(),
                        content: "# Updated Workspace\n".to_string(),
                    },
                },
                rationale: "apply the requested change".to_string(),
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["README.md".to_string()],
                    resolution: None,
                },
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_mode_request_and_sink(
                    "Plan the change before you edit README.md",
                    session,
                    Some(CollaborationModeRequest::new(
                        CollaborationModeRequestTarget::Known(CollaborationMode::Planning),
                        CollaborationModeRequestSource::OperatorSurface,
                        Some("operator selected planning mode".to_string()),
                    )),
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert!(reply.contains("Need clarification before mutating"));
        assert!(reply.contains("stay_in_planning"));
        assert!(reply.contains("switch_to_execution"));
        assert_eq!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .clone(),
            vec![WorkspaceAction::Read {
                path: "README.md".to_string(),
            }],
            "planning mode should preserve read-only exploration and fail closed before mutation"
        );
        assert!(
            synthesizer
                .handoffs
                .lock()
                .expect("handoffs lock")
                .is_empty(),
            "clarification should stop before synthesis"
        );
        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert_eq!(
            requests[0].collaboration.active.mode,
            CollaborationMode::Planning
        );
        assert_eq!(
            requests[1].collaboration.active.mode,
            CollaborationMode::Planning
        );
    }

    #[test]
    fn review_mode_bootstraps_local_diff_and_carries_findings_first_handoff() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(InitialAction::Answer, "reply directly"),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "local diff evidence is sufficient for review synthesis".to_string(),
                },
                rationale: "hand off the review after diff inspection".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: synthesizer.clone(),
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_mode_request_and_sink(
                    "Review the current local changes",
                    session,
                    Some(CollaborationModeRequest::new(
                        CollaborationModeRequestTarget::Known(CollaborationMode::Review),
                        CollaborationModeRequestSource::OperatorSurface,
                        Some("operator selected review mode".to_string()),
                    )),
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Applied the bounded action.");
        assert_eq!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .first()
                .cloned(),
            Some(WorkspaceAction::Diff { path: None }),
            "review mode should inspect the local diff before synthesis"
        );
        let handoffs = synthesizer.handoffs.lock().expect("handoffs lock").clone();
        let handoff = handoffs.last().expect("review handoff");
        assert_eq!(handoff.collaboration.active.mode, CollaborationMode::Review);
        assert_eq!(
            handoff.collaboration.active.output_contract.label(),
            "findings_first_review"
        );
        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert_eq!(
            requests[0].collaboration.active.mode,
            CollaborationMode::Review
        );
    }

    #[test]
    fn planning_mode_records_mode_selection_and_structured_clarification_for_replay() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "README.md".to_string(),
                    },
                },
                "inspect the target file before changing it",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::WriteFile {
                        path: "README.md".to_string(),
                        content: "# Updated Workspace\n".to_string(),
                    },
                },
                rationale: "apply the requested change".to_string(),
                answer: None,
                edit: InitialEditInstruction {
                    known_edit: true,
                    candidate_files: vec!["README.md".to_string()],
                    resolution: None,
                },
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let reply = runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_mode_request_and_sink(
                    "Plan the change before you edit README.md",
                    session.clone(),
                    Some(CollaborationModeRequest::new(
                        CollaborationModeRequestTarget::Known(CollaborationMode::Planning),
                        CollaborationModeRequestSource::OperatorSurface,
                        Some("operator selected planning mode".to_string()),
                    )),
                    sink.clone(),
                )
                .await
                .expect("process prompt")
        });

        assert!(reply.contains("Need clarification before mutating"));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::CollaborationModeChanged { result }
                if result.active.mode == CollaborationMode::Planning
                    && result.status == crate::domain::model::CollaborationModeResultStatus::Applied
        )));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::StructuredClarificationChanged { result }
                if result.status == StructuredClarificationStatus::Requested
                    && result.request.clarification_id == "planning-mode-clarification"
        )));

        let replay = recorder.replay(&session.task_id()).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::CollaborationModeDeclared(result)
                if result.active.mode == CollaborationMode::Planning
        )));
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::StructuredClarificationRecorded(result)
                if result.status == StructuredClarificationStatus::Requested
                    && result.request.clarification_id == "planning-mode-clarification"
        )));
    }

    #[test]
    fn unsupported_mode_requests_emit_typed_collaboration_results_instead_of_silent_fallback() {
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
            initial_action_decision(InitialAction::Answer, "reply directly"),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "no recursion needed".to_string(),
                },
                rationale: "finish the turn".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::new(Mutex::new(Vec::new())),
        ));
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let sink = Arc::new(RecordingTurnEventSink::default());
        let session = service.shared_conversation_session();

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
                gatherer: None,
            });
            service
                .process_prompt_in_session_with_mode_request_and_sink(
                    "Review this workspace",
                    session.clone(),
                    Some(CollaborationModeRequest::new(
                        CollaborationModeRequestTarget::Unsupported("pairing".to_string()),
                        CollaborationModeRequestSource::OperatorSurface,
                        Some("unsupported request".to_string()),
                    )),
                    sink.clone(),
                )
                .await
                .expect("process prompt");
        });

        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::CollaborationModeChanged { result }
                if result.active.mode == CollaborationMode::Execution
                    && result.status == crate::domain::model::CollaborationModeResultStatus::Invalid
                    && result.detail.contains("unsupported collaboration mode `pairing`")
        )));

        let replay = recorder.replay(&session.task_id()).expect("replay");
        assert!(replay.records.iter().any(|record| matches!(
            &record.kind,
            TraceRecordKind::CollaborationModeDeclared(result)
                if result.active.mode == CollaborationMode::Execution
                    && result.status == crate::domain::model::CollaborationModeResultStatus::Invalid
        )));
    }

    #[test]
    fn execution_mode_remains_the_default_mutation_path() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::WriteFile {
                        path: "README.md".to_string(),
                        content: "# Updated Workspace\n".to_string(),
                    },
                },
                "apply the requested edit",
            ),
            vec![RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "the edit is complete".to_string(),
                },
                rationale: "handoff after the change".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,

                deliberation_state: None,
            }],
            Arc::clone(&recorded_requests),
        ));
        let synthesizer = Arc::new(RecordingSynthesizer::default());
        let service = test_service(workspace.path());
        bind_workspace_action_executor(&service, synthesizer.clone());

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
                    "Update README.md",
                    Arc::new(RecordingTurnEventSink::default()),
                )
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Applied the bounded action.");
        assert_eq!(
            synthesizer
                .executed_actions
                .lock()
                .expect("executed actions lock")
                .first()
                .cloned(),
            Some(WorkspaceAction::WriteFile {
                path: "README.md".to_string(),
                content: "# Updated Workspace\n".to_string(),
            }),
            "execution mode should remain the default mutation lane"
        );
        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert_eq!(
            requests[0].collaboration.active.mode,
            CollaborationMode::Execution
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

                deliberation_state: None,
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

                deliberation_state: None,        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &loop_state,
            &decision,
            Path::new("/workspace"),
            &DeliberationSignals::default(),
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

            deliberation_state: None,
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &loop_state,
            &decision,
            Path::new("/workspace"),
            &DeliberationSignals::default(),
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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

                    deliberation_state: None,
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
    fn recent_turn_summaries_prefer_session_context_slice_before_history_or_synth_fallback() {
        #[derive(Default)]
        struct StaticHistorySynthesizer;

        impl SynthesizerEngine for StaticHistorySynthesizer {
            fn set_verbose(&self, _level: u8) {}

            fn respond_for_turn(
                &self,
                _prompt: &str,
                _turn_intent: TurnIntent,
                _gathered_evidence: Option<&EvidenceBundle>,
                _handoff: &SynthesisHandoff,
                _event_sink: Arc<dyn TurnEventSink>,
            ) -> Result<String> {
                Ok("unused".to_string())
            }

            fn recent_turn_summaries(&self) -> Result<Vec<String>> {
                Ok(vec!["synth fallback".to_string()])
            }
        }

        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let history_store = Arc::new(ConversationHistoryStore::with_path(
            workspace.path().join("state/conversation-history.toml"),
        ));
        history_store
            .record_turn("old prompt", "old reply")
            .expect("record history turn");
        service.set_conversation_history_store(Arc::clone(&history_store));

        let session = service.shared_conversation_session();
        record_completed_turn(
            recorder.as_ref(),
            &session.task_id(),
            "task-root",
            None,
            1,
            "fresh prompt",
            Some("fresh reply"),
        );

        let summaries = service
            .recent_turn_summaries(&session, &StaticHistorySynthesizer)
            .expect("session recent turns");

        assert_eq!(
            summaries,
            vec!["Q: fresh prompt A: fresh reply".to_string()]
        );
    }

    fn record_completed_turn(
        recorder: &dyn TraceRecorder,
        task_id: &TaskTraceId,
        turn_suffix: &str,
        parent_record_id: Option<&str>,
        sequence_start: u64,
        prompt: &str,
        reply: Option<&str>,
    ) {
        let turn_id = if turn_suffix == "task-root" {
            paddles_conversation::TurnTraceId::new(format!("{}.turn-0001", task_id.as_str()))
                .expect("turn id")
        } else {
            paddles_conversation::TurnTraceId::new(turn_suffix).expect("turn id")
        };
        let prompt_record_id =
            paddles_conversation::TraceRecordId::new(format!("{turn_suffix}.record-0001"))
                .expect("prompt record id");
        let parent_record_id = parent_record_id
            .map(|id| paddles_conversation::TraceRecordId::new(id).expect("parent record id"));
        let prompt_kind = if sequence_start == 1 {
            TraceRecordKind::TaskRootStarted(crate::domain::model::TraceTaskRoot {
                prompt: crate::domain::model::ArtifactEnvelope::text(
                    paddles_conversation::TraceArtifactId::new(format!(
                        "{turn_suffix}.artifact.prompt"
                    ))
                    .expect("artifact"),
                    crate::domain::model::ArtifactKind::Prompt,
                    "prompt",
                    prompt,
                    256,
                ),
                interpretation: None,
                planner_model: "planner".to_string(),
                synthesizer_model: "synth".to_string(),
                harness_profile: crate::domain::model::TraceHarnessProfileSelection {
                    requested_profile_id: "recursive-structured-v1".to_string(),
                    active_profile_id: "recursive-structured-v1".to_string(),
                    downgrade_reason: None,
                },
            })
        } else {
            TraceRecordKind::TurnStarted(crate::domain::model::TraceTurnStarted {
                prompt: crate::domain::model::ArtifactEnvelope::text(
                    paddles_conversation::TraceArtifactId::new(format!(
                        "{turn_suffix}.artifact.prompt"
                    ))
                    .expect("artifact"),
                    crate::domain::model::ArtifactKind::Prompt,
                    "prompt",
                    prompt,
                    256,
                ),
                interpretation: None,
                planner_model: "planner".to_string(),
                synthesizer_model: "synth".to_string(),
                harness_profile: crate::domain::model::TraceHarnessProfileSelection {
                    requested_profile_id: "recursive-structured-v1".to_string(),
                    active_profile_id: "recursive-structured-v1".to_string(),
                    downgrade_reason: None,
                },
                thread: crate::domain::model::ConversationThreadRef::Mainline,
            })
        };
        recorder
            .record(crate::domain::model::TraceRecord {
                record_id: prompt_record_id.clone(),
                sequence: sequence_start,
                lineage: crate::domain::model::TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id,
                },
                kind: prompt_kind,
            })
            .expect("record prompt");

        if let Some(reply) = reply {
            recorder
                .record(crate::domain::model::TraceRecord {
                    record_id: paddles_conversation::TraceRecordId::new(format!(
                        "{turn_suffix}.record-0002"
                    ))
                    .expect("reply record id"),
                    sequence: sequence_start + 1,
                    lineage: crate::domain::model::TraceLineage {
                        task_id: task_id.clone(),
                        turn_id,
                        branch_id: None,
                        parent_record_id: Some(prompt_record_id),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(
                        crate::domain::model::TraceCompletionCheckpoint {
                            checkpoint_id: paddles_conversation::TraceCheckpointId::new(format!(
                                "{turn_suffix}.checkpoint"
                            ))
                            .expect("checkpoint"),
                            kind: crate::domain::model::TraceCheckpointKind::TurnCompleted,
                            summary: "done".to_string(),
                            response: Some(crate::domain::model::ArtifactEnvelope::text(
                                paddles_conversation::TraceArtifactId::new(format!(
                                    "{turn_suffix}.artifact.reply"
                                ))
                                .expect("artifact"),
                                crate::domain::model::ArtifactKind::ModelOutput,
                                "reply",
                                reply,
                                256,
                            )),
                            authored_response: Some(
                                crate::domain::model::AuthoredResponse::from_plain_text(
                                    crate::domain::model::ResponseMode::GroundedAnswer,
                                    reply,
                                ),
                            ),
                            citations: Vec::new(),
                            grounded: true,
                        },
                    ),
                })
                .expect("record reply");
        }
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
    fn planner_requests_include_specialist_brain_runtime_notes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        record_completed_turn(
            recorder.as_ref(),
            &session.task_id(),
            "task-root",
            None,
            1,
            "fresh prompt",
            Some("fresh reply"),
        );
        service.trace_counter.store(3, Ordering::Relaxed);

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: None,
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Openai,
                model_id: "gpt-5.4".to_string(),
                paths: None,
            },
            gatherer: None,
        };
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "answer directly".to_string(),
                answer: Some("direct answer".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::clone(&recorded_requests),
        ));

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
                gatherer: None,
            });
            service
                .process_prompt("What changed?")
                .await
                .expect("process prompt");
        });

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        let request = requests.first().expect("initial planner request");
        assert!(request.runtime_notes.iter().any(|note| {
            note.contains("session-continuity-v1")
                && note.contains("reviewed 1 durable turn summary")
        }));
    }

    #[test]
    fn planner_requests_include_specialist_brain_fallback_for_prompt_envelope_safe_profiles() {
        let workspace = tempfile::tempdir().expect("workspace");
        let service = test_service(workspace.path());

        let prepared = PreparedRuntimeLanes {
            planner: PreparedModelLane {
                role: RuntimeLaneRole::Planner,
                provider: ModelProvider::Anthropic,
                model_id: "claude-sonnet-4-20250514".to_string(),
                paths: None,
            },
            synthesizer: PreparedModelLane {
                role: RuntimeLaneRole::Synthesizer,
                provider: ModelProvider::Sift,
                model_id: "qwen-1.5b".to_string(),
                paths: Some(sample_model_paths("synth")),
            },
            gatherer: None,
        };
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "answer directly".to_string(),
                answer: Some("direct answer".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            },
            Vec::new(),
            Arc::clone(&recorded_requests),
        ));

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        runtime.block_on(async {
            *service.runtime.write().await = Some(ActiveRuntimeState {
                prepared,
                planner_engine: planner,
                synthesizer_engine: Arc::new(RecordingSynthesizer::default()),
                gatherer: None,
            });
            service
                .process_prompt("What changed?")
                .await
                .expect("process prompt");
        });

        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        let request = requests.first().expect("initial planner request");
        assert!(request.runtime_notes.iter().any(|note| {
            note.contains("session-continuity-v1")
                && note.contains("unavailable")
                && note.contains("prompt-envelope-safe-v1")
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

            deliberation_state: None,
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
            &DeliberationSignals::default(),
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Execution
                && note.note.contains("Steering review [action-bias]")
                && note.note.contains("src/application/mod.rs")
        }));
    }

    #[test]
    fn continuation_signals_suspend_mid_loop_refinement() {
        let service = test_service(Path::new("/workspace"));
        let loop_state = PlannerLoopState {
            evidence_items: vec![EvidenceItem {
                source: "command: pwd".to_string(),
                snippet: "/workspace".to_string(),
                rationale: "existing evidence".to_string(),
                rank: 1,
            }],
            refinement_policy: crate::domain::ports::RefinementPolicy {
                trigger: crate::domain::ports::RefinementTrigger {
                    min_evidence_items: 1,
                    min_steps_without_new_evidence: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        let signals = DeliberationSignals {
            continuation: DeliberationSignal::Present(DeliberationContinuation {
                reusable_state: true,
                tool_results_required: true,
            }),
            ..DeliberationSignals::default()
        };

        let suspended = service.mid_loop_refinement_reason(3, &loop_state, 1, &signals);
        let default_reason =
            service.mid_loop_refinement_reason(3, &loop_state, 1, &DeliberationSignals::default());

        assert!(suspended.is_none());
        assert!(default_reason.is_some());
    }

    #[test]
    fn continuation_signals_request_steering_review_for_stop_actions() {
        let signals = DeliberationSignals {
            continuation: DeliberationSignal::Present(DeliberationContinuation {
                reusable_state: true,
                tool_results_required: true,
            }),
            uncertainty: DeliberationSignal::Unknown,
            ..DeliberationSignals::default()
        };

        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &PlannerLoopState::default(),
            &planner_decision(
                PlannerAction::Stop {
                    reason: "answer now".to_string(),
                },
                "stop after the first tool result",
            ),
            Path::new("/workspace"),
            &signals,
        );

        assert!(notes.iter().any(|note| {
            note.kind == super::SteeringReviewKind::Deliberation
                && note.note.contains("Steering review [deliberation-signals]")
                && note.note.contains("reusable continuation state")
        }));
    }

    #[test]
    fn explicit_none_signals_do_not_request_deliberation_review() {
        let notes = super::collect_steering_review_notes(
            &test_planner_loop_context(InitialEditInstruction::default()),
            &PlannerLoopState::default(),
            &planner_decision(
                PlannerAction::Stop {
                    reason: "answer now".to_string(),
                },
                "stop after the first tool result",
            ),
            Path::new("/workspace"),
            &DeliberationSignals::default(),
        );

        assert!(
            !notes
                .iter()
                .any(|note| { note.kind == super::SteeringReviewKind::Deliberation })
        );
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

    #[test]
    fn native_continuation_signals_retry_stop_decisions_on_the_active_path() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "pwd".to_string(),
                    },
                },
                "start with a local checkpoint",
            ),
            vec![
                RecursivePlannerDecision {
                    action: PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "README.md".to_string(),
                        },
                    },
                    rationale: "consume the active tool path once".to_string(),
                    answer: None,
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                    deliberation_state: Some(moonshot_continuation_state()),
                },
                planner_decision(
                    PlannerAction::Stop {
                        reason: "answer now".to_string(),
                    },
                    "the tool result should be enough",
                ),
                planner_decision(
                    PlannerAction::Workspace {
                        action: WorkspaceAction::Inspect {
                            command: "ls".to_string(),
                        },
                    },
                    "continue the active path before answering",
                ),
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the continued path is now sufficient".to_string(),
                    answer: Some("Continued the active path before answering.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                    deliberation_state: None,
                },
            ],
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
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_with_sink("Trace the current runtime state.", sink.clone())
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Continued the active path before answering.");
        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        let review_request = requests
            .iter()
            .find(|request| {
                request
                    .loop_state
                    .notes
                    .iter()
                    .any(|note| note.contains("Steering review [deliberation-signals]"))
            })
            .expect("deliberation steering review request should be recorded");
        assert!(review_request.loop_state.notes.iter().any(|note| {
            note.contains("reusable continuation state")
                && note.contains("before branching, refining, or stopping")
        }));

        let events = sink.recorded();
        let follow_up_event = events
            .iter()
            .find_map(|event| match event {
                TurnEvent::PlannerActionSelected {
                    sequence,
                    action,
                    rationale,
                    signal_summary,
                } if *sequence == 3 => Some((action, rationale, signal_summary)),
                _ => None,
            })
            .expect("signal-aware follow-up planner action event");
        assert_eq!(follow_up_event.0, "inspect `ls`");
        assert!(follow_up_event.1.contains("Paddles chose `inspect `ls``"));
        assert!(
            !follow_up_event
                .1
                .contains("inspect the current tool path before stopping")
        );
        assert!(matches!(
            follow_up_event.2.as_deref(),
            Some(summary)
                if summary.contains("continuation=tool_follow_up")
                    && summary.contains("uncertainty=opaque")
                    && !summary.contains("inspect the current tool path before stopping")
        ));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerStepProgress { step_number, action, .. }
                if *step_number == 3 && action.contains("inspect `ls`")
        )));
    }

    #[test]
    fn explicit_none_signals_allow_stop_without_deliberation_retry() {
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
        let recorded_requests = Arc::new(Mutex::new(Vec::new()));
        let planner = Arc::new(TestPlanner::new(
            initial_action_decision(
                InitialAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "pwd".to_string(),
                    },
                },
                "start with a local checkpoint",
            ),
            vec![
                planner_decision(
                    PlannerAction::Workspace {
                        action: WorkspaceAction::Read {
                            path: "README.md".to_string(),
                        },
                    },
                    "consume the active tool path once",
                ),
                RecursivePlannerDecision {
                    action: PlannerAction::Stop {
                        reason: "model selected answer".to_string(),
                    },
                    rationale: "the current path is sufficient".to_string(),
                    answer: Some("Stopped without a deliberation retry.".to_string()),
                    edit: InitialEditInstruction::default(),
                    grounding: None,
                    deliberation_state: None,
                },
            ],
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
                synthesizer_engine: synthesizer,
                gatherer: None,
            });
            service
                .process_prompt_with_sink("Trace the current runtime state.", sink.clone())
                .await
                .expect("process prompt")
        });

        assert_eq!(reply, "Stopped without a deliberation retry.");
        let requests = recorded_requests
            .lock()
            .expect("recorded requests lock")
            .clone();
        assert!(!requests.iter().any(|request| {
            request
                .loop_state
                .notes
                .iter()
                .any(|note| note.contains("Steering review [deliberation-signals]"))
        }));
        let events = sink.recorded();
        let terminal_event = events
            .iter()
            .find_map(|event| match event {
                TurnEvent::PlannerActionSelected {
                    sequence,
                    action,
                    rationale,
                    signal_summary,
                } if *sequence == 3 => Some((action, rationale, signal_summary)),
                _ => None,
            })
            .expect("terminal planner action event");
        assert_eq!(terminal_event.0, "stop (model selected answer)");
        assert!(terminal_event.1.contains("stop (model selected answer)"));
        assert_eq!(terminal_event.2, &None);
        assert!(!events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerStepProgress { step_number, action, .. }
                if *step_number == 3 && action.contains("inspect `ls`")
        )));
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
