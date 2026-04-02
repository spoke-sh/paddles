use super::{
    ArtifactEnvelope, ConversationForensicProjection, ForensicLifecycle, TaskTraceId,
    TraceLineageNodeKind, TraceLineageNodeRef, TraceRecordId, TraceRecordKind, TraceReplay,
    TraceSignalContribution, TraceSignalKind, TraceSignalSnapshot, TurnTraceId,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ManifoldPrimitiveKind {
    Chamber,
    Reservoir,
    Valve,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ManifoldPrimitiveBasis {
    SignalFamily { signal_kind: TraceSignalKind },
    LineageAnchor { anchor: TraceLineageNodeRef },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifoldPrimitiveState {
    pub primitive_id: String,
    pub kind: ManifoldPrimitiveKind,
    pub label: String,
    pub basis: ManifoldPrimitiveBasis,
    pub evidence_record_id: Option<TraceRecordId>,
    pub anchor: Option<TraceLineageNodeRef>,
    pub level: String,
    pub magnitude_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifoldConduitState {
    pub conduit_id: String,
    pub from_primitive_id: String,
    pub to_primitive_id: String,
    pub label: String,
    pub basis: ManifoldPrimitiveBasis,
    pub evidence_record_id: Option<TraceRecordId>,
}

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
    pub primitives: Vec<ManifoldPrimitiveState>,
    pub conduits: Vec<ManifoldConduitState>,
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
    let (primitives, conduits) = topology_from_active_states(active_states);

    ManifoldFrame {
        record_id: record_id.clone(),
        sequence,
        lifecycle,
        anchor,
        active_signals,
        primitives,
        conduits,
    }
}

fn topology_from_active_states(
    active_states: &BTreeMap<String, ManifoldSignalState>,
) -> (Vec<ManifoldPrimitiveState>, Vec<ManifoldConduitState>) {
    let mut primitives = BTreeMap::<String, ManifoldPrimitiveState>::new();
    let mut conduits = BTreeMap::<String, ManifoldConduitState>::new();

    for state in active_states.values() {
        let family_id = signal_family_primitive_id(state.kind);
        let (family_kind, family_label) = signal_family_descriptor(state.kind);
        insert_or_strengthen_primitive(
            &mut primitives,
            ManifoldPrimitiveState {
                primitive_id: family_id.clone(),
                kind: family_kind,
                label: family_label.to_string(),
                basis: ManifoldPrimitiveBasis::SignalFamily {
                    signal_kind: state.kind,
                },
                evidence_record_id: Some(state.snapshot_record_id.clone()),
                anchor: state.anchor.clone(),
                level: state.level.clone(),
                magnitude_percent: state.magnitude_percent,
            },
        );

        if let Some(anchor) = &state.anchor {
            let anchor_id = lineage_anchor_primitive_id(anchor);
            insert_or_strengthen_primitive(
                &mut primitives,
                ManifoldPrimitiveState {
                    primitive_id: anchor_id.clone(),
                    kind: lineage_anchor_kind(anchor.kind),
                    label: lineage_anchor_label(anchor),
                    basis: ManifoldPrimitiveBasis::LineageAnchor {
                        anchor: anchor.clone(),
                    },
                    evidence_record_id: None,
                    anchor: Some(anchor.clone()),
                    level: state.level.clone(),
                    magnitude_percent: state.magnitude_percent,
                },
            );

            let conduit_id = format!("conduit:{family_id}->{anchor_id}");
            conduits
                .entry(conduit_id.clone())
                .or_insert_with(|| ManifoldConduitState {
                    conduit_id,
                    from_primitive_id: family_id.clone(),
                    to_primitive_id: anchor_id,
                    label: format!("{} feeds {}", family_label, lineage_anchor_label(anchor)),
                    basis: ManifoldPrimitiveBasis::LineageAnchor {
                        anchor: anchor.clone(),
                    },
                    evidence_record_id: Some(state.snapshot_record_id.clone()),
                });
        }
    }

    (
        primitives.into_values().collect(),
        conduits.into_values().collect(),
    )
}

fn insert_or_strengthen_primitive(
    primitives: &mut BTreeMap<String, ManifoldPrimitiveState>,
    candidate: ManifoldPrimitiveState,
) {
    match primitives.get_mut(&candidate.primitive_id) {
        Some(existing) if candidate.magnitude_percent >= existing.magnitude_percent => {
            *existing = candidate;
        }
        Some(_) => {}
        None => {
            primitives.insert(candidate.primitive_id.clone(), candidate);
        }
    }
}

fn signal_family_primitive_id(kind: TraceSignalKind) -> String {
    format!("family:{}", kind.label())
}

fn signal_family_descriptor(kind: TraceSignalKind) -> (ManifoldPrimitiveKind, &'static str) {
    match kind {
        TraceSignalKind::ContextStrain => {
            (ManifoldPrimitiveKind::Chamber, "Context strain chamber")
        }
        TraceSignalKind::CompactionCue => {
            (ManifoldPrimitiveKind::Reservoir, "Compaction reservoir")
        }
        TraceSignalKind::ActionBias => (ManifoldPrimitiveKind::Valve, "Action bias valve"),
        TraceSignalKind::Fallback => (ManifoldPrimitiveKind::Valve, "Fallback bypass valve"),
        TraceSignalKind::BudgetBoundary => (ManifoldPrimitiveKind::Valve, "Budget boundary valve"),
    }
}

fn lineage_anchor_kind(kind: TraceLineageNodeKind) -> ManifoldPrimitiveKind {
    match kind {
        TraceLineageNodeKind::Conversation | TraceLineageNodeKind::Turn => {
            ManifoldPrimitiveKind::Reservoir
        }
        TraceLineageNodeKind::ModelCall
        | TraceLineageNodeKind::PlannerStep
        | TraceLineageNodeKind::Artifact
        | TraceLineageNodeKind::Output
        | TraceLineageNodeKind::Signal => ManifoldPrimitiveKind::Chamber,
    }
}

fn lineage_anchor_primitive_id(anchor: &TraceLineageNodeRef) -> String {
    format!(
        "anchor:{}:{}",
        anchor.kind.label(),
        sanitize_anchor_id(&anchor.id)
    )
}

fn lineage_anchor_label(anchor: &TraceLineageNodeRef) -> String {
    if anchor.label.trim().is_empty() {
        anchor.kind.label().replace('_', " ")
    } else {
        anchor.label.clone()
    }
}

fn sanitize_anchor_id(id: &str) -> String {
    id.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        ConversationManifoldProjection, ForensicLifecycle, ManifoldPrimitiveBasis,
        ManifoldPrimitiveKind,
    };
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

    #[test]
    fn projection_maps_signal_families_and_lineage_anchors_into_stable_primitives() {
        let task_id = TaskTraceId::new("task-2").expect("task");
        let turn_id = TurnTraceId::new("task-2.turn-0001").expect("turn");
        let signal_record_id =
            TraceRecordId::new("task-2.turn-0001.record-0001").expect("signal record");

        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![TraceRecord {
                record_id: signal_record_id.clone(),
                sequence: 1,
                lineage: TraceLineage {
                    task_id,
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
                        TraceArtifactId::new("artifact-2").expect("artifact"),
                        ArtifactKind::Checkpoint,
                        "signal",
                        "{\"kind\":\"action_bias\"}",
                        usize::MAX,
                    ),
                }),
            }],
        };

        let projection = ConversationManifoldProjection::from_trace_replay(&replay);
        let turn = projection.turn(&turn_id).expect("turn projection");
        let frame = &turn.frames[0];

        assert!(
            frame
                .primitives
                .iter()
                .any(|primitive| primitive.primitive_id == "family:action_bias"
                    && primitive.kind == ManifoldPrimitiveKind::Valve
                    && primitive.evidence_record_id.as_ref() == Some(&signal_record_id))
        );
        assert!(frame.primitives.iter().any(|primitive| matches!(
            primitive.basis,
            ManifoldPrimitiveBasis::LineageAnchor { .. }
        )));
        assert!(frame.conduits.iter().any(|conduit| {
            conduit.from_primitive_id == "family:action_bias"
                && conduit.evidence_record_id.as_ref() == Some(&signal_record_id)
        }));
    }

    #[test]
    fn projected_topology_keeps_evidence_or_lineage_basis_for_every_primitive() {
        let task_id = TaskTraceId::new("task-3").expect("task");
        let turn_id = TurnTraceId::new("task-3.turn-0001").expect("turn");
        let signal_record_id =
            TraceRecordId::new("task-3.turn-0001.record-0001").expect("signal record");

        let replay = TraceReplay {
            task_id: task_id.clone(),
            records: vec![TraceRecord {
                record_id: signal_record_id.clone(),
                sequence: 1,
                lineage: TraceLineage {
                    task_id,
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: None,
                },
                kind: TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                    kind: TraceSignalKind::ContextStrain,
                    summary: "context strain".to_string(),
                    level: "medium".to_string(),
                    magnitude_percent: 61,
                    applies_to: Some(TraceLineageNodeRef {
                        kind: TraceLineageNodeKind::ModelCall,
                        id: "model-call:planner-1".to_string(),
                        label: "planner call".to_string(),
                    }),
                    contributions: vec![TraceSignalContribution {
                        source: "operator_memory".to_string(),
                        share_percent: 100,
                        rationale: "test".to_string(),
                    }],
                    artifact: ArtifactEnvelope::text(
                        TraceArtifactId::new("artifact-3").expect("artifact"),
                        ArtifactKind::Checkpoint,
                        "signal",
                        "{\"kind\":\"context_strain\"}",
                        usize::MAX,
                    ),
                }),
            }],
        };

        let projection = ConversationManifoldProjection::from_trace_replay(&replay);
        let frame = &projection.turn(&turn_id).expect("turn projection").frames[0];

        for primitive in &frame.primitives {
            assert!(
                primitive.evidence_record_id.is_some()
                    || matches!(
                        primitive.basis,
                        ManifoldPrimitiveBasis::LineageAnchor { .. }
                    ),
                "primitive lacked evidence or lineage basis: {primitive:?}"
            );
        }

        for conduit in &frame.conduits {
            assert!(
                conduit.evidence_record_id.is_some()
                    || matches!(conduit.basis, ManifoldPrimitiveBasis::LineageAnchor { .. }),
                "conduit lacked evidence or lineage basis: {conduit:?}"
            );
        }
    }
}
