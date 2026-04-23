use super::planning::WorkspaceAction;
use crate::domain::model::{
    AppliedEdit, ExecutionGovernanceOutcome, ExecutionPermissionRequest, TurnEventSink,
};
use anyhow::Result;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WorkspaceCapabilitySurface {
    pub actions: Vec<WorkspaceActionCapability>,
    pub tools: Vec<WorkspaceToolCapability>,
    pub notes: Vec<String>,
}

impl WorkspaceCapabilitySurface {
    pub fn has_tool(&self, tool: &str) -> bool {
        self.tools.iter().any(|candidate| candidate.tool == tool)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceActionCapability {
    pub action: String,
    pub summary: String,
    pub mutating: bool,
}

impl WorkspaceActionCapability {
    pub fn new(action: impl Into<String>, summary: impl Into<String>, mutating: bool) -> Self {
        Self {
            action: action.into(),
            summary: summary.into(),
            mutating,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceToolCapability {
    pub tool: String,
    pub summary: String,
    pub suggested_probe: Option<WorkspaceAction>,
}

impl WorkspaceToolCapability {
    pub fn new(
        tool: impl Into<String>,
        summary: impl Into<String>,
        suggested_probe: Option<WorkspaceAction>,
    ) -> Self {
        Self {
            tool: tool.into(),
            summary: summary.into(),
            suggested_probe,
        }
    }
}

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
    fn capability_surface(&self) -> WorkspaceCapabilitySurface {
        WorkspaceCapabilitySurface::default()
    }

    fn execute_workspace_action(
        &self,
        action: &WorkspaceAction,
        frame: WorkspaceActionExecutionFrame<'_>,
    ) -> Result<WorkspaceActionResult>;
}
