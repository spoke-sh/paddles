use super::{
    TaskTraceId, TraceModelExchangeArtifact, TraceRecord, TraceRecordId, TraceRecordKind,
    TraceReplay, TurnTraceId,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ForensicLifecycle {
    Provisional,
    Superseded,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForensicRecordProjection {
    pub lifecycle: ForensicLifecycle,
    pub superseded_by_record_id: Option<TraceRecordId>,
    pub record: TraceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForensicTurnProjection {
    pub turn_id: TurnTraceId,
    pub lifecycle: ForensicLifecycle,
    pub records: Vec<ForensicRecordProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationForensicProjection {
    pub task_id: TaskTraceId,
    pub turns: Vec<ForensicTurnProjection>,
}

impl ConversationForensicProjection {
    pub fn from_trace_replay(replay: &TraceReplay) -> Self {
        let completed_turns = replay
            .records
            .iter()
            .filter(|record| matches!(record.kind, TraceRecordKind::CompletionCheckpoint(_)))
            .map(|record| record.lineage.turn_id.clone())
            .collect::<HashSet<_>>();

        let latest_exchange_by_group = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::ModelExchangeArtifact(artifact) => Some((
                    (
                        record.lineage.turn_id.as_str().to_string(),
                        artifact.lane.label().to_string(),
                        artifact.category.label().to_string(),
                    ),
                    (artifact.exchange_id.clone(), record.record_id.clone()),
                )),
                _ => None,
            })
            .fold(
                HashMap::<(String, String, String), (String, TraceRecordId)>::new(),
                |mut acc, (group, latest)| {
                    acc.insert(group, latest);
                    acc
                },
            );

        let superseded_exchange_targets = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::ModelExchangeArtifact(artifact) => Some((record, artifact)),
                _ => None,
            })
            .filter_map(|(record, artifact)| {
                let group = (
                    record.lineage.turn_id.as_str().to_string(),
                    artifact.lane.label().to_string(),
                    artifact.category.label().to_string(),
                );
                latest_exchange_by_group
                    .get(&group)
                    .filter(|(exchange_id, _)| exchange_id != &artifact.exchange_id)
                    .map(|(_, record_id)| (artifact.exchange_id.clone(), record_id.clone()))
            })
            .collect::<HashMap<_, _>>();

        let mut turns = Vec::new();
        let mut records_by_turn = HashMap::<TurnTraceId, Vec<&TraceRecord>>::new();
        for record in &replay.records {
            records_by_turn
                .entry(record.lineage.turn_id.clone())
                .or_default()
                .push(record);
        }

        let mut ordered_turns = records_by_turn.into_iter().collect::<Vec<_>>();
        ordered_turns.sort_by_key(|(_, records)| records.first().map(|record| record.sequence));

        for (turn_id, records) in ordered_turns {
            let turn_lifecycle = if completed_turns.contains(&turn_id) {
                ForensicLifecycle::Final
            } else {
                ForensicLifecycle::Provisional
            };
            let records = records
                .into_iter()
                .map(|record| {
                    let superseded_by_record_id =
                        superseded_by_record(record, &superseded_exchange_targets);
                    let lifecycle = superseded_by_record_id
                        .as_ref()
                        .map(|_| ForensicLifecycle::Superseded)
                        .unwrap_or(turn_lifecycle);
                    ForensicRecordProjection {
                        lifecycle,
                        superseded_by_record_id,
                        record: record.clone(),
                    }
                })
                .collect();
            turns.push(ForensicTurnProjection {
                turn_id,
                lifecycle: turn_lifecycle,
                records,
            });
        }

        Self {
            task_id: replay.task_id.clone(),
            turns,
        }
    }

    pub fn turn(&self, turn_id: &TurnTraceId) -> Option<ForensicTurnProjection> {
        self.turns
            .iter()
            .find(|turn| &turn.turn_id == turn_id)
            .cloned()
    }
}

fn superseded_by_record(
    record: &TraceRecord,
    superseded_exchange_targets: &HashMap<String, TraceRecordId>,
) -> Option<TraceRecordId> {
    match &record.kind {
        TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
            exchange_id, ..
        }) => superseded_exchange_targets.get(exchange_id).cloned(),
        TraceRecordKind::LineageEdge(edge) => extract_superseded_exchange_id(edge)
            .and_then(|exchange_id| superseded_exchange_targets.get(exchange_id).cloned()),
        _ => None,
    }
}

fn extract_superseded_exchange_id(edge: &super::TraceLineageEdge) -> Option<&str> {
    [edge.source.id.as_str(), edge.target.id.as_str()]
        .into_iter()
        .find_map(|id| id.strip_prefix("model-call:"))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConversationForensicUpdate {
    pub task_id: TaskTraceId,
    pub turn_id: TurnTraceId,
    pub record_id: TraceRecordId,
}

pub trait ForensicUpdateSink: Send + Sync {
    fn emit(&self, update: ConversationForensicUpdate);
}

#[derive(Default)]
pub struct NullForensicUpdateSink;

impl ForensicUpdateSink for NullForensicUpdateSink {
    fn emit(&self, _update: ConversationForensicUpdate) {}
}

#[cfg(test)]
mod tests {
    use super::{ConversationForensicProjection, ForensicLifecycle};
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, TaskTraceId, TraceLineage, TraceModelExchangeArtifact,
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase, TraceRecord,
        TraceRecordId, TraceRecordKind, TraceReplay, TurnTraceId,
    };
    use paddles_conversation::TraceArtifactId;

    #[test]
    fn projection_marks_replaced_model_call_records_as_superseded() {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let turn_id = TurnTraceId::new("task-1.turn-0001").expect("turn");
        let first = TraceRecord {
            record_id: TraceRecordId::new("task-1.turn-0001.record-0001").expect("record"),
            sequence: 1,
            lineage: TraceLineage {
                task_id: task_id.clone(),
                turn_id: turn_id.clone(),
                branch_id: None,
                parent_record_id: None,
            },
            kind: TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                exchange_id: "exchange-1".to_string(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::AssembledContext,
                provider: "openai".to_string(),
                model: "gpt".to_string(),
                parent_artifact_id: None,
                artifact: ArtifactEnvelope::text(
                    TraceArtifactId::new("artifact-1").expect("artifact"),
                    ArtifactKind::Prompt,
                    "first",
                    "{}",
                    usize::MAX,
                ),
            }),
        };
        let second = TraceRecord {
            record_id: TraceRecordId::new("task-1.turn-0001.record-0002").expect("record"),
            sequence: 2,
            lineage: TraceLineage {
                task_id: task_id.clone(),
                turn_id: turn_id.clone(),
                branch_id: None,
                parent_record_id: Some(first.record_id.clone()),
            },
            kind: TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                exchange_id: "exchange-2".to_string(),
                lane: TraceModelExchangeLane::Planner,
                category: TraceModelExchangeCategory::PlannerAction,
                phase: TraceModelExchangePhase::AssembledContext,
                provider: "openai".to_string(),
                model: "gpt".to_string(),
                parent_artifact_id: None,
                artifact: ArtifactEnvelope::text(
                    TraceArtifactId::new("artifact-2").expect("artifact"),
                    ArtifactKind::Prompt,
                    "second",
                    "{}",
                    usize::MAX,
                ),
            }),
        };

        let projection = ConversationForensicProjection::from_trace_replay(&TraceReplay {
            task_id,
            records: vec![first.clone(), second.clone()],
        });

        let records = &projection.turns[0].records;
        assert_eq!(records[0].lifecycle, ForensicLifecycle::Superseded);
        assert_eq!(records[1].lifecycle, ForensicLifecycle::Provisional);
        assert_eq!(
            records[0]
                .superseded_by_record_id
                .as_ref()
                .map(|id| id.as_str()),
            Some(second.record_id.as_str())
        );
    }
}
