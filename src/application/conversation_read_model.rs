//! Conversation read-model phase: emits transcript-update notifications and
//! projects persisted trace replays into the forensic, manifold, transcript,
//! trace-graph, delegation, and projection-snapshot read models. Free
//! functions here replace the prior `ConversationReadModelChamber` wrapper
//! struct.

use super::*;

pub(super) fn emit_transcript_update(service: &AgentRuntime, task_id: &TaskTraceId) {
    let update = ConversationTranscriptUpdate {
        task_id: task_id.clone(),
    };
    let observers = service
        .transcript_observers
        .lock()
        .expect("transcript observers lock")
        .clone();
    for observer in observers {
        observer.emit(update.clone());
    }
}

pub(super) fn replay_for_known_session(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<Option<crate::domain::model::TraceReplay>> {
    match service.trace_recorder.replay(task_id) {
        Ok(replay) => Ok(Some(replay)),
        Err(err) => {
            let known_session = service
                .sessions
                .lock()
                .expect("conversation sessions lock")
                .contains_key(task_id.as_str());
            if known_session { Ok(None) } else { Err(err) }
        }
    }
}

pub(super) fn replay_conversation_forensics(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<ConversationForensicProjection> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => Ok(ConversationForensicProjection::from_trace_replay(&replay)),
        None => Ok(ConversationForensicProjection {
            task_id: task_id.clone(),
            turns: Vec::new(),
        }),
    }
}

pub(super) fn replay_turn_forensics(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
    turn_id: &TurnTraceId,
) -> Result<Option<ForensicTurnProjection>> {
    Ok(replay_conversation_forensics(service, task_id)?.turn(turn_id))
}

pub(super) fn replay_conversation_manifold(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<ConversationManifoldProjection> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => Ok(ConversationManifoldProjection::from_trace_replay(&replay)),
        None => Ok(ConversationManifoldProjection {
            task_id: task_id.clone(),
            turns: Vec::new(),
        }),
    }
}

pub(super) fn replay_turn_manifold(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
    turn_id: &TurnTraceId,
) -> Result<Option<ManifoldTurnProjection>> {
    Ok(replay_conversation_manifold(service, task_id)?.turn(turn_id))
}

pub(super) fn replay_conversation_transcript(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<ConversationTranscript> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => Ok(ConversationTranscript::from_trace_replay(&replay)),
        None => Ok(ConversationTranscript {
            task_id: task_id.clone(),
            entries: Vec::new(),
        }),
    }
}

pub(super) fn replay_conversation_trace_graph(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<ConversationTraceGraph> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => Ok(ConversationTraceGraph::from_trace_replay(&replay)),
        None => Ok(ConversationTraceGraph::empty(task_id.clone())),
    }
}

pub(super) fn replay_conversation_delegation(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<crate::domain::model::ConversationDelegationProjection> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => {
            Ok(crate::domain::model::ConversationDelegationProjection::from_trace_replay(&replay))
        }
        None => Ok(crate::domain::model::ConversationDelegationProjection::empty(task_id.clone())),
    }
}

pub(super) fn replay_conversation_projection(
    service: &AgentRuntime,
    task_id: &TaskTraceId,
) -> Result<ConversationProjectionSnapshot> {
    match replay_for_known_session(service, task_id)? {
        Some(replay) => Ok(ConversationProjectionSnapshot::from_trace_replay(&replay)),
        None => Ok(ConversationProjectionSnapshot::empty(task_id.clone())),
    }
}

pub(super) fn projection_update_for_transcript(
    service: &AgentRuntime,
    update: &ConversationTranscriptUpdate,
) -> Result<ConversationProjectionUpdate> {
    let snapshot = replay_conversation_projection(service, &update.task_id)?;
    Ok(ConversationProjectionUpdate {
        task_id: update.task_id.clone(),
        kind: ConversationProjectionUpdateKind::Transcript,
        reducer: ConversationProjectionReducer::ReplaceSnapshot,
        version: snapshot.version(),
        transcript_update: Some(update.clone()),
        forensic_update: None,
        snapshot,
    })
}

pub(super) fn projection_update_for_forensic(
    service: &AgentRuntime,
    update: &ConversationForensicUpdate,
) -> Result<ConversationProjectionUpdate> {
    let snapshot = replay_conversation_projection(service, &update.task_id)?;
    Ok(ConversationProjectionUpdate {
        task_id: update.task_id.clone(),
        kind: ConversationProjectionUpdateKind::Forensic,
        reducer: ConversationProjectionReducer::ReplaceSnapshot,
        version: snapshot.version(),
        transcript_update: None,
        forensic_update: Some(update.clone()),
        snapshot,
    })
}
