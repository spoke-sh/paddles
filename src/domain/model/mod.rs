use anyhow::Result;

pub mod compaction;
pub mod context_quality;
pub mod execution_hand;
pub mod forensics;
pub mod generative;
pub mod harness;
pub mod harness_projector;
pub mod instructions;
pub mod interpretation;
pub mod manifold;
pub mod native_transport;
pub mod projection;
pub mod render;
pub mod runtime_events;
pub mod threading;
pub mod traces;
pub mod transcript;
pub mod turns;
pub use compaction::{CompactionBudget, CompactionDecision, CompactionPlan, CompactionRequest};
pub use context_quality::{ContextStrain, StrainFactor, StrainLevel, StrainTracker};
pub use execution_hand::{
    ExecutionHandAuthority, ExecutionHandDescriptor, ExecutionHandDiagnostic, ExecutionHandKind,
    ExecutionHandOperation, ExecutionHandPhase, default_local_execution_hand_descriptors,
};
pub use forensics::{
    ConversationForensicProjection, ConversationForensicUpdate, ForensicLifecycle,
    ForensicRecordProjection, ForensicTurnProjection, ForensicUpdateSink, NullForensicUpdateSink,
};
pub use generative::{AuthoredResponse, ResponseMode};
pub use harness::{
    GovernorState, HarnessChamber, HarnessSnapshot, HarnessStatus, TimeoutPhase, TimeoutState,
};
pub use harness_projector::derive_harness_snapshot;
pub use instructions::{
    InstructionDeliverable, InstructionFrame, InstructionIntent, InstructionObligation,
    InstructionSatisfaction,
};
pub use interpretation::{
    GuidanceCategory, InterpretationConflict, InterpretationContext,
    InterpretationCoverageConfidence, InterpretationDecisionFramework, InterpretationDocument,
    InterpretationProcedure, InterpretationProcedureStep, InterpretationToolHint, WorkspaceAction,
};
pub use manifold::{
    ConversationManifoldProjection, ManifoldConduitState, ManifoldFrame, ManifoldPrimitiveBasis,
    ManifoldPrimitiveKind, ManifoldPrimitiveState, ManifoldSignalState, ManifoldTurnProjection,
};
pub use native_transport::{
    NativeTransportAuth, NativeTransportAuthMode, NativeTransportCapability,
    NativeTransportChannel, NativeTransportConfiguration, NativeTransportConfigurations,
    NativeTransportDiagnostic, NativeTransportKind, NativeTransportPhase,
    NativeTransportSessionIdentity,
};
pub use paddles_conversation::{
    ArtifactEnvelope, ArtifactKind, ContextLocator, ContextTier, ConversationThread,
    ConversationThreadRef, ConversationThreadStatus, TaskTraceId, ThreadCandidate,
    ThreadCandidateId, ThreadDecision, ThreadDecisionId, ThreadDecisionKind, ThreadMergeMode,
    ThreadMergeRecord, TraceArtifactId, TraceBranchId, TraceCheckpointId, TraceRecordId,
    TurnTraceId,
};
pub use projection::{
    ConversationProjectionSnapshot, ConversationProjectionUpdate, ConversationProjectionUpdateKind,
    ConversationTraceGraph, ConversationTraceGraphBranch, ConversationTraceGraphEdge,
    ConversationTraceGraphNode,
};
pub use render::{RenderBlock, RenderDocument, SUPPORTED_RENDER_TYPES};
pub use runtime_events::{
    RuntimeEventPresentation, project_runtime_event, project_runtime_event_for_tui,
};
pub use threading::ConversationReplayView;
pub use traces::{
    SteeringGateKind, SteeringGatePhase, TraceBranch, TraceBranchStatus, TraceCheckpointKind,
    TraceCompletionCheckpoint, TraceLineage, TraceLineageEdge, TraceLineageNodeKind,
    TraceLineageNodeRef, TraceLineageRelation, TraceModelExchangeArtifact,
    TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase, TraceRecord,
    TraceRecordKind, TraceReplay, TraceSelectionArtifact, TraceSelectionKind,
    TraceSignalContribution, TraceSignalKind, TraceSignalSnapshot, TraceTaskRoot, TraceToolCall,
    TraceTurnStarted,
};
pub use transcript::{
    ConversationTranscript, ConversationTranscriptEntry, ConversationTranscriptSpeaker,
    ConversationTranscriptUpdate, NullTranscriptUpdateSink, TranscriptUpdateSink,
};
pub use turns::{
    AppliedEdit, ForensicArtifactCapture, ForensicTraceSink, MultiplexEventSink, NullTurnEventSink,
    PlanChecklistItem, PlanChecklistItemStatus, TurnEvent, TurnEventSink, TurnIntent,
};

/// Constitutional bounds for environmental calibration.
pub struct Constitution {
    pub min_weight: f64,
    pub max_weight: f64,
}

impl Default for Constitution {
    fn default() -> Self {
        Self {
            min_weight: 0.0,
            max_weight: 1.0,
        }
    }
}

impl Constitution {
    /// Validate if a weight is within constitutional bounds.
    pub fn validate(&self, weight: f64) -> Result<()> {
        if weight < self.min_weight || weight > self.max_weight {
            anyhow::bail!(
                "Calibration Failure: Weight {} is outside constitutional bounds [{}, {}].",
                weight,
                self.min_weight,
                self.max_weight
            );
        }
        Ok(())
    }
}

/// Religious dogmas (immutable invariants).
pub struct Dogma;

impl Dogma {
    /// Validate immutable invariants.
    pub fn validate(reality_mode: bool) -> Result<()> {
        if reality_mode {
            anyhow::bail!(
                "[UNCLEAN BOOT] Religious Violation: Simulation MUST take precedence over Reality."
            );
        }
        Ok(())
    }
}

/// Context established during the boot sequence.
pub struct BootContext {
    pub credits: u64,
    pub weight: f64,
    pub bias: f64,
    pub hf_token: Option<String>,
}

impl BootContext {
    /// Initialize and validate the boot context.
    pub fn new(
        credits: u64,
        weight: f64,
        bias: f64,
        hf_token: Option<String>,
        reality_mode: bool,
    ) -> Result<Self> {
        let constitution = Constitution::default();
        constitution.validate(weight)?;
        Dogma::validate(reality_mode)?;

        Ok(Self {
            credits,
            weight,
            bias,
            hf_token,
        })
    }
}
