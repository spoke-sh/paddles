use super::trim_for_planner_head_tail;

/// Maximum characters the planner-bound shell/inspect summary may carry.
/// Generous on purpose: at 32k chars a typical `cargo build` failure or
/// `pytest` traceback fits without losing the head or the tail. The
/// operator-visible TUI stream is already streamed live and capped
/// independently in `infrastructure::terminal`.
const PLANNER_TOOL_SUMMARY_CHAR_BUDGET: usize = 32_000;
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
                summary: trim_for_planner_head_tail(&rendered, PLANNER_TOOL_SUMMARY_CHAR_BUDGET),
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

    trim_for_planner_head_tail(
        &format!(
            "Shell command: {command}\nExit status: {status}\n{}",
            rendered.trim()
        ),
        PLANNER_TOOL_SUMMARY_CHAR_BUDGET,
    )
}

#[cfg(test)]
mod tests {
    use super::{
        PLANNER_TOOL_SUMMARY_CHAR_BUDGET, run_planner_inspect_command, run_planner_shell_command,
    };
    use crate::domain::model::{TurnEvent, TurnEventSink};
    use crate::infrastructure::execution_governance::GovernedTerminalCommandResult;
    use crate::infrastructure::execution_hand::ExecutionHandRegistry;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingTurnEventSink {
        events: Mutex<Vec<TurnEvent>>,
    }

    impl RecordingTurnEventSink {
        fn recorded(&self) -> Vec<TurnEvent> {
            self.events.lock().expect("turn events lock").clone()
        }
    }

    impl TurnEventSink for RecordingTurnEventSink {
        fn emit(&self, event: TurnEvent) {
            self.events.lock().expect("turn events lock").push(event);
        }
    }

    #[test]
    fn planner_shell_summary_preserves_output_well_beyond_old_1200_char_cap() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::with_default_local_governance());
        let sink = RecordingTurnEventSink::default();

        // Generate ~16 KiB of output — comfortably above the old 1,200-char
        // cap that used to silently slice cargo build / pytest tracebacks.
        let summary = run_planner_shell_command(
            workspace.path(),
            Arc::clone(&registry),
            "for i in $(seq 1 800); do printf 'log line %04d -- some context here\\n' \"$i\"; done",
            "call-shell-large",
            &sink,
        )
        .expect("shell command should run");

        assert!(summary.command_succeeded);
        let body = &summary.summary;
        // Both the head and the tail of the run must be visible to the planner.
        assert!(
            body.contains("log line 0001"),
            "head of output should survive in planner-bound summary"
        );
        assert!(
            body.contains("log line 0800"),
            "tail of output should survive in planner-bound summary"
        );
        // Far above the old 1,200-char ceiling.
        assert!(
            body.chars().count() > 4_000,
            "summary should carry well over 1,200 chars (got {})",
            body.chars().count()
        );
        // Operator-visible streaming chunks reach the trace as the command
        // runs, not just as a single end-of-command payload.
        let chunk_count = sink
            .recorded()
            .iter()
            .filter(|event| matches!(event, TurnEvent::ToolOutput { .. }))
            .count();
        assert!(
            chunk_count > 1,
            "shell output should stream as multiple ToolOutput chunks (got {chunk_count})"
        );
    }

    #[test]
    fn planner_shell_summary_uses_head_tail_truncation_above_budget() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::with_default_local_governance());
        let sink = RecordingTurnEventSink::default();

        // Generate output that comfortably exceeds the planner-bound budget so
        // we exercise the head+tail truncation path.
        let line_count = (PLANNER_TOOL_SUMMARY_CHAR_BUDGET / 32) + 2_000;
        let command = format!(
            "for i in $(seq 1 {line_count}); do printf 'verbose log line %06d xxxxxxxxxxxxxxxx\\n' \"$i\"; done"
        );

        let summary = run_planner_shell_command(
            workspace.path(),
            Arc::clone(&registry),
            &command,
            "call-shell-overflow",
            &sink,
        )
        .expect("shell command should run");

        let body = &summary.summary;
        assert!(
            body.contains("…[truncated"),
            "head+tail truncation marker should appear in the summary"
        );
        assert!(
            body.contains("verbose log line 000001"),
            "head should survive head+tail truncation"
        );
        assert!(
            body.contains(&format!("verbose log line {line_count:06}")),
            "tail should survive head+tail truncation"
        );
    }

    #[test]
    fn planner_inspect_summary_preserves_output_well_beyond_old_1200_char_cap() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::with_default_local_governance());
        let sink = RecordingTurnEventSink::default();

        let summary = run_planner_inspect_command(
            workspace.path(),
            Arc::clone(&registry),
            "seq 1 600",
            "call-inspect-large",
            &sink,
        )
        .expect("inspect command should run");

        assert!(summary.command_succeeded);
        let body = &summary.summary;
        assert!(
            body.starts_with("1\n2\n3\n"),
            "head should survive: {body:?}"
        );
        assert!(body.ends_with("600"), "tail should survive: {body:?}");
        assert!(
            body.chars().count() > 1_500,
            "summary should carry well over 1,200 chars (got {})",
            body.chars().count()
        );
    }

    #[test]
    fn planner_blocked_command_does_not_emit_streaming_chunks() {
        let workspace = tempfile::tempdir().expect("workspace");
        // No governance profile installed -> command is denied before exec.
        let registry = Arc::new(ExecutionHandRegistry::default());
        let sink = RecordingTurnEventSink::default();

        let summary = run_planner_shell_command(
            workspace.path(),
            Arc::clone(&registry),
            "echo blocked",
            "call-shell-blocked",
            &sink,
        )
        .expect("shell command result");

        assert!(!summary.command_succeeded);
        assert!(matches!(
            summary.governance_outcome.kind,
            crate::domain::model::ExecutionGovernanceOutcomeKind::PolicyUnavailable
        ));
        let _ = GovernedTerminalCommandResult::Blocked {
            governance_request: summary.governance_request.clone(),
            governance_outcome: summary.governance_outcome.clone(),
        };
        assert!(
            sink.recorded()
                .iter()
                .all(|event| !matches!(event, TurnEvent::ToolOutput { .. })),
            "blocked command should not stream tool output"
        );
    }
}
