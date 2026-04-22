mod context_gathering;
mod context_resolution;
mod entity_resolution;
mod execution_hands;
mod external_capabilities;
mod operator_memory;
mod planning;
mod specialist_brains;
mod synthesis;
mod trace_recording;
mod workspace_action_execution;
mod workspace_editing;

use async_trait::async_trait;
use std::path::PathBuf;

pub use context_gathering::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBudget, EvidenceBundle,
    EvidenceItem, GathererCapability, PlannerConfig, PlannerDecision, PlannerGraphBranch,
    PlannerGraphBranchStatus, PlannerGraphEdge, PlannerGraphEdgeKind, PlannerGraphEpisode,
    PlannerGraphFrontierEntry, PlannerGraphNode, PlannerStrategyKind, PlannerTraceMetadata,
    PlannerTraceStep, RetainedEvidence, RetrievalMode, RetrievalStrategy, RetrieverOption,
};
pub use context_resolution::ContextResolver;
pub use entity_resolution::{
    EntityLookupMode, EntityResolutionCandidate, EntityResolutionOutcome, EntityResolutionRequest,
    EntityResolver, NormalizedEntityHint,
};
pub use execution_hands::ExecutionHand;
pub use external_capabilities::ExternalCapabilityBroker;
pub use operator_memory::OperatorMemory;
pub use planning::{
    CompactionPlan, CompactionRequest, GroundingDomain, GroundingRequirement, GuidanceCategory,
    InitialAction, InitialActionDecision, InitialEditInstruction, InterpretationConflict,
    InterpretationContext, InterpretationCoverageConfidence, InterpretationDecisionFramework,
    InterpretationDocument, InterpretationProcedure, InterpretationProcedureStep,
    InterpretationRequest, InterpretationToolHint, OperatorMemoryDocument, PlannerAction,
    PlannerBudget, PlannerCapability, PlannerDecision as RecursivePlannerDecision,
    PlannerLoopState, PlannerRequest, PlannerStepRecord, RecursivePlanner, RefinementPolicy,
    RefinementTrigger, RefinementTriggerSource, ThreadDecisionRequest, WorkspaceAction,
};
pub use specialist_brains::{
    SpecialistBrain, SpecialistBrainCapability, SpecialistBrainNote, SpecialistBrainRequest,
};
pub use synthesis::{SynthesisHandoff, SynthesizerEngine};
pub use trace_recording::{
    NoopTraceRecorder, TraceRecorder, TraceRecorderCapability, TraceReplaySlice,
    TraceReplaySliceAnchor, TraceReplaySliceDirection, TraceReplaySliceRequest,
    TraceSessionCheckpointCursor, TraceSessionContextQuery, TraceSessionContextSlice,
    TraceSessionWake,
};
pub use workspace_action_execution::{
    WorkspaceActionExecutionFrame, WorkspaceActionExecutor, WorkspaceActionResult,
};
pub use workspace_editing::WorkspaceEditor;

/// Port for model discovery and acquisition.
#[async_trait]
pub trait ModelRegistry: Send + Sync {
    /// Get the local paths for a model by its ID.
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error>;
}

/// Paths to local model assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPaths {
    pub weights: Vec<PathBuf>,
    pub tokenizer: PathBuf,
    pub config: PathBuf,
    pub generation_config: Option<PathBuf>,
}
