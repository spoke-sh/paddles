//! Synthesis phase: gathers recent-turn summaries, derives specialist
//! runtime notes, and finalizes the authored response for the operator.
//! Free functions here replace the prior `SynthesisChamber` wrapper struct.

use super::*;

pub(super) fn recent_turn_summaries(
    service: &AgentRuntime,
    session: &ConversationSession,
    synthesizer_engine: &dyn SynthesizerEngine,
) -> Result<Vec<String>> {
    let session_slice = service.query_session_context_slice(
        &session.task_id(),
        TraceSessionContextQuery::AdaptiveReplay { turn_limit: 4 },
    )?;
    if !session_slice.turn_summaries.is_empty() {
        return Ok(session_slice.turn_summaries);
    }

    if let Some(store) = service.conversation_history_store() {
        let recent_turns = store.recent_turn_summaries()?;
        if !recent_turns.is_empty() {
            return Ok(recent_turns);
        }
    }

    synthesizer_engine.recent_turn_summaries()
}

pub(super) fn specialist_runtime_notes(
    service: &AgentRuntime,
    prompt: &str,
    session: &ConversationSession,
    prepared: &PreparedRuntimeLanes,
) -> Vec<String> {
    let Ok(session_context) = service.query_session_context_slice(
        &session.task_id(),
        TraceSessionContextQuery::AdaptiveReplay { turn_limit: 4 },
    ) else {
        return vec![
            "Specialist brains unavailable: adaptive session context could not be queried."
                .to_string(),
        ];
    };
    let profile = prepared.harness_profile();
    service.specialist_brain_registry().runtime_notes(
        &profile,
        &SpecialistBrainRequest {
            user_prompt: prompt.to_string(),
            workspace_root: service.workspace_root.clone(),
            active_profile_id: profile.active_profile_id().to_string(),
            session_context,
        },
    )
}

pub(super) fn finalize_turn_response(
    service: &AgentRuntime,
    trace: &StructuredTurnTrace,
    session: &ConversationSession,
    active_thread: &ConversationThreadRef,
    prompt: &str,
    response: &AuthoredResponse,
) -> String {
    let reply = response.to_plain_text();
    trace.record_completion(response);
    conversation_read_model::emit_transcript_update(service, &session.task_id());
    session.note_thread_reply(active_thread, prompt, &reply);
    service.persist_recent_turn_summary(prompt, &reply);
    reply
}
