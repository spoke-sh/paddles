use super::planning::WorkspaceAction;
use crate::domain::model::{
    AppliedEdit, ExecutionGovernanceOutcome, ExecutionPermissionRequest, TurnEventSink,
};
use anyhow::Result;

#[derive(Debug)]
pub struct WorkspaceActionResult {
    pub name: String,
    pub summary: String,
    pub applied_edit: Option<AppliedEdit>,
    pub governance_request: Option<ExecutionPermissionRequest>,
    pub governance_outcome: Option<ExecutionGovernanceOutcome>,
}

pub struct WorkspaceActionExecutionFrame<'a> {
    pub call_id: &'a str,
    pub event_sink: &'a dyn TurnEventSink,
}

pub trait WorkspaceActionExecutor: Send + Sync {
    fn execute_workspace_action(
        &self,
        action: &WorkspaceAction,
        frame: WorkspaceActionExecutionFrame<'_>,
    ) -> Result<WorkspaceActionResult>;
}
