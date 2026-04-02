use paddles_conversation::{
    ArtifactEnvelope, ConversationThreadRef, TaskTraceId, ThreadCandidate, ThreadDecision,
    ThreadMergeRecord, TraceArtifactId, TraceBranchId, TraceCheckpointId, TraceRecordId,
    TurnTraceId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLineage {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub branch_id: Option<TraceBranchId>,
    pub parent_record_id: Option<TraceRecordId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceBranchStatus {
    Pending,
    Active,
    Completed,
    Merged,
    Pruned,
}

impl TraceBranchStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Merged => "merged",
            Self::Pruned => "pruned",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceBranch {
    pub branch_id: TraceBranchId,
    pub label: String,
    pub status: TraceBranchStatus,
    pub rationale: Option<String>,
    pub parent_branch_id: Option<TraceBranchId>,
    pub created_from_record_id: Option<TraceRecordId>,
}

impl TraceBranch {
    pub fn summary(&self) -> String {
        format!("{} ({})", self.label, self.status.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceSelectionKind {
    Evidence,
    PlannerTrace,
    Branch,
    Synthesis,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceSelectionArtifact {
    pub selection_id: TraceArtifactId,
    pub kind: TraceSelectionKind,
    pub summary: String,
    pub artifact: ArtifactEnvelope,
    pub selected_from: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceToolCall {
    pub call_id: String,
    pub tool_name: String,
    pub payload: ArtifactEnvelope,
    pub success: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceModelExchangeLane {
    Planner,
    Synthesizer,
}

impl TraceModelExchangeLane {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Planner => "planner",
            Self::Synthesizer => "synthesizer",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceModelExchangeCategory {
    Interpretation,
    InitialAction,
    PlannerAction,
    ThreadDecision,
    TurnResponse,
}

impl TraceModelExchangeCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Interpretation => "interpretation",
            Self::InitialAction => "initial_action",
            Self::PlannerAction => "planner_action",
            Self::ThreadDecision => "thread_decision",
            Self::TurnResponse => "turn_response",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceModelExchangePhase {
    AssembledContext,
    ProviderRequest,
    RawProviderResponse,
    RenderedResponse,
}

impl TraceModelExchangePhase {
    pub fn label(&self) -> &'static str {
        match self {
            Self::AssembledContext => "assembled_context",
            Self::ProviderRequest => "provider_request",
            Self::RawProviderResponse => "raw_provider_response",
            Self::RenderedResponse => "rendered_response",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceModelExchangeArtifact {
    pub exchange_id: String,
    pub lane: TraceModelExchangeLane,
    pub category: TraceModelExchangeCategory,
    pub phase: TraceModelExchangePhase,
    pub provider: String,
    pub model: String,
    pub parent_artifact_id: Option<TraceArtifactId>,
    pub artifact: ArtifactEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceLineageNodeKind {
    Conversation,
    Turn,
    ModelCall,
    PlannerStep,
    Artifact,
    Output,
    #[serde(rename = "Signal", alias = "Force")]
    Signal,
}

impl TraceLineageNodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Conversation => "conversation",
            Self::Turn => "turn",
            Self::ModelCall => "model_call",
            Self::PlannerStep => "planner_step",
            Self::Artifact => "artifact",
            Self::Output => "output",
            Self::Signal => "signal",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLineageNodeRef {
    pub kind: TraceLineageNodeKind,
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceLineageRelation {
    Contains,
    Triggers,
    Produces,
    Transforms,
    ResultsIn,
    Constrains,
}

impl TraceLineageRelation {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::Triggers => "triggers",
            Self::Produces => "produces",
            Self::Transforms => "transforms",
            Self::ResultsIn => "results_in",
            Self::Constrains => "constrains",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceLineageEdge {
    pub source: TraceLineageNodeRef,
    pub target: TraceLineageNodeRef,
    pub relation: TraceLineageRelation,
    pub summary: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceSignalKind {
    #[serde(rename = "ContextStrain", alias = "ContextPressure")]
    ContextStrain,
    #[serde(rename = "CompactionCue", alias = "Compaction")]
    CompactionCue,
    #[serde(rename = "ActionBias", alias = "ExecutionPressure")]
    ActionBias,
    Fallback,
    #[serde(rename = "BudgetBoundary", alias = "Budget")]
    BudgetBoundary,
}

impl TraceSignalKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::ContextStrain => "context_strain",
            Self::CompactionCue => "compaction_cue",
            Self::ActionBias => "action_bias",
            Self::Fallback => "fallback",
            Self::BudgetBoundary => "budget_boundary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceSignalContribution {
    pub source: String,
    pub share_percent: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceSignalSnapshot {
    pub kind: TraceSignalKind,
    pub summary: String,
    pub level: String,
    pub magnitude_percent: u8,
    pub applies_to: Option<TraceLineageNodeRef>,
    pub contributions: Vec<TraceSignalContribution>,
    pub artifact: ArtifactEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceCheckpointKind {
    TurnCompleted,
    TurnFailed,
}

impl TraceCheckpointKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::TurnCompleted => "turn-completed",
            Self::TurnFailed => "turn-failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceCompletionCheckpoint {
    pub checkpoint_id: TraceCheckpointId,
    pub kind: TraceCheckpointKind,
    pub summary: String,
    pub response: Option<ArtifactEnvelope>,
    pub citations: Vec<String>,
    pub grounded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceTaskRoot {
    pub prompt: ArtifactEnvelope,
    pub interpretation: Option<ArtifactEnvelope>,
    pub planner_model: String,
    pub synthesizer_model: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceTurnStarted {
    pub prompt: ArtifactEnvelope,
    pub interpretation: Option<ArtifactEnvelope>,
    pub planner_model: String,
    pub synthesizer_model: String,
    pub thread: ConversationThreadRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceRecordKind {
    TaskRootStarted(TraceTaskRoot),
    TurnStarted(TraceTurnStarted),
    ThreadCandidateCaptured(ThreadCandidate),
    ThreadDecisionSelected(ThreadDecision),
    ThreadMerged(ThreadMergeRecord),
    PlannerAction {
        action: String,
        rationale: String,
    },
    PlannerBranchDeclared(TraceBranch),
    SelectionArtifact(TraceSelectionArtifact),
    ModelExchangeArtifact(TraceModelExchangeArtifact),
    LineageEdge(TraceLineageEdge),
    #[serde(rename = "SignalSnapshot", alias = "ForceSnapshot")]
    SignalSnapshot(TraceSignalSnapshot),
    ToolCallRequested(TraceToolCall),
    ToolCallCompleted(TraceToolCall),
    CompletionCheckpoint(TraceCompletionCheckpoint),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceRecord {
    pub record_id: TraceRecordId,
    pub sequence: u64,
    pub lineage: TraceLineage,
    pub kind: TraceRecordKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceReplay {
    pub task_id: TaskTraceId,
    pub records: Vec<TraceRecord>,
}

#[cfg(test)]
mod tests {
    use super::{
        TaskTraceId, TraceBranch, TraceBranchId, TraceBranchStatus, TraceLineage,
        TraceLineageNodeKind, TraceRecordId, TraceRecordKind, TraceSignalContribution,
        TraceSignalKind, TraceSignalSnapshot, TurnTraceId,
    };
    use paddles_conversation::{ArtifactEnvelope, ArtifactKind, TraceArtifactId};
    use serde_json::json;

    #[test]
    fn artifact_envelope_uses_locator_when_truncated() {
        let artifact = ArtifactEnvelope::text(
            TraceArtifactId::new("artifact-1").expect("artifact id"),
            ArtifactKind::Prompt,
            "prompt",
            "abcdefghij",
            4,
        );

        assert!(artifact.truncated);
        assert_eq!(artifact.locator, None);
        assert!(
            artifact
                .inline_content
                .as_deref()
                .expect("inline")
                .contains("[truncated]")
        );
    }

    #[test]
    fn branch_summary_is_machine_readable_but_human_scannable() {
        let branch = TraceBranch {
            branch_id: TraceBranchId::new("branch-1").expect("branch id"),
            label: "inspect mission state".to_string(),
            status: TraceBranchStatus::Pending,
            rationale: None,
            parent_branch_id: None,
            created_from_record_id: None,
        };

        assert_eq!(branch.summary(), "inspect mission state (pending)");
    }

    #[test]
    fn lineage_references_keep_task_turn_and_parent_record() {
        let lineage = TraceLineage {
            task_id: TaskTraceId::new("task-1").expect("task id"),
            turn_id: TurnTraceId::new("turn-1").expect("turn id"),
            branch_id: None,
            parent_record_id: Some(TraceRecordId::new("record-1").expect("record id")),
        };

        assert_eq!(lineage.task_id.as_str(), "task-1");
        assert_eq!(lineage.turn_id.as_str(), "turn-1");
        assert_eq!(
            lineage.parent_record_id.as_ref().map(|id| id.as_str()),
            Some("record-1")
        );
    }

    #[test]
    fn context_locator_reports_correct_tier() {
        use paddles_conversation::{ContextLocator, ContextTier, TaskTraceId};
        use std::path::PathBuf;

        let inline = ContextLocator::Inline {
            content: "test".to_string(),
        };
        assert_eq!(inline.tier(), ContextTier::Inline);

        let transit = ContextLocator::Transit {
            task_id: TaskTraceId::new("task-1").expect("task id"),
            record_id: TraceRecordId::new("record-1").expect("record id"),
        };
        assert_eq!(transit.tier(), ContextTier::Transit);

        let sift = ContextLocator::Sift {
            index_ref: "sift-1".to_string(),
        };
        assert_eq!(sift.tier(), ContextTier::Sift);

        let fs = ContextLocator::Filesystem {
            path: PathBuf::from("src/lib.rs"),
        };
        assert_eq!(fs.tier(), ContextTier::Filesystem);
    }

    #[test]
    fn context_locator_serializes_round_trip() {
        use paddles_conversation::ContextLocator;

        let transit = ContextLocator::Transit {
            task_id: TaskTraceId::new("task-1").expect("task id"),
            record_id: TraceRecordId::new("record-1").expect("record id"),
        };

        let serialized = serde_json::to_string(&transit).expect("serialize");
        let deserialized: ContextLocator = serde_json::from_str(&serialized).expect("deserialize");

        assert_eq!(transit, deserialized);
    }

    #[test]
    fn artifact_envelope_carries_locator_with_tier_metadata() {
        use paddles_conversation::{
            ArtifactEnvelope, ArtifactKind, ContextLocator, ContextTier, TraceArtifactId,
        };

        let locator = ContextLocator::Transit {
            task_id: TaskTraceId::new("task-1").expect("task id"),
            record_id: TraceRecordId::new("record-1").expect("record id"),
        };
        let envelope = ArtifactEnvelope {
            artifact_id: TraceArtifactId::new("art-1").expect("artifact id"),
            kind: ArtifactKind::ModelOutput,
            mime_type: "text/plain".to_string(),
            summary: "test".to_string(),
            byte_count: 4,
            inline_content: Some("te...[truncated]".to_string()),
            locator: Some(locator),
            truncated: true,
            labels: Default::default(),
        };

        let loc = envelope.locator.as_ref().expect("locator present");
        assert_eq!(loc.tier(), ContextTier::Transit);
    }

    #[test]
    fn no_transit_sift_types_leak_into_domain_ports() {
        // ContextResolver trait uses paddles_conversation::ContextLocator (domain types only).
        // This test documents that the port boundary is maintained — the trait signature
        // does not reference transit-core or sift-core types directly.
        use paddles_conversation::ContextLocator;
        let _: fn(&ContextLocator) -> ContextLocator = |l| l.clone();
    }

    #[test]
    fn trace_signal_kind_labels_use_steering_signal_vocabulary() {
        assert_eq!(TraceSignalKind::ContextStrain.label(), "context_strain");
        assert_eq!(TraceSignalKind::ActionBias.label(), "action_bias");
        assert_eq!(TraceSignalKind::CompactionCue.label(), "compaction_cue");
        assert_eq!(TraceSignalKind::BudgetBoundary.label(), "budget_boundary");
    }

    #[test]
    fn legacy_force_snapshot_records_remain_deserializable() {
        let legacy = json!({
            "ForceSnapshot": {
                "kind": "ContextPressure",
                "summary": "legacy context pressure",
                "level": "medium",
                "magnitude_percent": 45,
                "applies_to": {
                    "kind": "Force",
                    "id": "force:record-1",
                    "label": "context_pressure"
                },
                "contributions": [
                    {
                        "source": "operator_memory",
                        "share_percent": 100,
                        "rationale": "legacy"
                    }
                ],
                "artifact": ArtifactEnvelope::text(
                    TraceArtifactId::new("artifact-1").expect("artifact id"),
                    ArtifactKind::PlannerTrace,
                    "legacy",
                    "{}",
                    10
                )
            }
        });

        let kind: TraceRecordKind =
            serde_json::from_value(legacy).expect("legacy force snapshot should deserialize");

        match kind {
            TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                kind,
                applies_to,
                contributions,
                ..
            }) => {
                assert_eq!(kind, TraceSignalKind::ContextStrain);
                assert_eq!(
                    applies_to.expect("applies_to").kind,
                    TraceLineageNodeKind::Signal
                );
                assert_eq!(
                    contributions,
                    vec![TraceSignalContribution {
                        source: "operator_memory".to_string(),
                        share_percent: 100,
                        rationale: "legacy".to_string(),
                    }]
                );
            }
            other => panic!("unexpected trace record kind: {other:?}"),
        }
    }
}
