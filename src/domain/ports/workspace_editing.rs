use super::{planning::WorkspaceAction, workspace_action_execution::WorkspaceActionResult};
use anyhow::Result;

pub trait WorkspaceEditor: Send + Sync {
    fn diff(&self, path: Option<&str>) -> Result<WorkspaceActionResult>;

    fn write_file(&self, path: &str, content: &str) -> Result<WorkspaceActionResult>;

    fn replace_in_file(
        &self,
        path: &str,
        old: &str,
        new: &str,
        replace_all: bool,
    ) -> Result<WorkspaceActionResult>;

    fn apply_patch(&self, patch: &str) -> Result<WorkspaceActionResult>;

    fn execute_action(&self, action: &WorkspaceAction) -> Result<Option<WorkspaceActionResult>> {
        match action {
            WorkspaceAction::Diff { path } => self.diff(path.as_deref()).map(Some),
            WorkspaceAction::WriteFile { path, content } => {
                self.write_file(path, content).map(Some)
            }
            WorkspaceAction::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => self.replace_in_file(path, old, new, *replace_all).map(Some),
            WorkspaceAction::ApplyPatch { patch } => self.apply_patch(patch).map(Some),
            _ => Ok(None),
        }
    }
}
