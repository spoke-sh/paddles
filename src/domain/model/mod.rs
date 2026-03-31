use anyhow::Result;

pub mod compaction;
pub mod context_quality;
pub mod interpretation;
pub mod threading;
pub mod traces;
pub mod turns;
pub use compaction::{CompactionBudget, CompactionDecision, CompactionPlan, CompactionRequest};
pub use context_quality::{ContextPressure, PressureFactor, PressureLevel, PressureTracker};
pub use interpretation::{
    GuidanceCategory, InterpretationConflict, InterpretationContext,
    InterpretationCoverageConfidence, InterpretationDecisionFramework, InterpretationDocument,
    InterpretationProcedure, InterpretationProcedureStep, InterpretationToolHint, WorkspaceAction,
};
pub use paddles_conversation::{
    ArtifactEnvelope, ArtifactKind, ContextLocator, ContextTier, ConversationThread,
    ConversationThreadRef, ConversationThreadStatus, TaskTraceId, ThreadCandidate,
    ThreadCandidateId, ThreadDecision, ThreadDecisionId, ThreadDecisionKind, ThreadMergeMode,
    ThreadMergeRecord, TraceArtifactId, TraceBranchId, TraceCheckpointId, TraceRecordId,
    TurnTraceId,
};
pub use threading::ConversationReplayView;
pub use traces::{
    TraceBranch, TraceBranchStatus, TraceCheckpointKind, TraceCompletionCheckpoint, TraceLineage,
    TraceRecord, TraceRecordKind, TraceReplay, TraceSelectionArtifact, TraceSelectionKind,
    TraceTaskRoot, TraceToolCall, TraceTurnStarted,
};
pub use turns::{MultiplexEventSink, NullTurnEventSink, TurnEvent, TurnEventSink, TurnIntent};

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
