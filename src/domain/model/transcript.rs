use super::execution_hand::{ExecutionGovernanceDecision, ExecutionGovernanceSnapshot};
use super::generative::ResponseMode;
use super::render::RenderDocument;
use super::traces::{TraceRecordKind, TraceReplay};
use super::{ControlResult, ControlSubject, trace_control_result};
use paddles_conversation::{ArtifactEnvelope, TaskTraceId, TraceRecordId, TurnTraceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTranscriptSpeaker {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTranscriptEntry {
    pub record_id: TraceRecordId,
    pub turn_id: TurnTraceId,
    pub speaker: ConversationTranscriptSpeaker,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mode: Option<ResponseMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render: Option<RenderDocument>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTranscript {
    pub task_id: TaskTraceId,
    pub entries: Vec<ConversationTranscriptEntry>,
}

impl ConversationTranscript {
    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        let mut entries = Vec::new();
        for record in &replay.records {
            match &record.kind {
                TraceRecordKind::TaskRootStarted(root) => {
                    entries.push(ConversationTranscriptEntry {
                        record_id: record.record_id.clone(),
                        turn_id: record.lineage.turn_id.clone(),
                        speaker: ConversationTranscriptSpeaker::User,
                        content: artifact_content(&root.prompt),
                        response_mode: None,
                        render: None,
                    })
                }
                TraceRecordKind::TurnStarted(turn) => entries.push(ConversationTranscriptEntry {
                    record_id: record.record_id.clone(),
                    turn_id: record.lineage.turn_id.clone(),
                    speaker: ConversationTranscriptSpeaker::User,
                    content: artifact_content(&turn.prompt),
                    response_mode: None,
                    render: None,
                }),
                TraceRecordKind::CompletionCheckpoint(checkpoint) => {
                    if let Some(response) = checkpoint.response.as_ref() {
                        let render =
                            RenderDocument::from_assistant_plain_text(&artifact_content(response));
                        entries.push(ConversationTranscriptEntry {
                            record_id: record.record_id.clone(),
                            turn_id: record.lineage.turn_id.clone(),
                            speaker: ConversationTranscriptSpeaker::Assistant,
                            content: render.to_plain_text(),
                            response_mode: response_mode_label(response),
                            render: Some(render),
                        });
                    }
                }
                TraceRecordKind::ExecutionGovernanceProfileDeclared(snapshot) => {
                    entries.push(ConversationTranscriptEntry {
                        record_id: record.record_id.clone(),
                        turn_id: record.lineage.turn_id.clone(),
                        speaker: ConversationTranscriptSpeaker::System,
                        content: format_execution_governance_snapshot(snapshot),
                        response_mode: None,
                        render: None,
                    });
                }
                TraceRecordKind::ExecutionGovernanceDecisionRecorded(decision) => {
                    entries.push(ConversationTranscriptEntry {
                        record_id: record.record_id.clone(),
                        turn_id: record.lineage.turn_id.clone(),
                        speaker: ConversationTranscriptSpeaker::System,
                        content: format_execution_governance_decision(decision),
                        response_mode: None,
                        render: None,
                    });
                }
                kind => {
                    if let Some(result) = trace_control_result(kind) {
                        entries.push(ConversationTranscriptEntry {
                            record_id: record.record_id.clone(),
                            turn_id: record.lineage.turn_id.clone(),
                            speaker: ConversationTranscriptSpeaker::System,
                            content: format_control_result(&result),
                            response_mode: None,
                            render: None,
                        });
                    }
                }
            }
        }

        Self {
            task_id: replay.task_id.clone(),
            entries,
        }
    }
}

fn artifact_content(artifact: &ArtifactEnvelope) -> String {
    artifact
        .inline_content
        .clone()
        .unwrap_or_else(|| artifact.summary.clone())
}

fn response_mode_label(artifact: &ArtifactEnvelope) -> Option<ResponseMode> {
    artifact
        .labels
        .get("paddles.response_mode")
        .and_then(|value| ResponseMode::from_label(value))
}

fn format_execution_governance_snapshot(snapshot: &ExecutionGovernanceSnapshot) -> String {
    format!("{}\n{}", snapshot.summary(), snapshot.detail())
}

fn format_execution_governance_decision(decision: &ExecutionGovernanceDecision) -> String {
    format!("{}\n{}", decision.summary(), decision.detail())
}

fn format_control_result(result: &ControlResult) -> String {
    format!(
        "control: {}\nsubject={}\n{}",
        result.summary(),
        control_subject_label(&result.subject),
        result.detail
    )
}

fn control_subject_label(subject: &ControlSubject) -> String {
    subject
        .thread
        .as_ref()
        .map(|thread| thread.stable_id())
        .or_else(|| {
            subject
                .turn_id
                .as_ref()
                .map(|turn| turn.as_str().to_string())
        })
        .unwrap_or_else(|| "session".to_string())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTranscriptUpdate {
    pub task_id: TaskTraceId,
}

pub trait TranscriptUpdateSink: Send + Sync {
    fn emit(&self, update: ConversationTranscriptUpdate);
}

#[derive(Default)]
pub struct NullTranscriptUpdateSink;

impl TranscriptUpdateSink for NullTranscriptUpdateSink {
    fn emit(&self, _update: ConversationTranscriptUpdate) {}
}

#[cfg(test)]
mod tests {
    use super::{ConversationTranscript, ConversationTranscriptSpeaker};
    use crate::domain::model::{
        ControlOperation, ControlResult, ControlResultStatus, ControlSubject,
        ExecutionApprovalPolicy, ExecutionEscalationRequest, ExecutionGovernanceDecision,
        ExecutionGovernanceOutcome, ExecutionGovernanceProfile, ExecutionGovernanceSnapshot,
        ExecutionHandKind, ExecutionPermission, ExecutionPermissionRequest,
        ExecutionPermissionRequirement, ExecutionPermissionReuseScope, ExecutionSandboxMode,
        ResponseMode, TraceCheckpointKind, TraceCompletionCheckpoint, TraceLineage, TraceRecord,
        TraceRecordKind, TraceReplay, TraceTaskRoot, TurnControlOperation,
    };
    use paddles_conversation::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, TraceArtifactId, TraceCheckpointId,
        TraceRecordId, TurnTraceId,
    };

    #[test]
    fn projects_prompt_and_completion_entries_from_trace_replay() {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let turn_id = TurnTraceId::new("task-1.turn-0001").expect("turn");
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "hello",
                            200,
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
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-1").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "turn completed".to_string(),
                        response: Some(
                            ArtifactEnvelope::text(
                                TraceArtifactId::new("artifact-2").expect("artifact"),
                                ArtifactKind::ModelOutput,
                                "assistant response",
                                "hi",
                                200,
                            )
                            .with_label("paddles.response_mode", "grounded_answer"),
                        ),
                        citations: Vec::new(),
                        grounded: true,
                    }),
                },
            ],
        };

        let transcript = ConversationTranscript::from_trace_replay(&replay);
        assert_eq!(transcript.entries.len(), 2);
        assert_eq!(
            transcript.entries[0].speaker,
            ConversationTranscriptSpeaker::User
        );
        assert_eq!(transcript.entries[0].content, "hello");
        assert_eq!(
            transcript.entries[1].speaker,
            ConversationTranscriptSpeaker::Assistant
        );
        assert_eq!(transcript.entries[1].content, "hi");
        assert_eq!(
            transcript.entries[1].response_mode,
            Some(ResponseMode::GroundedAnswer)
        );
        assert!(transcript.entries[1].render.is_some());
    }

    #[test]
    fn projects_governance_records_into_system_transcript_entries() {
        let task_id = TaskTraceId::new("task-2").expect("task");
        let turn_id = TurnTraceId::new("task-2.turn-0001").expect("turn");
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "ship it",
                            200,
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
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-1").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::ExecutionGovernanceProfileDeclared(
                        ExecutionGovernanceSnapshot::new(
                            "recursive-structured-v1",
                            "prompt-envelope-safe-v1",
                            ExecutionGovernanceProfile::new(
                                ExecutionSandboxMode::WorkspaceWrite,
                                ExecutionApprovalPolicy::OnRequest,
                                vec![
                                    ExecutionPermissionReuseScope::Turn,
                                    ExecutionPermissionReuseScope::Hand,
                                ],
                                Some(
                                    "prompt-envelope-safe-v1 disables bounded command-prefix escalation reuse"
                                        .to_string(),
                                ),
                            ),
                        ),
                    ),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-3").expect("record"),
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-2").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::ExecutionGovernanceDecisionRecorded(
                        ExecutionGovernanceDecision::new(
                            Some("tool-1".to_string()),
                            Some("shell".to_string()),
                            ExecutionPermissionRequest::new(
                                ExecutionHandKind::TerminalRunner,
                                ExecutionPermissionRequirement::new(
                                    "run shell command",
                                    vec![ExecutionPermission::RunWorkspaceCommand],
                                ),
                            )
                            .with_bounded_reuse(
                                ExecutionPermissionReuseScope::CommandPrefix,
                                vec!["cargo".to_string(), "test".to_string()],
                            ),
                            ExecutionGovernanceOutcome::escalation_required(
                                "approval is required before reusing this command prefix",
                                ExecutionPermissionRequirement::new(
                                    "run shell command",
                                    vec![ExecutionPermission::RunWorkspaceCommand],
                                ),
                                ExecutionEscalationRequest::new(
                                    "allow cargo test",
                                    vec![ExecutionPermission::RunWorkspaceCommand],
                                    Some(ExecutionPermissionReuseScope::CommandPrefix),
                                    Some(vec!["cargo".to_string(), "test".to_string()]),
                                ),
                            ),
                        ),
                    ),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-4").expect("record"),
                    sequence: 4,
                    lineage: TraceLineage {
                        task_id,
                        turn_id,
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-3").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "turn completed".to_string(),
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "assistant response",
                            "done",
                            200,
                        )),
                        citations: Vec::new(),
                        grounded: false,
                    }),
                },
            ],
        };

        let transcript = ConversationTranscript::from_trace_replay(&replay);
        assert_eq!(transcript.entries.len(), 4);
        assert_eq!(
            transcript.entries[1].speaker,
            ConversationTranscriptSpeaker::System
        );
        assert!(transcript.entries[1].content.contains("execution posture"));
        assert!(transcript.entries[1].content.contains("downgrade="));
        assert_eq!(
            transcript.entries[2].speaker,
            ConversationTranscriptSpeaker::System
        );
        assert!(
            transcript.entries[2]
                .content
                .contains("escalation required shell")
        );
        assert!(
            transcript.entries[2]
                .content
                .contains("escalation_prefix=cargo test")
        );
    }

    #[test]
    fn projects_control_results_into_system_transcript_entries() {
        let task_id = TaskTraceId::new("task-3").expect("task");
        let turn_id = TurnTraceId::new("task-3.turn-0001").expect("turn");
        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "pause this run",
                            200,
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
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id,
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-1").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::ControlResultRecorded(ControlResult {
                        operation: ControlOperation::Turn(TurnControlOperation::Interrupt),
                        status: ControlResultStatus::Unavailable,
                        subject: ControlSubject {
                            turn_id: Some(turn_id.clone()),
                            thread: None,
                        },
                        detail: "planner lane is reconfiguring and cannot honor interrupt yet"
                            .to_string(),
                    }),
                },
            ],
        };

        let transcript = ConversationTranscript::from_trace_replay(&replay);
        assert_eq!(transcript.entries.len(), 2);
        assert_eq!(
            transcript.entries[1].speaker,
            ConversationTranscriptSpeaker::System
        );
        assert!(
            transcript.entries[1]
                .content
                .contains("interrupt unavailable")
        );
        assert!(
            transcript.entries[1]
                .content
                .contains("planner lane is reconfiguring and cannot honor interrupt yet")
        );
        assert!(transcript.entries[1].content.contains(turn_id.as_str()));
    }
}
