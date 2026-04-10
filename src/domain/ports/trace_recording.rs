use crate::domain::model::{
    TaskTraceId, TraceBranchId, TraceCheckpointId, TraceCheckpointKind, TraceRecord, TraceRecordId,
    TraceRecordKind, TraceReplay, TurnTraceId,
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
