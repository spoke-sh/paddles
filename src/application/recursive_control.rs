use super::*;

pub(super) struct RecursiveControlChamber<'a> {
    service: &'a MechSuitService,
}

impl<'a> RecursiveControlChamber<'a> {
    pub(super) const fn new(service: &'a MechSuitService) -> Self {
        Self { service }
    }

    pub(super) async fn execute_recursive_planner_loop(
        &self,
        prompt: &str,
        context: PlannerLoopContext,
        initial_decision: Option<RecursivePlannerDecision>,
        execution_checklist: Option<ExecutionChecklistState>,
        trace: Arc<StructuredTurnTrace>,
    ) -> Result<PlannerLoopOutcome> {
        self.service
            .execute_recursive_planner_loop(
                prompt,
                context,
                initial_decision,
                execution_checklist,
                trace,
            )
            .await
    }

    pub(super) fn expire_turn_control_requests(
        &self,
        trace: &Arc<StructuredTurnTrace>,
        detail: &str,
    ) {
        self.service.expire_turn_control_requests(trace, detail);
    }
}
