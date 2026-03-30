mod context_gathering;
mod operator_memory;
mod planning;
mod synthesis;
mod trace_recording;

use async_trait::async_trait;
use std::path::PathBuf;

pub use context_gathering::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBudget, EvidenceBundle,
    EvidenceItem, GathererCapability, PlannerConfig, PlannerDecision, PlannerGraphBranch,
    PlannerGraphBranchStatus, PlannerGraphEdge, PlannerGraphEdgeKind, PlannerGraphEpisode,
    PlannerGraphFrontierEntry, PlannerGraphNode, PlannerStrategyKind, PlannerTraceMetadata,
    PlannerTraceStep, RetainedEvidence, RetrievalMode, RetrievalStrategy,
};
pub use operator_memory::OperatorMemory;
pub use planning::{
    GuidanceCategory, InitialAction, InitialActionDecision, InterpretationConflict,
    InterpretationContext, InterpretationCoverageConfidence, InterpretationDecisionFramework,
    InterpretationDocument, InterpretationProcedure, InterpretationProcedureStep,
    InterpretationRequest, InterpretationToolHint, OperatorMemoryDocument, PlannerAction,
    PlannerBudget, PlannerCapability, PlannerDecision as RecursivePlannerDecision,
    PlannerLoopState, PlannerRequest, PlannerStepRecord, RecursivePlanner, ThreadDecisionRequest,
    WorkspaceAction,
};
pub use synthesis::{SynthesizerEngine, WorkspaceActionResult};
pub use trace_recording::{NoopTraceRecorder, TraceRecorder, TraceRecorderCapability};

/// Port for model discovery and acquisition.
#[async_trait]
pub trait ModelRegistry: Send + Sync {
    /// Get the local paths for a model by its ID.
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error>;
}

/// Paths to local model assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPaths {
    pub weights: PathBuf,
    pub tokenizer: PathBuf,
    pub config: PathBuf,
}
