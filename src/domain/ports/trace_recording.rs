use crate::domain::model::{
    ConversationTranscript, ConversationTranscriptSpeaker, TaskTraceId, TraceBranchId,
    TraceCheckpointId, TraceCheckpointKind, TraceRecord, TraceRecordId, TraceRecordKind,
    TraceReplay, TurnTraceId,
};
use anyhow::{Result, anyhow};
use std::any::Any;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceRecorderCapability {
    Persistent { backend: String, location: String },
    Ephemeral { backend: String, reason: String },
    Unsupported { reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceSessionWake {
    pub task_id: TaskTraceId,
    pub latest_record_id: Option<TraceRecordId>,
    pub latest_sequence: Option<u64>,
    pub checkpoints: Vec<TraceSessionCheckpointCursor>,
}

impl TraceSessionWake {
    pub fn from_replay(replay: &TraceReplay) -> Self {
        let latest_record_id = replay.records.last().map(|record| record.record_id.clone());
        let latest_sequence = replay.records.last().map(|record| record.sequence);
        let checkpoints = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::CompletionCheckpoint(checkpoint) => {
                    Some(TraceSessionCheckpointCursor {
                        checkpoint_id: checkpoint.checkpoint_id.clone(),
                        record_id: record.record_id.clone(),
                        turn_id: record.lineage.turn_id.clone(),
                        sequence: record.sequence,
                        kind: checkpoint.kind,
                        summary: checkpoint.summary.clone(),
                        resume_request: TraceReplaySliceRequest::from_anchor(
                            TraceReplaySliceAnchor::Checkpoint(checkpoint.checkpoint_id.clone()),
                        ),
                    })
                }
                _ => None,
            })
            .collect();
        Self {
            task_id: replay.task_id.clone(),
            latest_record_id,
            latest_sequence,
            checkpoints,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceSessionCheckpointCursor {
    pub checkpoint_id: TraceCheckpointId,
    pub record_id: TraceRecordId,
    pub turn_id: TurnTraceId,
    pub sequence: u64,
    pub kind: TraceCheckpointKind,
    pub summary: String,
    pub resume_request: TraceReplaySliceRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceReplaySliceAnchor {
    TaskRoot,
    Turn(TurnTraceId),
    Branch(TraceBranchId),
    Record(TraceRecordId),
    Checkpoint(TraceCheckpointId),
    Tail,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub enum TraceReplaySliceDirection {
    Forward,
    Backward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceReplaySliceRequest {
    pub anchor: TraceReplaySliceAnchor,
    pub direction: TraceReplaySliceDirection,
    pub include_anchor: bool,
    pub limit: Option<usize>,
}

impl TraceReplaySliceRequest {
    pub fn from_anchor(anchor: TraceReplaySliceAnchor) -> Self {
        Self {
            anchor,
            direction: TraceReplaySliceDirection::Forward,
            include_anchor: true,
            limit: None,
        }
    }

    pub fn backward_from_anchor(anchor: TraceReplaySliceAnchor, limit: Option<usize>) -> Self {
        Self {
            anchor,
            direction: TraceReplaySliceDirection::Backward,
            include_anchor: true,
            limit,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceReplaySlice {
    pub task_id: TaskTraceId,
    pub request: TraceReplaySliceRequest,
    pub anchor_record_id: TraceRecordId,
    pub records: Vec<TraceRecord>,
}

impl TraceReplaySlice {
    pub fn from_replay(replay: &TraceReplay, request: &TraceReplaySliceRequest) -> Result<Self> {
        let anchor_index = resolve_anchor_index(&replay.records, &request.anchor)?;
        let anchor_record_id = replay.records[anchor_index].record_id.clone();
        let records = match request.direction {
            TraceReplaySliceDirection::Forward => {
                let start = if request.include_anchor {
                    anchor_index
                } else {
                    anchor_index + 1
                };
                let mut records = replay
                    .records
                    .iter()
                    .skip(start)
                    .cloned()
                    .collect::<Vec<_>>();
                if let Some(limit) = request.limit {
                    records.truncate(limit);
                }
                records
            }
            TraceReplaySliceDirection::Backward => {
                let end = if request.include_anchor {
                    anchor_index + 1
                } else {
                    anchor_index
                };
                let mut records = replay.records[..end].to_vec();
                if let Some(limit) = request.limit
                    && records.len() > limit
                {
                    records = records.split_off(records.len() - limit);
                }
                records
            }
        };

        Ok(Self {
            task_id: replay.task_id.clone(),
            request: request.clone(),
            anchor_record_id,
            records,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceSessionContextQuery {
    AdaptiveReplay {
        turn_limit: usize,
    },
    Rewind {
        anchor: TraceReplaySliceAnchor,
        record_limit: usize,
    },
    CompactionWindow {
        anchor: TraceReplaySliceAnchor,
        before_record_limit: usize,
        after_record_limit: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraceSessionContextSlice {
    pub task_id: TaskTraceId,
    pub query: TraceSessionContextQuery,
    pub wake: TraceSessionWake,
    pub anchor_record_id: Option<TraceRecordId>,
    pub records: Vec<TraceRecord>,
    pub transcript: ConversationTranscript,
    pub turn_summaries: Vec<String>,
}

impl TraceSessionContextSlice {
    pub fn from_replay(
        replay: &TraceReplay,
        wake: &TraceSessionWake,
        query: &TraceSessionContextQuery,
    ) -> Result<Self> {
        let (records, anchor_record_id) = match query {
            TraceSessionContextQuery::AdaptiveReplay { turn_limit } => {
                if replay.records.is_empty() || *turn_limit == 0 {
                    (Vec::new(), None)
                } else {
                    let ordered_turns = ordered_turns(replay);
                    let retained_turns = ordered_turns
                        .into_iter()
                        .rev()
                        .take(*turn_limit)
                        .collect::<Vec<_>>();
                    let mut selected = replay
                        .records
                        .iter()
                        .filter(|record| retained_turns.contains(&record.lineage.turn_id))
                        .cloned()
                        .collect::<Vec<_>>();
                    selected.sort_by_key(|record| record.sequence);
                    let anchor = selected.first().map(|record| record.record_id.clone());
                    (selected, anchor)
                }
            }
            TraceSessionContextQuery::Rewind {
                anchor,
                record_limit,
            } => {
                let request = TraceReplaySliceRequest::backward_from_anchor(
                    anchor.clone(),
                    Some(*record_limit),
                );
                let slice = TraceReplaySlice::from_replay(replay, &request)?;
                (slice.records, Some(slice.anchor_record_id))
            }
            TraceSessionContextQuery::CompactionWindow {
                anchor,
                before_record_limit,
                after_record_limit,
            } => {
                let anchor_index = resolve_anchor_index(&replay.records, anchor)?;
                let start = anchor_index.saturating_sub(*before_record_limit);
                let end = (anchor_index + *after_record_limit)
                    .min(replay.records.len().saturating_sub(1));
                (
                    replay.records[start..=end].to_vec(),
                    Some(replay.records[anchor_index].record_id.clone()),
                )
            }
        };

        let transcript = ConversationTranscript::from_trace_replay(&TraceReplay {
            task_id: replay.task_id.clone(),
            records: records.clone(),
        });
        let turn_summaries = summarize_transcript_turns(&transcript);

        Ok(Self {
            task_id: replay.task_id.clone(),
            query: query.clone(),
            wake: wake.clone(),
            anchor_record_id,
            records,
            transcript,
            turn_summaries,
        })
    }
}

pub trait TraceRecorder: Send + Sync {
    fn as_any(&self) -> &dyn Any;

    fn capability(&self) -> TraceRecorderCapability;

    fn record(&self, record: TraceRecord) -> Result<()>;

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay>;

    fn wake(&self, task_id: &TaskTraceId) -> Result<TraceSessionWake> {
        let replay = self.replay(task_id)?;
        Ok(TraceSessionWake::from_replay(&replay))
    }

    fn replay_slice(
        &self,
        task_id: &TaskTraceId,
        request: &TraceReplaySliceRequest,
    ) -> Result<TraceReplaySlice> {
        let replay = self.replay(task_id)?;
        TraceReplaySlice::from_replay(&replay, request)
    }

    fn resume_from_checkpoint(
        &self,
        task_id: &TaskTraceId,
        checkpoint_id: &TraceCheckpointId,
    ) -> Result<TraceReplaySlice> {
        self.replay_slice(
            task_id,
            &TraceReplaySliceRequest::from_anchor(TraceReplaySliceAnchor::Checkpoint(
                checkpoint_id.clone(),
            )),
        )
    }

    fn query_session_context(
        &self,
        task_id: &TaskTraceId,
        query: &TraceSessionContextQuery,
    ) -> Result<TraceSessionContextSlice> {
        let replay = self.replay(task_id)?;
        let wake = self.wake(task_id)?;
        TraceSessionContextSlice::from_replay(&replay, &wake, query)
    }

    fn task_ids(&self) -> Vec<TaskTraceId> {
        Vec::new()
    }
}

#[derive(Default)]
pub struct NoopTraceRecorder;

impl TraceRecorder for NoopTraceRecorder {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn capability(&self) -> TraceRecorderCapability {
        TraceRecorderCapability::Unsupported {
            reason: "trace recording is disabled".to_string(),
        }
    }

    fn record(&self, _record: TraceRecord) -> Result<()> {
        Ok(())
    }

    fn replay(&self, task_id: &TaskTraceId) -> Result<TraceReplay> {
        Ok(TraceReplay {
            task_id: task_id.clone(),
            records: Vec::new(),
        })
    }
}

fn resolve_anchor_index(records: &[TraceRecord], anchor: &TraceReplaySliceAnchor) -> Result<usize> {
    if records.is_empty() {
        return Err(anyhow!(
            "trace replay is empty; no slice anchor is available"
        ));
    }

    let index = match anchor {
        TraceReplaySliceAnchor::TaskRoot => records
            .iter()
            .position(|record| matches!(record.kind, TraceRecordKind::TaskRootStarted(_)))
            .unwrap_or(0),
        TraceReplaySliceAnchor::Turn(turn_id) => records
            .iter()
            .position(|record| record.lineage.turn_id == *turn_id)
            .ok_or_else(|| anyhow!("turn slice anchor '{}' was not found", turn_id.as_str()))?,
        TraceReplaySliceAnchor::Branch(branch_id) => records
            .iter()
            .position(|record| record.lineage.branch_id.as_ref() == Some(branch_id))
            .ok_or_else(|| anyhow!("branch slice anchor '{}' was not found", branch_id.as_str()))?,
        TraceReplaySliceAnchor::Record(record_id) => records
            .iter()
            .position(|record| record.record_id == *record_id)
            .ok_or_else(|| anyhow!("record slice anchor '{}' was not found", record_id.as_str()))?,
        TraceReplaySliceAnchor::Checkpoint(checkpoint_id) => records
            .iter()
            .position(|record| {
                matches!(
                    &record.kind,
                    TraceRecordKind::CompletionCheckpoint(checkpoint)
                        if checkpoint.checkpoint_id == *checkpoint_id
                )
            })
            .ok_or_else(|| {
                anyhow!(
                    "checkpoint slice anchor '{}' was not found",
                    checkpoint_id.as_str()
                )
            })?,
        TraceReplaySliceAnchor::Tail => records.len() - 1,
    };

    Ok(index)
}

fn ordered_turns(replay: &TraceReplay) -> Vec<TurnTraceId> {
    let mut turn_ids = Vec::new();
    for record in &replay.records {
        if !turn_ids.contains(&record.lineage.turn_id) {
            turn_ids.push(record.lineage.turn_id.clone());
        }
    }
    turn_ids
}

fn summarize_transcript_turns(transcript: &ConversationTranscript) -> Vec<String> {
    let mut ordered_turns = Vec::<TurnTraceId>::new();
    let mut prompts = std::collections::HashMap::<TurnTraceId, String>::new();
    let mut replies = std::collections::HashMap::<TurnTraceId, String>::new();

    for entry in &transcript.entries {
        if !ordered_turns.contains(&entry.turn_id) {
            ordered_turns.push(entry.turn_id.clone());
        }

        match entry.speaker {
            ConversationTranscriptSpeaker::User => {
                prompts
                    .entry(entry.turn_id.clone())
                    .or_insert_with(|| entry.content.clone());
            }
            ConversationTranscriptSpeaker::Assistant => {
                replies
                    .entry(entry.turn_id.clone())
                    .or_insert_with(|| entry.content.clone());
            }
            ConversationTranscriptSpeaker::System => {}
        }
    }

    ordered_turns
        .into_iter()
        .filter_map(|turn_id| {
            let prompt = prompts.get(&turn_id)?;
            let reply = replies.get(&turn_id);
            Some(match reply {
                Some(reply) => format!("Q: {prompt} A: {reply}"),
                None => format!("Q: {prompt}"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{TraceReplaySliceAnchor, TraceSessionContextQuery, TraceSessionWake};
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, ConversationTranscript, ConversationTranscriptSpeaker,
        TraceCheckpointKind, TraceCompletionCheckpoint, TraceHarnessProfileSelection, TraceLineage,
        TraceRecord, TraceRecordKind, TraceReplay, TraceTaskRoot, TraceTurnStarted,
    };
    use paddles_conversation::{
        TaskTraceId, TraceArtifactId, TraceCheckpointId, TraceRecordId, TurnTraceId,
    };

    #[test]
    fn adaptive_replay_query_returns_recent_turn_summaries() {
        let replay = sample_replay();
        let wake = TraceSessionWake::from_replay(&replay);
        let slice = super::TraceSessionContextSlice::from_replay(
            &replay,
            &wake,
            &TraceSessionContextQuery::AdaptiveReplay { turn_limit: 1 },
        )
        .expect("adaptive replay slice");

        assert_eq!(slice.records.len(), 2);
        assert_eq!(slice.transcript.entries.len(), 2);
        assert_eq!(
            slice.turn_summaries,
            vec!["Q: second prompt A: second reply".to_string()]
        );
        assert_eq!(
            slice.transcript.entries[0].speaker,
            ConversationTranscriptSpeaker::User
        );
    }

    #[test]
    fn transcript_turn_summaries_ignore_system_governance_entries() {
        let transcript = ConversationTranscript {
            task_id: TaskTraceId::new("task-1").expect("task"),
            entries: vec![
                crate::domain::model::ConversationTranscriptEntry {
                    record_id: TraceRecordId::new("record-1").expect("record"),
                    turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                    speaker: ConversationTranscriptSpeaker::User,
                    content: "hello".to_string(),
                    response_mode: None,
                    render: None,
                    citations: Vec::new(),
                    grounded: None,
                },
                crate::domain::model::ConversationTranscriptEntry {
                    record_id: TraceRecordId::new("record-2").expect("record"),
                    turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                    speaker: ConversationTranscriptSpeaker::System,
                    content: "execution posture recursive-structured-v1".to_string(),
                    response_mode: None,
                    render: None,
                    citations: Vec::new(),
                    grounded: None,
                },
                crate::domain::model::ConversationTranscriptEntry {
                    record_id: TraceRecordId::new("record-3").expect("record"),
                    turn_id: TurnTraceId::new("task-1.turn-0001").expect("turn"),
                    speaker: ConversationTranscriptSpeaker::Assistant,
                    content: "hi".to_string(),
                    response_mode: None,
                    render: None,
                    citations: Vec::new(),
                    grounded: Some(false),
                },
            ],
        };

        assert_eq!(
            super::summarize_transcript_turns(&transcript),
            vec!["Q: hello A: hi".to_string()]
        );
    }

    #[test]
    fn compaction_window_query_returns_anchor_neighborhood() {
        let replay = sample_replay();
        let wake = TraceSessionWake::from_replay(&replay);
        let anchor = TraceRecordId::new("record-3").expect("record id");
        let slice = super::TraceSessionContextSlice::from_replay(
            &replay,
            &wake,
            &TraceSessionContextQuery::CompactionWindow {
                anchor: TraceReplaySliceAnchor::Record(anchor.clone()),
                before_record_limit: 1,
                after_record_limit: 1,
            },
        )
        .expect("compaction window");

        let record_ids = slice
            .records
            .iter()
            .map(|record| record.record_id.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(slice.anchor_record_id, Some(anchor));
        assert_eq!(
            record_ids,
            vec![
                "record-2".to_string(),
                "record-3".to_string(),
                "record-4".to_string()
            ]
        );
    }

    #[test]
    fn rewind_query_returns_backward_slice_from_anchor() {
        let replay = sample_replay();
        let wake = TraceSessionWake::from_replay(&replay);
        let anchor = TraceRecordId::new("record-4").expect("record id");
        let slice = super::TraceSessionContextSlice::from_replay(
            &replay,
            &wake,
            &TraceSessionContextQuery::Rewind {
                anchor: TraceReplaySliceAnchor::Record(anchor.clone()),
                record_limit: 2,
            },
        )
        .expect("rewind slice");

        let record_ids = slice
            .records
            .iter()
            .map(|record| record.record_id.as_str().to_string())
            .collect::<Vec<_>>();
        assert_eq!(slice.anchor_record_id, Some(anchor));
        assert_eq!(
            record_ids,
            vec!["record-3".to_string(), "record-4".to_string()]
        );
    }

    fn sample_replay() -> TraceReplay {
        let task_id = TaskTraceId::new("task-context").expect("task id");
        let turn_one = TurnTraceId::new("task-context.turn-0001").expect("turn id");
        let turn_two = TurnTraceId::new("task-context.turn-0002").expect("turn id");
        TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("record-1").expect("record id"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_one.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "first prompt",
                            200,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                        harness_profile: sample_harness_profile(),
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-2").expect("record id"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_one,
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-1").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-1")
                            .expect("checkpoint id"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "first done".to_string(),
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "reply",
                            "first reply",
                            200,
                        )),
                        authored_response: None,
                        citations: Vec::new(),
                        grounded: false,
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-3").expect("record id"),
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_two.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-2").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::TurnStarted(TraceTurnStarted {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-3").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "second prompt",
                            200,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                        harness_profile: sample_harness_profile(),
                        thread: crate::domain::model::ConversationThreadRef::Mainline,
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("record-4").expect("record id"),
                    sequence: 4,
                    lineage: TraceLineage {
                        task_id,
                        turn_id: turn_two,
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("record-3").expect("parent record"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-2")
                            .expect("checkpoint id"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "second done".to_string(),
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-4").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "reply",
                            "second reply",
                            200,
                        )),
                        authored_response: None,
                        citations: Vec::new(),
                        grounded: true,
                    }),
                },
            ],
        }
    }

    fn sample_harness_profile() -> TraceHarnessProfileSelection {
        TraceHarnessProfileSelection {
            requested_profile_id: "recursive-structured-v1".to_string(),
            active_profile_id: "recursive-structured-v1".to_string(),
            downgrade_reason: None,
        }
    }
}
