//! Context assembly phase: derives the per-turn `InterpretationContext` and
//! prepares turn interpretation. Free functions here replace the prior
//! `InterpretationChamber` wrapper struct — they take the `AgentRuntime`
//! explicitly so call sites read as plain phase invocations rather than method
//! chains through a stateless wrapper.

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
