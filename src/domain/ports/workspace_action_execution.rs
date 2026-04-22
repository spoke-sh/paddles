use super::{planning::WorkspaceAction, synthesis::WorkspaceActionResult};
use crate::domain::model::TurnEventSink;
use anyhow::Result;

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
