mod context_gathering;
mod context_resolution;
mod entity_resolution;
mod execution_hands;
mod external_capabilities;
mod model_registry;
mod operator_memory;
mod planning;
mod semantic_workspace;
mod session_store;
mod specialist_brains;
mod synthesis;
mod trace_recording;
mod workspace_action_execution;
mod workspace_editing;

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
pub use model_registry::{
    ModelPaths, ModelRegistry, ProviderModelPostureEntry, ProviderModelPostureStatus,
    ProviderRegistryPosture, ProviderRegistryPostureRequest,
};
pub use operator_memory::OperatorMemory;
pub use planning::{
    CompactionPlan, CompactionRequest, GroundingDomain, GroundingRequirement, GuidanceCategory,
    InitialAction, InitialActionDecision, InitialEditInstruction, InterpretationConflict,
    InterpretationContext, InterpretationCoverageConfidence, InterpretationDecisionFramework,
    InterpretationDocument, InterpretationProcedure, InterpretationProcedureStep,
    InterpretationRequest, InterpretationToolHint, OperatorMemoryDocument, PlannerAction,
    PlannerBudget, PlannerCapability, PlannerDecision as RecursivePlannerDecision,
    PlannerExecutionContract, PlannerLoopState, PlannerRequest, PlannerStepRecord,
    RecursivePlanner, RefinementPolicy, RefinementTrigger, RefinementTriggerSource,
    ThreadDecisionRequest, WorkspaceAction,
};
pub use semantic_workspace::{
    SemanticWorkspaceOperation, SemanticWorkspacePort, SemanticWorkspaceQuery,
    SemanticWorkspaceResult, SemanticWorkspaceStatus,
};
pub use session_store::{
    SESSION_STORE_SCHEMA, SESSION_STORE_SCHEMA_VERSION, SessionCompactionLineage,
    SessionCompactionRecord, SessionEvidenceRecord, SessionGovernanceRecord,
    SessionModelVisibleContextEntry, SessionPlannerDecisionRecord, SessionReplayRecord,
    SessionRollbackAnchor, SessionSnapshotRecord, SessionSnapshotReplayValidation,
    SessionSnapshotStatus, SessionStorePort, SessionStoreRecord, SessionStoreRecordKind,
    SessionStoreSnapshot, SessionTurnRecord, VersionedSessionStoreRecord,
};
pub use specialist_brains::{
    SpecialistBrain, SpecialistBrainCapability, SpecialistBrainNote, SpecialistBrainRequest,
};
pub use synthesis::{SynthesisHandoff, SynthesizerEngine};
pub use trace_recording::{
    NoopTraceRecorder, TraceRecorder, TraceRecorderCapability, TraceReplaySlice,
    TraceReplaySliceAnchor, TraceReplaySliceDirection, TraceReplaySliceRequest,
    TraceSessionCheckpointCursor, TraceSessionContextQuery, TraceSessionContextSlice,
    TraceSessionHostedCursor, TraceSessionHostedMaterialization, TraceSessionWake,
};
pub use workspace_action_execution::{
    WorkspaceActionCapability, WorkspaceActionExecutionFrame, WorkspaceActionExecutor,
    WorkspaceActionResult, WorkspaceCapabilitySurface, WorkspaceToolCapability,
};
pub use workspace_editing::WorkspaceEditor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_registry_posture_reports_configured_discovered_unavailable_and_deprecated_entries()
    {
        let posture = ProviderRegistryPosture::local_first(vec![
            ProviderModelPostureEntry::configured("sift", "qwen-1.5b"),
            ProviderModelPostureEntry::discovered("ollama", "qwen3:8b"),
            ProviderModelPostureEntry::unavailable("openai", "gpt-5.4", "missing API key"),
            ProviderModelPostureEntry::deprecated(
                "openai",
                "gpt-4o",
                "kept for compatibility; prefer gpt-5.4",
            ),
        ]);

        assert!(posture.has_status(ProviderModelPostureStatus::Configured));
        assert!(posture.has_status(ProviderModelPostureStatus::Discovered));
        assert!(posture.has_status(ProviderModelPostureStatus::Unavailable));
        assert!(posture.has_status(ProviderModelPostureStatus::Deprecated));
        assert_eq!(
            posture.entries_by_status(ProviderModelPostureStatus::Unavailable)[0].reason,
            Some("missing API key".to_string())
        );
    }

    #[test]
    fn provider_registry_offline_builds_local_first_posture_without_network_discovery() {
        let request = ProviderRegistryPostureRequest::local_first();
        let posture = ProviderRegistryPosture::from_configured_models(
            request,
            vec![("sift", "qwen-1.5b"), ("openai", "gpt-5.4")],
        );

        assert!(!posture.network_discovery_required);
        assert!(posture.is_offline_safe());
        assert_eq!(posture.entries.len(), 2);
        assert!(
            posture
                .entries
                .iter()
                .all(|entry| entry.status == ProviderModelPostureStatus::Configured)
        );
    }
}
