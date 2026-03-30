use crate::domain::model::ThreadDecision;
use crate::domain::ports::{
    InitialActionDecision, InterpretationContext, InterpretationRequest, PlannerCapability,
    PlannerRequest, RecursivePlanner, RecursivePlannerDecision, ThreadDecisionRequest,
};
use crate::infrastructure::adapters::sift_agent::SiftAgentAdapter;
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;

pub struct SiftPlannerAdapter {
    engine: Arc<SiftAgentAdapter>,
}

impl SiftPlannerAdapter {
    pub fn new(engine: Arc<SiftAgentAdapter>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl RecursivePlanner for SiftPlannerAdapter {
    fn capability(&self) -> PlannerCapability {
        PlannerCapability::Available
    }

    async fn derive_interpretation_context(
        &self,
        request: &InterpretationRequest,
    ) -> Result<InterpretationContext, anyhow::Error> {
        self.engine.derive_interpretation_context(request)
    }

    async fn select_initial_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<InitialActionDecision, anyhow::Error> {
        self.engine.select_initial_action(request)
    }

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<RecursivePlannerDecision, anyhow::Error> {
        self.engine.select_planner_action(request)
    }

    async fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
    ) -> Result<ThreadDecision, anyhow::Error> {
        self.engine.select_thread_decision(request)
    }
}
