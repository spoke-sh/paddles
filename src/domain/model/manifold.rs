use super::{
    ArtifactEnvelope, ConversationForensicProjection, ForensicLifecycle, TaskTraceId,
    TraceLineageNodeRef, TraceRecordId, TraceRecordKind, TraceReplay, TraceSignalContribution,
    TraceSignalKind, TraceSignalSnapshot, TurnTraceId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifoldSignalState {
    pub snapshot_record_id: TraceRecordId,
    pub lifecycle: ForensicLifecycle,
    pub kind: TraceSignalKind,
    pub summary: String,
    pub level: String,
    pub magnitude_percent: u8,
    pub anchor: Option<TraceLineageNodeRef>,
    pub contributions: Vec<TraceSignalContribution>,
    pub artifact: ArtifactEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifoldFrame {
    pub record_id: TraceRecordId,
    pub sequence: u64,
    pub lifecycle: ForensicLifecycle,
    pub anchor: Option<TraceLineageNodeRef>,
    pub active_signals: Vec<ManifoldSignalState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifoldTurnProjection {
    pub turn_id: TurnTraceId,
    pub lifecycle: ForensicLifecycle,
    pub frames: Vec<ManifoldFrame>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationManifoldProjection {
    pub task_id: TaskTraceId,
    pub turns: Vec<ManifoldTurnProjection>,
}

impl ConversationManifoldProjection {
    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        let forensic = ConversationForensicProjection::from_trace_replay(replay);
        let turns = forensic
            .turns
            .into_iter()
            .map(|turn| {
                let mut active_states = BTreeMap::<String, ManifoldSignalState>::new();
                let mut frames = Vec::new();

                for record_projection in &turn.records {
                    match &record_projection.record.kind {
                        TraceRecordKind::SignalSnapshot(snapshot) => {
                            let state = signal_state_from_snapshot(
                                &record_projection.record.record_id,
                                record_projection.lifecycle,
                                snapshot,
                            );
                            active_states.insert(snapshot.kind.label().to_string(), state);
                            frames.push(frame_from_active_states(
                                &record_projection.record.record_id,
                                record_projection.record.sequence,
                                record_projection.lifecycle,
                                snapshot.applies_to.clone(),
                                &active_states,
                            ));
                        }
                        TraceRecordKind::CompletionCheckpoint(_) if !active_states.is_empty() => {
                            frames.push(frame_from_active_states(
                                &record_projection.record.record_id,
                                record_projection.record.sequence,
                                record_projection.lifecycle,
                                None,
                                &active_states,
                            ));
                        }
                        _ => {}
                    }
                }

                ManifoldTurnProjection {
                    turn_id: turn.turn_id,
                    lifecycle: turn.lifecycle,
                    frames,
                }
            })
            .collect();

        Self {
            task_id: forensic.task_id,
            turns,
        }
    }

    pub fn turn(&self, turn_id: &TurnTraceId) -> Option<ManifoldTurnProjection> {
        self.turns
            .iter()
            .find(|turn| &turn.turn_id == turn_id)
            .cloned()
    }
}

fn signal_state_from_snapshot(
    record_id: &TraceRecordId,
    lifecycle: ForensicLifecycle,
    snapshot: &TraceSignalSnapshot,
) -> ManifoldSignalState {
    ManifoldSignalState {
        snapshot_record_id: record_id.clone(),
        lifecycle,
        kind: snapshot.kind,
        summary: snapshot.summary.clone(),
        level: snapshot.level.clone(),
        magnitude_percent: snapshot.magnitude_percent,
        anchor: snapshot.applies_to.clone(),
        contributions: snapshot.contributions.clone(),
        artifact: snapshot.artifact.clone(),
    }
}

fn frame_from_active_states(
    record_id: &TraceRecordId,
    sequence: u64,
    lifecycle: ForensicLifecycle,
    anchor: Option<TraceLineageNodeRef>,
    active_states: &BTreeMap<String, ManifoldSignalState>,
) -> ManifoldFrame {
    let active_signals = active_states
        .values()
        .cloned()
        .map(|mut state| {
            state.lifecycle = lifecycle;
            state
        })
        .collect();

    ManifoldFrame {
        record_id: record_id.clone(),
        sequence,
        lifecycle,
        anchor,
        active_signals,
    }
}

#[cfg(test)]
mod tests {
    use super::{ConversationManifoldProjection, ForensicLifecycle};
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, TraceCheckpointKind,
        TraceCompletionCheckpoint, TraceLineage, TraceLineageNodeKind, TraceLineageNodeRef,
        TraceRecord, TraceRecordId, TraceRecordKind, TraceReplay, TraceSignalContribution,
        TraceSignalKind, TraceSignalSnapshot, TurnTraceId,
    };
    use paddles_conversation::{TraceArtifactId, TraceCheckpointId};

    #[test]
    fn projection_builds_cumulative_frames_from_signal_snapshots() {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let turn_id = TurnTraceId::new("task-1.turn-0001").expect("turn");
        let signal_record_id =
            TraceRecordId::new("task-1.turn-0001.record-0001").expect("signal record");
        let checkpoint_record_id =
            TraceRecordId::new("task-1.turn-0001.record-0002").expect("checkpoint record");

        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: signal_record_id.clone(),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                        kind: TraceSignalKind::ActionBias,
                        summary: "action bias".to_string(),
                        level: "high".to_string(),
                        magnitude_percent: 80,
                        applies_to: Some(TraceLineageNodeRef {
                            kind: TraceLineageNodeKind::Turn,
                            id: turn_id.as_str().to_string(),
                            label: "turn".to_string(),
                        }),
                        contributions: vec![TraceSignalContribution {
                            source: "controller".to_string(),
                            share_percent: 100,
                            rationale: "test".to_string(),
                        }],
                        artifact: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Checkpoint,
                            "signal",
                            "{\"kind\":\"action_bias\"}",
                            usize::MAX,
                        ),
                    }),
                },
                TraceRecord {
                    record_id: checkpoint_record_id.clone(),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id,
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(signal_record_id.clone()),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("task-1.turn-0001.checkpoint")
                            .expect("checkpoint"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "done".to_string(),
                        response: None,
                        citations: Vec::new(),
                        grounded: true,
                    }),
                },
            ],
        };

        let projection = ConversationManifoldProjection::from_trace_replay(&replay);
        let turn = projection.turn(&turn_id).expect("turn projection");

        assert_eq!(turn.frames.len(), 2);
        assert_eq!(turn.frames[0].record_id, signal_record_id);
        assert_eq!(turn.frames[0].active_signals.len(), 1);
        assert_eq!(
            turn.frames[0].active_signals[0].kind,
            TraceSignalKind::ActionBias
        );
        assert_eq!(turn.frames[1].record_id, checkpoint_record_id);
        assert_eq!(turn.frames[1].lifecycle, ForensicLifecycle::Final);
    }
}
