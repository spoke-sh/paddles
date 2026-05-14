//! Context assembly phase: derives the per-turn `InterpretationContext` and
//! optionally bootstraps a known-edit initial planner action. Free functions
//! here replace the prior `InterpretationChamber` wrapper struct — they take
//! the `AgentRuntime` explicitly so call sites read as plain phase
//! invocations rather than method chains through a stateless wrapper.

use super::*;

pub(super) async fn derive_interpretation_context(
    service: &AgentRuntime,
    prompt: &str,
    planner: &dyn ActionSelectionEngine,
    event_sink: Arc<dyn TurnEventSink>,
) -> InterpretationContext {
    service
        .derive_interpretation_context(prompt, planner, event_sink)
        .await
}

pub(super) async fn bootstrap_known_edit_initial_action(
    service: &AgentRuntime,
    prompt: &str,
    interpretation: &InterpretationContext,
    recent_turns: &[String],
    retrieval_provider: Option<&Arc<dyn RetrievalProvider>>,
    decision: &InitialActionDecision,
    trace: &StructuredTurnTrace,
) -> Result<Option<InitialActionDecision>> {
    service
        .bootstrap_known_edit_initial_action(
            prompt,
            interpretation,
            recent_turns,
            retrieval_provider,
            decision,
            trace,
        )
        .await
}
