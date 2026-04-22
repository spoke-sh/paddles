use super::*;

pub(super) struct SynthesisChamber<'a> {
    service: &'a MechSuitService,
}

impl<'a> SynthesisChamber<'a> {
    pub(super) const fn new(service: &'a MechSuitService) -> Self {
        Self { service }
    }

    pub(super) fn recent_turn_summaries(
        &self,
        session: &ConversationSession,
        synthesizer_engine: &dyn SynthesizerEngine,
    ) -> Result<Vec<String>> {
        let session_slice = self.service.query_session_context_slice(
            &session.task_id(),
            TraceSessionContextQuery::AdaptiveReplay { turn_limit: 4 },
        )?;
        if !session_slice.turn_summaries.is_empty() {
            return Ok(session_slice.turn_summaries);
        }

        if let Some(store) = self.service.conversation_history_store() {
            let recent_turns = store.recent_turn_summaries()?;
            if !recent_turns.is_empty() {
                return Ok(recent_turns);
            }
        }

        synthesizer_engine.recent_turn_summaries()
    }

    pub(super) fn specialist_runtime_notes(
        &self,
        prompt: &str,
        session: &ConversationSession,
        prepared: &PreparedRuntimeLanes,
    ) -> Vec<String> {
        let Ok(session_context) = self.service.query_session_context_slice(
            &session.task_id(),
            TraceSessionContextQuery::AdaptiveReplay { turn_limit: 4 },
        ) else {
            return vec![
                "Specialist brains unavailable: adaptive session context could not be queried."
                    .to_string(),
            ];
        };
        let profile = prepared.harness_profile();
        self.service.specialist_brain_registry().runtime_notes(
            &profile,
            &SpecialistBrainRequest {
                user_prompt: prompt.to_string(),
                workspace_root: self.service.workspace_root.clone(),
                active_profile_id: profile.active_profile_id().to_string(),
                session_context,
            },
        )
    }

    pub(super) fn finalize_turn_response(
        &self,
        trace: &StructuredTurnTrace,
        session: &ConversationSession,
        active_thread: &ConversationThreadRef,
        prompt: &str,
        response: &AuthoredResponse,
    ) -> String {
        let reply = response.to_plain_text();
        trace.record_completion(response);
        self.service
            .conversation_read_model()
            .emit_transcript_update(&session.task_id());
        session.note_thread_reply(active_thread, prompt, &reply);
        self.service.persist_recent_turn_summary(prompt, &reply);
        reply
    }
}
