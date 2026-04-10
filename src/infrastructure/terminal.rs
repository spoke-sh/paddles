use crate::domain::model::{
    ExecutionHandDescriptor, ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation,
    ExecutionHandPhase, TurnEvent, TurnEventSink,
};
use crate::domain::ports::ExecutionHand;
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::sync::mpsc;

const TERMINAL_READ_BUFFER_BYTES: usize = 1024;
const MAX_TERMINAL_EVENT_CHARS: usize = 400;
const MAX_TERMINAL_EVENTS: usize = 24;
const MAX_CAPTURE_CHARS_PER_STREAM: usize = 24_000;

struct TerminalChunk {
    stream: &'static str,
    output: String,
}

#[allow(dead_code)]
pub(crate) fn run_background_terminal_command(
    workspace_root: &Path,
    command: &str,
    tool_name: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
) -> Result<Output> {
    BackgroundTerminalRunner::new(workspace_root).run(command, tool_name, call_id, event_sink)
}

pub(crate) fn run_background_terminal_command_with_execution_hand_registry(
    workspace_root: &Path,
    command: &str,
    tool_name: &str,
    call_id: &str,
    event_sink: &dyn TurnEventSink,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
) -> Result<Output> {
    BackgroundTerminalRunner::with_execution_hand_registry(workspace_root, execution_hand_registry)
        .run(command, tool_name, call_id, event_sink)
}

#[derive(Clone, Debug)]
struct BackgroundTerminalRunner {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
}

impl BackgroundTerminalRunner {
    #[allow(dead_code)]
    fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_execution_hand_registry(
            workspace_root,
            Arc::new(ExecutionHandRegistry::default()),
        )
    }

    fn with_execution_hand_registry(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            execution_hand_registry,
        }
    }

    fn descriptor() -> ExecutionHandDescriptor {
        ExecutionHandDescriptor::new(
            ExecutionHandKind::TerminalRunner,
            ExecutionHandKind::TerminalRunner.default_authority(),
            ExecutionHandKind::TerminalRunner.default_summary(),
            vec![
                ExecutionHandOperation::Describe,
                ExecutionHandOperation::Provision,
                ExecutionHandOperation::Execute,
                ExecutionHandOperation::Recover,
                ExecutionHandOperation::Degrade,
            ],
        )
    }

    fn current_diagnostic(&self) -> ExecutionHandDiagnostic {
        self.execution_hand_registry
            .diagnostic(ExecutionHandKind::TerminalRunner)
            .unwrap_or_else(|| ExecutionHandDiagnostic::from_descriptor(&Self::descriptor()))
    }

    fn record_execution_started(&self, command: &str) {
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::TerminalRunner,
            ExecutionHandPhase::Executing,
            ExecutionHandOperation::Execute,
            format!("terminal runner executing `{command}`"),
            None,
        );
    }

    fn record_execution_finished(&self, command: &str, last_error: Option<String>) {
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::TerminalRunner,
            ExecutionHandPhase::Ready,
            ExecutionHandOperation::Execute,
            format!("terminal runner completed `{command}`"),
            last_error,
        );
    }

    fn run(
        &self,
        command: &str,
        tool_name: &str,
        call_id: &str,
        event_sink: &dyn TurnEventSink,
    ) -> Result<Output> {
        self.record_execution_started(command);
        let mut child = match Command::new("sh")
            .arg("-lc")
            .arg(command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.workspace_root)
            .spawn()
        {
            Ok(child) => child,
            Err(error) => {
                self.execution_hand_registry.record_phase(
                    ExecutionHandKind::TerminalRunner,
                    ExecutionHandPhase::Failed,
                    ExecutionHandOperation::Execute,
                    format!("terminal runner failed to spawn `{command}`"),
                    Some(error.to_string()),
                );
                return Err(error)
                    .with_context(|| format!("failed to spawn shell command `{command}`"));
            }
        };

        let stdout = child.stdout.take().context("missing stdout pipe")?;
        let stderr = child.stderr.take().context("missing stderr pipe")?;
        let (tx, rx) = mpsc::channel();

        let stdout_handle = spawn_terminal_reader(stdout, "stdout", tx.clone());
        let stderr_handle = spawn_terminal_reader(stderr, "stderr", tx);

        let mut stdout_output = String::new();
        let mut stderr_output = String::new();
        let mut emitted_events = 0usize;
        let mut overflow_noted = false;
        let mut stdout_truncated = false;
        let mut stderr_truncated = false;

        for chunk in rx {
            match chunk.stream {
                "stdout" => append_capped(
                    &mut stdout_output,
                    &chunk.output,
                    MAX_CAPTURE_CHARS_PER_STREAM,
                    &mut stdout_truncated,
                ),
                "stderr" => append_capped(
                    &mut stderr_output,
                    &chunk.output,
                    MAX_CAPTURE_CHARS_PER_STREAM,
                    &mut stderr_truncated,
                ),
                _ => {}
            }

            let rendered = trim_terminal_chunk(&chunk.output);
            if rendered.is_empty() {
                continue;
            }
            if emitted_events < MAX_TERMINAL_EVENTS {
                event_sink.emit(TurnEvent::ToolOutput {
                    call_id: call_id.to_string(),
                    tool_name: tool_name.to_string(),
                    stream: chunk.stream.to_string(),
                    output: rendered,
                });
                emitted_events += 1;
            } else if !overflow_noted {
                event_sink.emit(TurnEvent::ToolOutput {
                    call_id: call_id.to_string(),
                    tool_name: tool_name.to_string(),
                    stream: "system".to_string(),
                    output: "additional terminal output suppressed".to_string(),
                });
                overflow_noted = true;
            }
        }

        let status = child
            .wait()
            .with_context(|| format!("failed to wait for shell command `{command}`"))?;

        stdout_handle
            .join()
            .expect("stdout terminal reader thread should join");
        stderr_handle
            .join()
            .expect("stderr terminal reader thread should join");

        if stdout_truncated {
            append_terminal_truncation_notice(&mut stdout_output, "stdout");
        }
        if stderr_truncated {
            append_terminal_truncation_notice(&mut stderr_output, "stderr");
        }

        let output = Output {
            status,
            stdout: stdout_output.into_bytes(),
            stderr: stderr_output.into_bytes(),
        };
        let last_error = if output.status.success() {
            None
        } else {
            Some(format!(
                "terminal command `{command}` exited with {}",
                output.status
            ))
        };
        self.record_execution_finished(command, last_error);
        Ok(output)
    }
}

impl ExecutionHand for BackgroundTerminalRunner {
    fn describe(&self) -> ExecutionHandDescriptor {
        Self::descriptor()
    }

    fn diagnostic(&self) -> ExecutionHandDiagnostic {
        self.current_diagnostic()
    }
}

fn spawn_terminal_reader<R: Read + Send + 'static>(
    mut reader: R,
    stream: &'static str,
    tx: mpsc::Sender<TerminalChunk>,
) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let mut buffer = [0u8; TERMINAL_READ_BUFFER_BYTES];
        loop {
            match reader.read(&mut buffer) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    let output = String::from_utf8_lossy(&buffer[..bytes_read]).into_owned();
                    if tx.send(TerminalChunk { stream, output }).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = tx.send(TerminalChunk {
                        stream: "stderr",
                        output: format!("[terminal reader error: {error}]"),
                    });
                    break;
                }
            }
        }
    })
}

fn append_capped(target: &mut String, chunk: &str, cap: usize, truncated: &mut bool) {
    if *truncated {
        return;
    }

    let current = target.chars().count();
    if current >= cap {
        *truncated = true;
        return;
    }

    let remaining = cap - current;
    let mut taken = String::new();
    for (index, ch) in chunk.chars().enumerate() {
        if index >= remaining {
            *truncated = true;
            break;
        }
        taken.push(ch);
    }
    target.push_str(&taken);
}

fn trim_terminal_chunk(chunk: &str) -> String {
    let trimmed = chunk.trim_end_matches('\0');
    if trimmed.trim().is_empty() {
        return String::new();
    }
    let mut clipped = String::new();
    for (index, ch) in trimmed.chars().enumerate() {
        if index >= MAX_TERMINAL_EVENT_CHARS {
            clipped.push_str("...");
            return clipped;
        }
        clipped.push(ch);
    }
    clipped
}

fn append_terminal_truncation_notice(target: &mut String, stream: &str) {
    if !target.is_empty() && !target.ends_with('\n') {
        target.push('\n');
    }
    target.push_str(&format!("[terminal {stream} truncated]"));
}

#[cfg(test)]
mod tests {
    use super::run_background_terminal_command_with_execution_hand_registry;
    use crate::domain::model::{
        ExecutionHandKind, ExecutionHandOperation, ExecutionHandPhase, TurnEvent, TurnEventSink,
    };
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
    fn terminal_runner_reports_hand_execution_diagnostics_after_command_completion() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::default());
        let sink = RecordingTurnEventSink::default();

        let output = run_background_terminal_command_with_execution_hand_registry(
            workspace.path(),
            "printf 'hello from terminal\\n'",
            "inspect",
            "call-1",
            &sink,
            Arc::clone(&registry),
        )
        .expect("terminal command output");

        assert!(output.status.success());
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::ToolOutput { output, .. } if output.contains("hello from terminal")
        )));

        let diagnostic = registry
            .diagnostic(ExecutionHandKind::TerminalRunner)
            .expect("terminal runner diagnostic");
        assert_eq!(diagnostic.phase, ExecutionHandPhase::Ready);
        assert_eq!(
            diagnostic.last_operation,
            Some(ExecutionHandOperation::Execute)
        );
        assert!(diagnostic.summary.contains("printf 'hello from terminal"));
    }
}
