use super::render::RenderDocument;
use super::traces::{TraceRecordKind, TraceReplay};
use paddles_conversation::{ArtifactEnvelope, TaskTraceId, TraceRecordId, TurnTraceId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversationTranscriptSpeaker {
    User,
    Assistant,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationTranscriptEntry {
    pub record_id: TraceRecordId,
    pub turn_id: TurnTraceId,
    pub speaker: ConversationTranscriptSpeaker,
    pub content: String,
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
                        render: None,
                    })
                }
                TraceRecordKind::TurnStarted(turn) => entries.push(ConversationTranscriptEntry {
                    record_id: record.record_id.clone(),
                    turn_id: record.lineage.turn_id.clone(),
                    speaker: ConversationTranscriptSpeaker::User,
                    content: artifact_content(&turn.prompt),
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
                            render: Some(render),
                        });
                    }
                }
                _ => {}
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
        TraceCheckpointKind, TraceCompletionCheckpoint, TraceLineage, TraceRecord, TraceRecordKind,
        TraceReplay, TraceTaskRoot,
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
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "assistant response",
                            "hi",
                            200,
                        )),
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
        assert!(transcript.entries[1].render.is_some());
    }
}
