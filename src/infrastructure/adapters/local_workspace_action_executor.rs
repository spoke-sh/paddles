use super::local_workspace_editor::{LocalWorkspaceEditor, relative_path, resolve_workspace_path};
use crate::domain::ports::{
    WorkspaceAction, WorkspaceActionExecutionFrame, WorkspaceActionExecutor, WorkspaceActionResult,
    WorkspaceEditor,
};
use crate::infrastructure::execution_governance::{
    GovernedTerminalCommandResult, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::terminal::run_background_terminal_command_with_runtime_mediator;
use crate::infrastructure::transport_mediator::TransportToolMediator;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

const MAX_TOOL_OUTPUT_CHARS: usize = 12_000;
const MAX_FILE_CHARS: usize = 16_000;
const MAX_LISTED_FILES: usize = 200;

#[derive(Clone, Debug)]
pub struct LocalWorkspaceActionExecutor {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
}

impl LocalWorkspaceActionExecutor {
    pub fn with_execution_hand_registry(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
    ) -> Self {
        let transport_mediator = Arc::new(TransportToolMediator::with_execution_hand_registry(
            Arc::clone(&execution_hand_registry),
        ));
        Self::with_runtime_mediator(workspace_root, execution_hand_registry, transport_mediator)
    }

    pub fn with_runtime_mediator(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        transport_mediator: Arc<TransportToolMediator>,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            execution_hand_registry,
            transport_mediator,
        }
    }

    fn workspace_editor(&self) -> LocalWorkspaceEditor {
        LocalWorkspaceEditor::with_runtime_mediator(
            self.workspace_root.clone(),
            Arc::clone(&self.execution_hand_registry),
            Arc::clone(&self.transport_mediator),
        )
    }

    fn execute_local_action(
        &self,
        action: &WorkspaceAction,
        frame: WorkspaceActionExecutionFrame<'_>,
    ) -> Result<WorkspaceActionResult> {
        match action {
            WorkspaceAction::ListFiles { pattern } => {
                let files = list_files(&self.workspace_root, pattern.as_deref())?;
                let summary = if files.is_empty() {
                    "No matching files found.".to_string()
                } else {
                    format!("Listed {} file(s):\n{}", files.len(), files.join("\n"))
                };
                Ok(WorkspaceActionResult {
                    name: "list_files".to_string(),
                    summary: trim_for_context(&summary, MAX_TOOL_OUTPUT_CHARS),
                    applied_edit: None,
                    governance_request: None,
                    governance_outcome: None,
                })
            }
            WorkspaceAction::Read { path } => {
                let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
                let content = fs::read(&resolved)
                    .with_context(|| format!("failed to read {}", resolved.display()))?;
                let content = String::from_utf8_lossy(&content).to_string();
                let rel = relative_path(&self.workspace_root, &resolved);
                Ok(WorkspaceActionResult {
                    name: "read_file".to_string(),
                    summary: format!(
                        "Read file {rel}:\n{}",
                        trim_for_context(&content, MAX_FILE_CHARS)
                    ),
                    applied_edit: None,
                    governance_request: None,
                    governance_outcome: None,
                })
            }
            WorkspaceAction::Inspect { command } => {
                validate_inspect_command(command)?;
                let output = run_background_terminal_command_with_runtime_mediator(
                    &self.workspace_root,
                    command,
                    "inspect",
                    frame.call_id,
                    frame.event_sink,
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .with_context(|| format!("failed to execute inspect command `{command}`"))?;
                let (summary, governance_request, governance_outcome) = match output {
                    GovernedTerminalCommandResult::Executed {
                        output,
                        governance_request,
                        governance_outcome,
                    } => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let rendered = if stderr.trim().is_empty() {
                            stdout
                        } else {
                            format!("{stdout}\n{stderr}")
                        };
                        (
                            trim_for_context(&rendered, MAX_TOOL_OUTPUT_CHARS),
                            Some(governance_request),
                            Some(governance_outcome),
                        )
                    }
                    GovernedTerminalCommandResult::Blocked {
                        governance_request,
                        governance_outcome,
                    } => (
                        summarize_governance_outcome(&governance_outcome),
                        Some(governance_request),
                        Some(governance_outcome),
                    ),
                };
                Ok(WorkspaceActionResult {
                    name: "inspect".to_string(),
                    summary,
                    applied_edit: None,
                    governance_request,
                    governance_outcome,
                })
            }
            WorkspaceAction::Diff { path } => self.workspace_editor().diff(path.as_deref()),
            WorkspaceAction::WriteFile { path, content } => {
                self.workspace_editor().write_file(path, content)
            }
            WorkspaceAction::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => self
                .workspace_editor()
                .replace_in_file(path, old, new, *replace_all),
            WorkspaceAction::ApplyPatch { patch } => self.workspace_editor().apply_patch(patch),
            WorkspaceAction::Shell { command } => {
                let output = run_background_terminal_command_with_runtime_mediator(
                    &self.workspace_root,
                    command,
                    "shell",
                    frame.call_id,
                    frame.event_sink,
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .with_context(|| format!("failed to execute shell command `{command}`"))?;
                let (summary, governance_request, governance_outcome) = match output {
                    GovernedTerminalCommandResult::Executed {
                        output,
                        governance_request,
                        governance_outcome,
                    } => {
                        let summary = format_command_summary("Shell command", command, &output);
                        if !output.status.success() {
                            bail!("{summary}");
                        }
                        (summary, Some(governance_request), Some(governance_outcome))
                    }
                    GovernedTerminalCommandResult::Blocked {
                        governance_request,
                        governance_outcome,
                    } => (
                        summarize_governance_outcome(&governance_outcome),
                        Some(governance_request),
                        Some(governance_outcome),
                    ),
                };
                Ok(WorkspaceActionResult {
                    name: "shell".to_string(),
                    summary,
                    applied_edit: None,
                    governance_request,
                    governance_outcome,
                })
            }
            WorkspaceAction::Search { .. } | WorkspaceAction::ExternalCapability { .. } => {
                Err(anyhow!(
                    "workspace action `{}` is not executable via the local workspace executor",
                    action.label()
                ))
            }
        }
    }
}

impl WorkspaceActionExecutor for LocalWorkspaceActionExecutor {
    fn execute_workspace_action(
        &self,
        action: &WorkspaceAction,
        frame: WorkspaceActionExecutionFrame<'_>,
    ) -> Result<WorkspaceActionResult> {
        self.execute_local_action(action, frame)
    }
}

fn list_files(workspace_root: &Path, pattern: Option<&str>) -> Result<Vec<String>> {
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let mut files = Vec::new();
    visit_files(
        workspace_root,
        workspace_root,
        &path_policy,
        pattern,
        &mut files,
    )?;
    files.sort();
    if files.len() > MAX_LISTED_FILES {
        files.truncate(MAX_LISTED_FILES);
    }
    Ok(files)
}

fn visit_files(
    dir: &Path,
    workspace_root: &Path,
    path_policy: &WorkspacePathPolicy,
    pattern: Option<&str>,
    files: &mut Vec<String>,
) -> Result<()> {
    if files.len() >= MAX_LISTED_FILES {
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            let Some(relative_dir) = path
                .strip_prefix(workspace_root)
                .ok()
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            else {
                continue;
            };
            if !path_policy.allows_relative_directory(&relative_dir) {
                continue;
            }
            visit_files(&path, workspace_root, path_policy, pattern, files)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let rel = relative_path(workspace_root, &path);
        if path_policy.allows_relative_file(&rel)
            && pattern.is_none_or(|needle| rel.contains(needle))
        {
            files.push(rel);
        }
        if files.len() >= MAX_LISTED_FILES {
            break;
        }
    }

    Ok(())
}

fn trim_for_context(input: &str, max_chars: usize) -> String {
    let mut trimmed = input.chars().take(max_chars).collect::<String>();
    if input.chars().count() > max_chars {
        trimmed.push_str("\n...[truncated]");
    }
    trimmed
}

fn format_command_summary(header: &str, command: &str, output: &std::process::Output) -> String {
    let mut summary = format!("{header}: {command}\nExit status: {}\n", output.status);

    if !output.stdout.is_empty() {
        summary.push_str("STDOUT:\n");
        summary.push_str(&String::from_utf8_lossy(&output.stdout));
        summary.push('\n');
    }

    if !output.stderr.is_empty() {
        summary.push_str("STDERR:\n");
        summary.push_str(&String::from_utf8_lossy(&output.stderr));
    }

    trim_for_context(&summary, MAX_TOOL_OUTPUT_CHARS)
}

fn validate_inspect_command(command: &str) -> Result<()> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        bail!("inspect command must not be empty");
    }
    if trimmed.contains('\n')
        || trimmed.contains("&&")
        || trimmed.contains("||")
        || trimmed.contains(';')
        || trimmed.contains('|')
        || trimmed.contains('>')
        || trimmed.contains('<')
    {
        bail!("inspect command must be a single read-only command");
    }
    Ok(())
}
