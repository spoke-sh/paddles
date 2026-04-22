use super::*;

pub(super) struct ConversationReadModelChamber<'a> {
    service: &'a MechSuitService,
}

impl<'a> ConversationReadModelChamber<'a> {
    pub(super) const fn new(service: &'a MechSuitService) -> Self {
        Self { service }
    }

    pub(super) fn emit_transcript_update(&self, task_id: &TaskTraceId) {
        let update = ConversationTranscriptUpdate {
            task_id: task_id.clone(),
        };
        let observers = self
            .service
            .transcript_observers
            .lock()
            .expect("transcript observers lock")
            .clone();
        for observer in observers {
            observer.emit(update.clone());
        }
    }

    pub(super) fn replay_for_known_session(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<Option<crate::domain::model::TraceReplay>> {
        match self.service.trace_recorder.replay(task_id) {
            Ok(replay) => Ok(Some(replay)),
            Err(err) => {
                let known_session = self
                    .service
                    .sessions
                    .lock()
                    .expect("conversation sessions lock")
                    .contains_key(task_id.as_str());
                if known_session { Ok(None) } else { Err(err) }
            }
        }
    }

    pub(super) fn replay_conversation_forensics(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationForensicProjection> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationForensicProjection::from_trace_replay(&replay)),
            None => Ok(ConversationForensicProjection {
                task_id: task_id.clone(),
                turns: Vec::new(),
            }),
        }
    }

    pub(super) fn replay_turn_forensics(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<ForensicTurnProjection>> {
        Ok(self.replay_conversation_forensics(task_id)?.turn(turn_id))
    }

    pub(super) fn replay_conversation_manifold(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationManifoldProjection> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationManifoldProjection::from_trace_replay(&replay)),
            None => Ok(ConversationManifoldProjection {
                task_id: task_id.clone(),
                turns: Vec::new(),
            }),
        }
    }

    pub(super) fn replay_turn_manifold(
        &self,
        task_id: &TaskTraceId,
        turn_id: &TurnTraceId,
    ) -> Result<Option<ManifoldTurnProjection>> {
        Ok(self.replay_conversation_manifold(task_id)?.turn(turn_id))
    }

    pub(super) fn replay_conversation_transcript(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationTranscript> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationTranscript::from_trace_replay(&replay)),
            None => Ok(ConversationTranscript {
                task_id: task_id.clone(),
                entries: Vec::new(),
            }),
        }
    }

    pub(super) fn replay_conversation_trace_graph(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationTraceGraph> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationTraceGraph::from_trace_replay(&replay)),
            None => Ok(ConversationTraceGraph::empty(task_id.clone())),
        }
    }

    pub(super) fn replay_conversation_delegation(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<crate::domain::model::ConversationDelegationProjection> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(
                crate::domain::model::ConversationDelegationProjection::from_trace_replay(&replay),
            ),
            None => {
                Ok(crate::domain::model::ConversationDelegationProjection::empty(task_id.clone()))
            }
        }
    }

    pub(super) fn replay_conversation_projection(
        &self,
        task_id: &TaskTraceId,
    ) -> Result<ConversationProjectionSnapshot> {
        match self.replay_for_known_session(task_id)? {
            Some(replay) => Ok(ConversationProjectionSnapshot::from_trace_replay(&replay)),
            None => Ok(ConversationProjectionSnapshot::empty(task_id.clone())),
        }
    }

    pub(super) fn projection_update_for_transcript(
        &self,
        update: &ConversationTranscriptUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        let snapshot = self.replay_conversation_projection(&update.task_id)?;
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
        &self,
        update: &ConversationForensicUpdate,
    ) -> Result<ConversationProjectionUpdate> {
        let snapshot = self.replay_conversation_projection(&update.task_id)?;
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
}
