use super::*;

pub(super) struct InterpretationChamber<'a> {
    service: &'a AgentRuntime,
}

impl<'a> InterpretationChamber<'a> {
    pub(super) const fn new(service: &'a AgentRuntime) -> Self {
        Self { service }
    }

    pub(super) async fn derive_interpretation_context(
        &self,
        prompt: &str,
        planner: &dyn RecursivePlanner,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> InterpretationContext {
        self.service
            .derive_interpretation_context(prompt, planner, event_sink)
            .await
    }

    pub(super) async fn bootstrap_known_edit_initial_action(
        &self,
        prompt: &str,
        interpretation: &InterpretationContext,
        recent_turns: &[String],
        gatherer: Option<&Arc<dyn ContextGatherer>>,
        decision: &InitialActionDecision,
        trace: &StructuredTurnTrace,
    ) -> Result<Option<InitialActionDecision>> {
        self.service
            .bootstrap_known_edit_initial_action(
                prompt,
                interpretation,
                recent_turns,
                gatherer,
                decision,
                trace,
            )
            .await
    }
}
