mod context_gathering;
mod planning;

use async_trait::async_trait;
use std::path::PathBuf;

pub use context_gathering::{
    ContextGatherRequest, ContextGatherResult, ContextGatherer, EvidenceBudget, EvidenceBundle,
    EvidenceItem, GathererCapability, PlannerConfig, PlannerDecision, PlannerStrategyKind,
    PlannerTraceMetadata, PlannerTraceStep, RetainedEvidence,
};
pub use planning::{
    InitialAction, InitialActionDecision, InterpretationContext, InterpretationDocument,
    PlannerAction, PlannerBudget, PlannerCapability, PlannerDecision as RecursivePlannerDecision,
    PlannerLoopState, PlannerRequest, PlannerStepRecord, RecursivePlanner,
};

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
