use super::{
    ConversationDelegationProjection, ConversationForensicProjection, ConversationForensicUpdate,
    ConversationManifoldProjection, ConversationTranscript, ConversationTranscriptUpdate,
    TaskTraceId, TraceRecordKind, TraceReplay,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTraceGraph {
    pub task_id: TaskTraceId,
    pub nodes: Vec<ConversationTraceGraphNode>,
    pub edges: Vec<ConversationTraceGraphEdge>,
    pub branches: Vec<ConversationTraceGraphBranch>,
}

impl ConversationTraceGraph {
    pub fn empty(task_id: TaskTraceId) -> Self {
        Self {
            task_id,
            nodes: Vec::new(),
            edges: Vec::new(),
            branches: Vec::new(),
        }
    }

    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        Self::from_trace_replays(std::slice::from_ref(replay))
    }

    pub fn from_trace_replays(replays: &[TraceReplay]) -> Self {
        let task_id = replays
            .first()
            .map(|replay| replay.task_id.clone())
            .unwrap_or_else(|| TaskTraceId::new("task-000000").expect("empty task id"));
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut branches = Vec::new();

        for replay in replays {
            for record in &replay.records {
                let (kind, label) = match &record.kind {
                    TraceRecordKind::TaskRootStarted(root) => {
                        ("root".to_string(), root.planner_model.clone())
                    }
                    TraceRecordKind::TurnStarted(_) => ("turn".to_string(), "turn".to_string()),
                    TraceRecordKind::ExecutionGovernanceProfileDeclared(snapshot) => (
                        "governance".to_string(),
                        truncate(&snapshot.profile_selection(), 24),
                    ),
                    TraceRecordKind::ExecutionGovernanceDecisionRecorded(decision) => {
                        ("governance".to_string(), truncate(&decision.summary(), 24))
                    }
                    TraceRecordKind::PlannerAction { action, .. } => {
                        ("action".to_string(), truncate(action, 24))
                    }
                    TraceRecordKind::PlannerBranchDeclared(branch) => {
                        branches.push(ConversationTraceGraphBranch {
                            id: branch.branch_id.as_str().to_string(),
                            label: branch.label.clone(),
                            status: branch.status.label().to_string(),
                            parent_branch_id: branch
                                .parent_branch_id
                                .as_ref()
                                .map(|id| id.as_str().to_string()),
                        });
                        ("branch".to_string(), truncate(&branch.label, 24))
                    }
                    TraceRecordKind::ToolCallRequested(tool) => {
                        ("tool".to_string(), trace_tool_graph_label(tool, false))
                    }
                    TraceRecordKind::ToolCallCompleted(tool) => {
                        ("tool_done".to_string(), trace_tool_graph_label(tool, true))
                    }
                    TraceRecordKind::SelectionArtifact(sel) => {
                        ("evidence".to_string(), truncate(&sel.summary, 24))
                    }
                    TraceRecordKind::ModelExchangeArtifact(artifact) => (
                        "forensic".to_string(),
                        truncate(
                            &format!("{} {}", artifact.category.label(), artifact.phase.label()),
                            24,
                        ),
                    ),
                    TraceRecordKind::LineageEdge(edge) => {
                        ("lineage".to_string(), truncate(&edge.summary, 24))
                    }
                    TraceRecordKind::SignalSnapshot(signal) => (
                        "signal".to_string(),
                        truncate(&format!("{} {}", signal.kind.label(), signal.level), 24),
                    ),
                    TraceRecordKind::CompletionCheckpoint(cp) => {
                        ("checkpoint".to_string(), cp.kind.label().to_string())
                    }
                    TraceRecordKind::ControlResultRecorded(result) => {
                        ("control".to_string(), result.summary())
                    }
                    TraceRecordKind::WorkerLifecycleRecorded(lifecycle) => (
                        "worker".to_string(),
                        truncate(&lifecycle.result.summary(), 24),
                    ),
                    TraceRecordKind::WorkerArtifactRecorded(artifact) => (
                        "worker_artifact".to_string(),
                        truncate(
                            &format!("{} {}", artifact.record.kind.label(), artifact.record.label),
                            24,
                        ),
                    ),
                    TraceRecordKind::WorkerIntegrationRecorded(integration) => (
                        "worker".to_string(),
                        truncate(&format!("integrate {}", integration.status.label()), 24),
                    ),
                    TraceRecordKind::CollaborationModeDeclared(result) => (
                        "mode".to_string(),
                        truncate(
                            &format!("{} {}", result.active.mode.label(), result.status.label()),
                            24,
                        ),
                    ),
                    TraceRecordKind::StructuredClarificationRecorded(result) => (
                        "clarification".to_string(),
                        truncate(
                            &format!("{} {}", result.request.kind.label(), result.status.label()),
                            24,
                        ),
                    ),
                    TraceRecordKind::ThreadMerged(_) => ("merge".to_string(), "merge".to_string()),
                    TraceRecordKind::ThreadCandidateCaptured(_) => {
                        ("thread".to_string(), "candidate".to_string())
                    }
                    TraceRecordKind::ThreadDecisionSelected(_) => {
                        ("thread".to_string(), "decision".to_string())
                    }
                };

                nodes.push(ConversationTraceGraphNode {
                    id: record.record_id.as_str().to_string(),
                    kind,
                    label,
                    branch_id: record
                        .lineage
                        .branch_id
                        .as_ref()
                        .map(|id| id.as_str().to_string()),
                    sequence: record.sequence,
                });

                if let Some(parent_id) = &record.lineage.parent_record_id {
                    edges.push(ConversationTraceGraphEdge {
                        from: parent_id.as_str().to_string(),
                        to: record.record_id.as_str().to_string(),
                    });
                }
            }
        }

        Self {
            task_id,
            nodes,
            edges,
            branches,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTraceGraphNode {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub branch_id: Option<String>,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTraceGraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTraceGraphBranch {
    pub id: String,
    pub label: String,
    pub status: String,
    pub parent_branch_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationProjectionSnapshot {
    pub task_id: TaskTraceId,
    pub transcript: ConversationTranscript,
    pub forensics: ConversationForensicProjection,
    pub manifold: ConversationManifoldProjection,
    pub trace_graph: ConversationTraceGraph,
    pub delegation: ConversationDelegationProjection,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationProjectionUpdateKind {
    Transcript,
    Forensic,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationProjectionUpdate {
    pub task_id: TaskTraceId,
    pub kind: ConversationProjectionUpdateKind,
    pub transcript_update: Option<ConversationTranscriptUpdate>,
    pub forensic_update: Option<ConversationForensicUpdate>,
    pub snapshot: ConversationProjectionSnapshot,
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}...", &s[..n])
    } else {
        s.to_string()
    }
}

fn trace_tool_graph_label(tool: &crate::domain::model::TraceToolCall, completed: bool) -> String {
    if tool.tool_name != "external_capability" {
        return tool.tool_name.clone();
    }

    let payload = tool.payload.inline_content.as_deref().unwrap_or_default();
    let first_line = payload.lines().next().unwrap_or_default();
    let fabric = first_line
        .split_whitespace()
        .find_map(|segment| segment.strip_prefix("fabric="))
        .unwrap_or("external_capability");
    let status = first_line
        .split_whitespace()
        .find_map(|segment| segment.strip_prefix("status="))
        .unwrap_or(if completed { "completed" } else { "requested" });
    let availability = first_line
        .split_whitespace()
        .find_map(|segment| segment.strip_prefix("availability="));

    let mut label = format!("{fabric} {status}");
    if completed && let Some(availability) = availability {
        label.push_str(&format!(" ({availability})"));
    }
    label
}

#[cfg(test)]
mod tests {
    use super::ConversationTraceGraph;
    use crate::domain::model::{
        TaskTraceId, TraceBranch, TraceBranchId, TraceBranchStatus, TraceLineage, TraceRecord,
        TraceRecordId, TraceRecordKind, TraceReplay, TraceSignalSnapshot, TurnTraceId,
    };
    use paddles_conversation::{ArtifactEnvelope, ArtifactKind, TraceArtifactId};

    #[test]
    fn trace_graph_projection_replays_root_actions_signals_and_branches() {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let turn_id = TurnTraceId::new("task-1.turn-0001").expect("turn");
        let root_id = TraceRecordId::new("task-1.turn-0001.record-0001").expect("root");
        let action_id = TraceRecordId::new("task-1.turn-0001.record-0002").expect("action");
        let signal_id = TraceRecordId::new("task-1.turn-0001.record-0003").expect("signal");
        let branch_id = TraceBranchId::new("branch-1").expect("branch");
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: root_id.clone(),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(crate::domain::model::TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "hello",
                            64,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                        harness_profile: crate::domain::model::TraceHarnessProfileSelection {
                            requested_profile_id: "recursive-structured-v1".to_string(),
                            active_profile_id: "recursive-structured-v1".to_string(),
                            downgrade_reason: None,
                        },
                    }),
                },
                TraceRecord {
                    record_id: action_id.clone(),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: Some(branch_id.clone()),
                        parent_record_id: Some(root_id),
                    },
                    kind: TraceRecordKind::PlannerAction {
                        action: "inspect `git status --short`".to_string(),
                        rationale: "inspect first".to_string(),
                    },
                },
                TraceRecord {
                    record_id: signal_id,
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: Some(branch_id.clone()),
                        parent_record_id: Some(action_id),
                    },
                    kind: TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                        kind: crate::domain::model::TraceSignalKind::ActionBias,
                        gate: None,
                        phase: None,
                        summary: "bias".to_string(),
                        level: "high".to_string(),
                        magnitude_percent: 78,
                        applies_to: None,
                        contributions: Vec::new(),
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::Prompt,
                            "bias",
                            "{}",
                            usize::MAX,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("task-1.turn-0001.record-0004")
                        .expect("branch record"),
                    sequence: 4,
                    lineage: TraceLineage {
                        task_id: TaskTraceId::new("task-1").expect("task"),
                        turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                        branch_id: Some(branch_id.clone()),
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::PlannerBranchDeclared(TraceBranch {
                        branch_id,
                        parent_branch_id: None,
                        label: "main investigation branch".to_string(),
                        status: TraceBranchStatus::Active,
                        rationale: Some("branch".to_string()),
                        created_from_record_id: None,
                    }),
                },
            ],
        };

        let graph = ConversationTraceGraph::from_trace_replay(&replay);
        assert_eq!(graph.task_id.as_str(), "task-1");
        assert!(graph.nodes.iter().any(|node| node.kind == "root"));
        assert!(graph.nodes.iter().any(|node| node.kind == "action"));
        assert!(graph.nodes.iter().any(|node| node.kind == "signal"));
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.branches.len(), 1);
    }
}
