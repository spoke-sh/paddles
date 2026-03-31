use paddles_conversation::{
    ArtifactEnvelope, ConversationThreadRef, TaskTraceId, ThreadCandidate, ThreadDecision,
    ThreadMergeRecord, TraceArtifactId, TraceBranchId, TraceCheckpointId, TraceRecordId,
    TurnTraceId,
};
use serde::{Deserialize, Serialize};

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
    PlannerAction { action: String, rationale: String },
    PlannerBranchDeclared(TraceBranch),
    SelectionArtifact(TraceSelectionArtifact),
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
        TaskTraceId, TraceBranch, TraceBranchId, TraceBranchStatus, TraceLineage, TraceRecordId,
        TurnTraceId,
    };
    use paddles_conversation::{ArtifactEnvelope, ArtifactKind, TraceArtifactId};

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
}
