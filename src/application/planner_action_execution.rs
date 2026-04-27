use super::trim_for_planner;
use crate::domain::model::{ExecutionGovernanceOutcome, ExecutionPermissionRequest, TurnEventSink};
use crate::domain::ports::{PlannerAction, WorkspaceAction};
use crate::infrastructure::execution_governance::{
    GovernedTerminalCommandResult, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::terminal::run_background_terminal_command_with_execution_hand_registry;
use anyhow::Result;
use std::path::Path;
use std::sync::Arc;

pub(super) struct GovernedPlannerCommandSummary {
    pub summary: String,
    pub command_succeeded: bool,
    pub governance_request: ExecutionPermissionRequest,
    pub governance_outcome: ExecutionGovernanceOutcome,
}

pub(super) fn run_planner_inspect_command(
    workspace_root: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<GovernedPlannerCommandSummary> {
    validate_inspect_command(command)?;
    let output = run_background_terminal_command_with_execution_hand_registry(
        workspace_root,
        command,
        "inspect",
        call_id,
        event_sink,
        execution_hand_registry,
    )?;
    match output {
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

            Ok(GovernedPlannerCommandSummary {
                summary: trim_for_planner(&rendered, 1_200),
                command_succeeded: output.status.success(),
                governance_request,
                governance_outcome,
            })
        }
        GovernedTerminalCommandResult::Blocked {
            governance_request,
            governance_outcome,
        } => Ok(GovernedPlannerCommandSummary {
            summary: summarize_governance_outcome(&governance_outcome),
            command_succeeded: false,
            governance_request,
            governance_outcome,
        }),
    }
}

pub(super) fn run_planner_shell_command(
    workspace_root: &Path,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    command: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<GovernedPlannerCommandSummary> {
    let output = run_background_terminal_command_with_execution_hand_registry(
        workspace_root,
        command,
        "shell",
        call_id,
        event_sink,
        execution_hand_registry,
    )?;
    match output {
        GovernedTerminalCommandResult::Executed {
            output,
            governance_request,
            governance_outcome,
        } => {
            let summary = format_command_output_summary(command, &output);
            Ok(GovernedPlannerCommandSummary {
                summary,
                command_succeeded: output.status.success(),
                governance_request,
                governance_outcome,
            })
        }
        GovernedTerminalCommandResult::Blocked {
            governance_request,
            governance_outcome,
        } => Ok(GovernedPlannerCommandSummary {
            summary: summarize_governance_outcome(&governance_outcome),
            command_succeeded: false,
            governance_request,
            governance_outcome,
        }),
    }
}

pub(super) fn validate_inspect_command(command: &str) -> Result<()> {
    let normalized = command.trim();
    if normalized.is_empty() {
        anyhow::bail!("planner inspect command must not be empty");
    }
    if normalized.contains("&&")
        || normalized.contains("||")
        || normalized.contains(';')
        || normalized.contains('>')
        || normalized.contains('<')
    {
        anyhow::bail!(
            "planner inspect command must be a single read-only probe without chaining or redirection"
        );
    }
    Ok(())
}

pub(super) fn planner_terminal_tool_success_summary(tool_name: &str, output: &str) -> String {
    let had_output = !output.trim().is_empty();
    match (tool_name, had_output) {
        ("inspect", true) => "inspection completed".to_string(),
        ("inspect", false) => "inspection completed with no output".to_string(),
        ("shell", true) => "command completed".to_string(),
        ("shell", false) => "command completed with no output".to_string(),
        (_, true) => "tool completed".to_string(),
        (_, false) => "tool completed with no output".to_string(),
    }
}

pub(super) fn planner_action_query(action: &PlannerAction) -> Option<String> {
    match action {
        PlannerAction::Workspace { action } => match action {
            WorkspaceAction::Search { query, .. } => Some(query.clone()),
            WorkspaceAction::ListFiles { pattern } => pattern.clone(),
            WorkspaceAction::Read { path } => Some(path.clone()),
            WorkspaceAction::Inspect { command } => Some(command.clone()),
            WorkspaceAction::Shell { command } => Some(command.clone()),
            WorkspaceAction::Diff { path } => path
                .clone()
                .or_else(|| Some("git diff --no-ext-diff".to_string())),
            WorkspaceAction::WriteFile { path, .. } => Some(path.clone()),
            WorkspaceAction::ReplaceInFile { path, .. } => Some(path.clone()),
            WorkspaceAction::ApplyPatch { .. } => {
                Some("git apply --whitespace=nowarn -".to_string())
            }
            WorkspaceAction::SemanticDefinitions { path, .. }
            | WorkspaceAction::SemanticReferences { path, .. }
            | WorkspaceAction::SemanticSymbols { path }
            | WorkspaceAction::SemanticHover { path, .. } => Some(path.clone()),
            WorkspaceAction::SemanticDiagnostics { path } => path
                .clone()
                .or_else(|| Some("workspace diagnostics".to_string())),
            WorkspaceAction::ExternalCapability { invocation } => Some(invocation.summary()),
        },
        PlannerAction::Refine { query, .. } => Some(query.clone()),
        PlannerAction::Branch { branches, .. } => Some(branches.join(" | ")),
        PlannerAction::Stop { reason } => Some(reason.clone()),
    }
}

pub(super) fn workspace_action_evidence_source(action: &WorkspaceAction) -> String {
    match action {
        WorkspaceAction::Search { query, .. } => format!("search: {query}"),
        WorkspaceAction::ListFiles { pattern } => match pattern {
            Some(pattern) => format!("list_files: {pattern}"),
            None => "list_files".to_string(),
        },
        WorkspaceAction::Read { path } => path.clone(),
        WorkspaceAction::Inspect { command } => format!("command: {command}"),
        WorkspaceAction::Shell { command } => format!("command: {command}"),
        WorkspaceAction::Diff { path } => match path {
            Some(path) => format!("diff: {path}"),
            None => "git diff --no-ext-diff".to_string(),
        },
        WorkspaceAction::WriteFile { path, .. } => path.clone(),
        WorkspaceAction::ReplaceInFile { path, .. } => path.clone(),
        WorkspaceAction::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
        WorkspaceAction::SemanticDefinitions { path, .. } => {
            format!("semantic_definitions: {path}")
        }
        WorkspaceAction::SemanticReferences { path, .. } => {
            format!("semantic_references: {path}")
        }
        WorkspaceAction::SemanticSymbols { path } => format!("semantic_symbols: {path}"),
        WorkspaceAction::SemanticHover { path, .. } => format!("semantic_hover: {path}"),
        WorkspaceAction::SemanticDiagnostics { path } => match path {
            Some(path) => format!("semantic_diagnostics: {path}"),
            None => "semantic_diagnostics".to_string(),
        },
        WorkspaceAction::ExternalCapability { invocation } => {
            format!("external_capability:{}", invocation.capability_id)
        }
    }
}

fn format_command_output_summary(command: &str, output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let rendered = if stderr.trim().is_empty() {
        stdout
    } else if stdout.trim().is_empty() {
        stderr
    } else {
        format!("{stdout}\n{stderr}")
    };
    let status = output
        .status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| output.status.to_string());

    trim_for_planner(
        &format!(
            "Shell command: {command}\nExit status: {status}\n{}",
            rendered.trim()
        ),
        1_200,
    )
}
