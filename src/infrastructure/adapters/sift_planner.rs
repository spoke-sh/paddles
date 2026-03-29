use crate::domain::ports::{
    InitialActionDecision, PlannerCapability, PlannerRequest, RecursivePlanner,
    RecursivePlannerDecision,
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
}
