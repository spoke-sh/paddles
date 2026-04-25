use crate::domain::model::{
    AppliedEdit, AppliedEditEvidence, AppliedEditEvidenceKind, AppliedEditEvidenceStatus,
    ExecutionGovernanceOutcome, ExecutionGovernanceOutcomeKind, ExecutionHandDescriptor,
    ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation, ExecutionHandPhase,
    ExecutionPermission, ExecutionPermissionRequest, ExecutionPermissionRequirement,
    ExecutionPolicyEvaluationInput,
};
use crate::domain::ports::{ExecutionHand, WorkspaceActionResult, WorkspaceEditor};
use crate::infrastructure::execution_governance::{
    ExecutionPolicyPermissionGate, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::transport_mediator::TransportToolMediator;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use anyhow::{Context, Result, anyhow, bail};
use std::collections::BTreeSet;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_EDITOR_OUTPUT_CHARS: usize = 12_000;
const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

static WORKSPACE_EDIT_LOCKS: OnceLock<Mutex<BTreeSet<PathBuf>>> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FileLineEnding {
    Lf,
    Crlf,
    Cr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorkspaceFileFormat {
    has_utf8_bom: bool,
    line_ending: FileLineEnding,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkspaceFileText {
    text: String,
    format: WorkspaceFileFormat,
}

struct WorkspaceEditLock {
    path: PathBuf,
}

impl Drop for WorkspaceEditLock {
    fn drop(&mut self) {
        if let Some(locks) = WORKSPACE_EDIT_LOCKS.get() {
            locks
                .lock()
                .expect("workspace edit lock registry")
                .remove(&self.path);
        }
    }
}

fn acquire_workspace_edit_lock(path: &Path) -> Result<WorkspaceEditLock> {
    let locks = WORKSPACE_EDIT_LOCKS.get_or_init(|| Mutex::new(BTreeSet::new()));
    let mut guard = locks
        .lock()
        .map_err(|_| anyhow!("workspace edit lock registry is poisoned"))?;
    let path = path.to_path_buf();
    if !guard.insert(path.clone()) {
        bail!("workspace edit lock is already held for {}", path.display());
    }
    Ok(WorkspaceEditLock { path })
}

fn acquire_workspace_edit_locks(paths: Vec<PathBuf>) -> Result<Vec<WorkspaceEditLock>> {
    let mut locks = Vec::new();
    let unique_paths = paths.into_iter().collect::<BTreeSet<_>>();
    for path in unique_paths {
        locks.push(acquire_workspace_edit_lock(&path)?);
    }
    Ok(locks)
}

fn read_workspace_file_text(path: &Path) -> Result<WorkspaceFileText> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    decode_workspace_file_text(&bytes)
        .with_context(|| format!("failed to decode {}", path.display()))
}

fn decode_workspace_file_text(bytes: &[u8]) -> Result<WorkspaceFileText> {
    let has_utf8_bom = bytes.starts_with(UTF8_BOM);
    let body = if has_utf8_bom {
        &bytes[UTF8_BOM.len()..]
    } else {
        bytes
    };
    let text = String::from_utf8(body.to_vec()).context("workspace file is not valid UTF-8")?;
    let format = WorkspaceFileFormat {
        has_utf8_bom,
        line_ending: detect_line_ending(&text),
    };
    Ok(WorkspaceFileText { text, format })
}

fn detect_line_ending(text: &str) -> FileLineEnding {
    let bytes = text.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        match byte {
            b'\n' => return FileLineEnding::Lf,
            b'\r' if bytes.get(index + 1) == Some(&b'\n') => return FileLineEnding::Crlf,
            b'\r' => return FileLineEnding::Cr,
            _ => {}
        }
    }
    FileLineEnding::Lf
}

fn encode_workspace_file_text(text: &str, format: WorkspaceFileFormat) -> Vec<u8> {
    let normalized = normalize_line_endings_for_format(text, format.line_ending);
    let mut bytes = Vec::with_capacity(normalized.len() + UTF8_BOM.len());
    if format.has_utf8_bom {
        bytes.extend_from_slice(UTF8_BOM);
    }
    bytes.extend_from_slice(normalized.as_bytes());
    bytes
}

fn normalize_line_endings_for_format(text: &str, line_ending: FileLineEnding) -> String {
    let normalized = normalize_line_endings_to_lf(text);
    match line_ending {
        FileLineEnding::Lf => normalized,
        FileLineEnding::Crlf => normalized.replace('\n', "\r\n"),
        FileLineEnding::Cr => normalized.replace('\n', "\r"),
    }
}

fn normalize_line_endings_to_lf(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

#[derive(Clone, Debug, Default)]
pub struct WorkspaceEditEvidenceHooks {
    pub formatter: Option<WorkspaceEditHookCommand>,
    pub diagnostics: Option<WorkspaceEditHookCommand>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkspaceEditHookCommand {
    kind: AppliedEditEvidenceKind,
    program: String,
    args: Vec<String>,
}

impl WorkspaceEditHookCommand {
    #[allow(dead_code)]
    pub fn new(
        kind: AppliedEditEvidenceKind,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            kind,
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }

    fn rendered_args(&self, paths: &[String]) -> Vec<String> {
        let first_path = paths.first().map(String::as_str).unwrap_or_default();
        let joined_paths = paths.join(" ");
        self.args
            .iter()
            .map(|arg| {
                arg.replace("{path}", first_path)
                    .replace("{paths}", &joined_paths)
            })
            .collect()
    }

    fn display_command(&self, args: &[String]) -> String {
        if args.is_empty() {
            self.program.clone()
        } else {
            format!("{} {}", self.program, args.join(" "))
        }
    }
}

#[derive(Clone, Debug)]
pub struct LocalWorkspaceEditor {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    edit_evidence_hooks: WorkspaceEditEvidenceHooks,
}

impl LocalWorkspaceEditor {
    #[allow(dead_code)]
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self::with_execution_hand_registry(
            workspace_root,
            Arc::new(ExecutionHandRegistry::with_default_local_governance()),
        )
    }

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
            edit_evidence_hooks: WorkspaceEditEvidenceHooks::default(),
        }
    }

    #[allow(dead_code)]
    pub fn with_edit_evidence_hooks(mut self, hooks: WorkspaceEditEvidenceHooks) -> Self {
        self.edit_evidence_hooks = hooks;
        self
    }

    fn descriptor() -> ExecutionHandDescriptor {
        ExecutionHandDescriptor::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionHandKind::WorkspaceEditor.default_authority(),
            ExecutionHandKind::WorkspaceEditor.default_summary(),
            vec![
                ExecutionHandOperation::Describe,
                ExecutionHandOperation::Provision,
                ExecutionHandOperation::Execute,
                ExecutionHandOperation::Recover,
                ExecutionHandOperation::Degrade,
            ],
        )
    }

    fn record_execution_started(&self, summary: impl Into<String>) {
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionHandPhase::Executing,
            ExecutionHandOperation::Execute,
            summary,
            None,
        );
    }

    fn record_execution_finished(&self, summary: impl Into<String>, last_error: Option<String>) {
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionHandPhase::Ready,
            ExecutionHandOperation::Execute,
            summary,
            last_error,
        );
    }

    fn protect_command_env(&self, command: &mut Command, purpose: &str) {
        self.transport_mediator
            .protect_command_env(command, purpose);
    }

    fn evaluate_request(
        &self,
        request: &ExecutionPermissionRequest,
        policy_input: &ExecutionPolicyEvaluationInput,
    ) -> ExecutionGovernanceOutcome {
        let governance_profile = self.execution_hand_registry.governance_profile();
        let execution_policy = self.execution_hand_registry.execution_policy();
        ExecutionPolicyPermissionGate::evaluate(
            execution_policy.as_ref(),
            governance_profile.as_ref(),
            request,
            policy_input,
        )
    }

    fn blocked_result(
        &self,
        action_name: &str,
        blocked_summary: impl Into<String>,
        request: ExecutionPermissionRequest,
        outcome: ExecutionGovernanceOutcome,
    ) -> WorkspaceActionResult {
        let summary = summarize_governance_outcome(&outcome);
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionHandPhase::Degraded,
            ExecutionHandOperation::Degrade,
            blocked_summary,
            Some(summary.clone()),
        );
        WorkspaceActionResult {
            name: action_name.to_string(),
            summary,
            applied_edit: None,
            governance_request: Some(request),
            governance_outcome: Some(outcome),
        }
    }

    fn collect_edit_evidence(&self, paths: &[String]) -> Vec<AppliedEditEvidence> {
        let mut evidence = Vec::new();
        if let Some(formatter) = &self.edit_evidence_hooks.formatter {
            evidence.push(run_edit_evidence_hook(
                formatter,
                paths,
                &self.workspace_root,
                &self.transport_mediator,
            ));
        }
        if let Some(diagnostics) = &self.edit_evidence_hooks.diagnostics {
            evidence.push(run_edit_evidence_hook(
                diagnostics,
                paths,
                &self.workspace_root,
                &self.transport_mediator,
            ));
        }
        evidence
    }
}

impl ExecutionHand for LocalWorkspaceEditor {
    fn describe(&self) -> ExecutionHandDescriptor {
        Self::descriptor()
    }

    fn diagnostic(&self) -> ExecutionHandDiagnostic {
        self.execution_hand_registry
            .diagnostic(ExecutionHandKind::WorkspaceEditor)
            .unwrap_or_else(|| ExecutionHandDiagnostic::from_descriptor(&self.describe()))
    }
}

impl WorkspaceEditor for LocalWorkspaceEditor {
    fn diff(&self, path: Option<&str>) -> Result<WorkspaceActionResult> {
        let target = path.unwrap_or(".").trim();
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionPermissionRequirement::new(
                format!("diff `{target}`"),
                vec![ExecutionPermission::ReadWorkspace],
            ),
        );
        let policy_input = ExecutionPolicyEvaluationInput::tool("diff");
        let governance = self.evaluate_request(&request, &policy_input);
        if !matches!(governance.kind, ExecutionGovernanceOutcomeKind::Allowed) {
            return Ok(self.blocked_result(
                "diff",
                format!("workspace editor blocked diff for {target}"),
                request,
                governance,
            ));
        }
        self.record_execution_started(format!("workspace editor diffing {target}"));
        let mut command = Command::new("git");
        let result = (|| {
            command
                .arg("diff")
                .arg("--no-ext-diff")
                .current_dir(&self.workspace_root);
            self.protect_command_env(&mut command, "workspace diff command");
            if let Some(path) = path.filter(|path| !path.trim().is_empty()) {
                let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
                command
                    .arg("--")
                    .arg(relative_path(&self.workspace_root, &resolved));
            }
            let output = command.output().context("failed to run git diff")?;
            if !output.status.success() {
                bail!(
                    "{}",
                    format_command_summary("git diff", "git diff --no-ext-diff", &output)
                );
            }

            let diff = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let summary = if diff.is_empty() && stderr.is_empty() {
                "No diff output.".to_string()
            } else {
                format!(
                    "Diff output:\n{}\n{}",
                    trim_for_summary(&diff, MAX_EDITOR_OUTPUT_CHARS),
                    trim_for_summary(&stderr, MAX_EDITOR_OUTPUT_CHARS / 2)
                )
            };
            Ok(WorkspaceActionResult {
                name: "diff".to_string(),
                summary,
                applied_edit: None,
                governance_request: Some(request.clone()),
                governance_outcome: Some(governance.clone()),
            })
        })();
        match &result {
            Ok(outcome) => {
                self.record_execution_finished(
                    format!(
                        "workspace editor completed diff for {target}: {}",
                        outcome.name
                    ),
                    None,
                );
            }
            Err(error) => {
                self.record_execution_finished(
                    format!("workspace editor diff for {target} failed"),
                    Some(error.to_string()),
                );
            }
        }
        result
    }

    fn write_file(&self, path: &str, content: &str) -> Result<WorkspaceActionResult> {
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionPermissionRequirement::new(
                format!("write `{path}`"),
                vec![
                    ExecutionPermission::ReadWorkspace,
                    ExecutionPermission::WriteWorkspace,
                ],
            ),
        )
        .with_bounded_reuse(
            crate::domain::model::ExecutionPermissionReuseScope::Turn,
            Vec::new(),
        );
        let policy_input = ExecutionPolicyEvaluationInput::tool("write_file");
        let governance = self.evaluate_request(&request, &policy_input);
        if !matches!(governance.kind, ExecutionGovernanceOutcomeKind::Allowed) {
            return Ok(self.blocked_result(
                "write_file",
                format!("workspace editor blocked write_file for {path}"),
                request,
                governance,
            ));
        }
        self.record_execution_started(format!("workspace editor writing {path}"));
        let result: Result<WorkspaceActionResult> = (|| {
            let resolved = resolve_workspace_path(&self.workspace_root, path, true)?;
            let _edit_lock = acquire_workspace_edit_lock(&resolved)?;
            let existing = if resolved.exists() {
                Some(read_workspace_file_text(&resolved)?)
            } else {
                None
            };
            let before = existing
                .as_ref()
                .map(|file_text| file_text.text.clone())
                .unwrap_or_default();
            let after = existing
                .as_ref()
                .map(|file_text| {
                    normalize_line_endings_for_format(content, file_text.format.line_ending)
                })
                .unwrap_or_else(|| content.to_string());
            if let Some(parent) = resolved.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create parent directory {}", parent.display())
                })?;
            }
            let bytes = existing
                .as_ref()
                .map(|file_text| encode_workspace_file_text(&after, file_text.format))
                .unwrap_or_else(|| after.as_bytes().to_vec());
            fs::write(&resolved, bytes)
                .with_context(|| format!("failed to write {}", resolved.display()))?;
            let rel = relative_path(&self.workspace_root, &resolved);
            let evidence = self.collect_edit_evidence(std::slice::from_ref(&rel));
            let after = if resolved.exists() {
                read_workspace_file_text(&resolved)?.text
            } else {
                after
            };
            let mut applied_edit =
                build_applied_edit(&rel, &before, &after, &self.transport_mediator)?;
            applied_edit.evidence = evidence;
            Ok(WorkspaceActionResult {
                name: "write_file".to_string(),
                summary: summarize_applied_edit("write_file", &applied_edit),
                applied_edit: Some(applied_edit),
                governance_request: Some(request.clone()),
                governance_outcome: Some(governance.clone()),
            })
        })();
        match &result {
            Ok(_) => {
                self.record_execution_finished(
                    format!("workspace editor completed write_file for {path}"),
                    None,
                );
            }
            Err(error) => {
                self.record_execution_finished(
                    format!("workspace editor write_file for {path} failed"),
                    Some(error.to_string()),
                );
            }
        }
        result
    }

    fn replace_in_file(
        &self,
        path: &str,
        old: &str,
        new: &str,
        replace_all: bool,
    ) -> Result<WorkspaceActionResult> {
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionPermissionRequirement::new(
                format!("replace content in `{path}`"),
                vec![
                    ExecutionPermission::ReadWorkspace,
                    ExecutionPermission::WriteWorkspace,
                ],
            ),
        )
        .with_bounded_reuse(
            crate::domain::model::ExecutionPermissionReuseScope::Turn,
            Vec::new(),
        );
        let policy_input = ExecutionPolicyEvaluationInput::tool("replace_in_file");
        let governance = self.evaluate_request(&request, &policy_input);
        if !matches!(governance.kind, ExecutionGovernanceOutcomeKind::Allowed) {
            return Ok(self.blocked_result(
                "replace_in_file",
                format!("workspace editor blocked replace_in_file for {path}"),
                request,
                governance,
            ));
        }
        self.record_execution_started(format!("workspace editor replacing in {path}"));
        let result: Result<WorkspaceActionResult> = (|| {
            let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
            let _edit_lock = acquire_workspace_edit_lock(&resolved)?;
            let original = read_workspace_file_text(&resolved)?;
            let old = normalize_line_endings_for_format(old, original.format.line_ending);
            let new = normalize_line_endings_for_format(new, original.format.line_ending);
            if old.is_empty() {
                bail!(
                    "replacement pattern cannot be empty for {}",
                    resolved.display()
                );
            }
            let candidates = replacement_candidates(&original.text, &old);
            if candidates.is_empty() {
                bail!("pattern not found in {}", resolved.display());
            }
            if !replace_all && candidates.len() > 1 {
                bail!(
                    "{}",
                    format_ambiguous_replacement_error(&resolved, &candidates)
                );
            }
            let updated = if replace_all {
                original.text.replace(&old, &new)
            } else {
                original.text.replacen(&old, &new, 1)
            };
            fs::write(
                &resolved,
                encode_workspace_file_text(&updated, original.format),
            )
            .with_context(|| format!("failed to write {}", resolved.display()))?;
            let rel = relative_path(&self.workspace_root, &resolved);
            let evidence = self.collect_edit_evidence(std::slice::from_ref(&rel));
            let updated = if resolved.exists() {
                read_workspace_file_text(&resolved)?.text
            } else {
                updated
            };
            let mut applied_edit =
                build_applied_edit(&rel, &original.text, &updated, &self.transport_mediator)?;
            applied_edit.evidence = evidence;
            Ok(WorkspaceActionResult {
                name: "replace_in_file".to_string(),
                summary: summarize_applied_edit("replace_in_file", &applied_edit),
                applied_edit: Some(applied_edit),
                governance_request: Some(request.clone()),
                governance_outcome: Some(governance.clone()),
            })
        })();
        match &result {
            Ok(_) => {
                self.record_execution_finished(
                    format!("workspace editor completed replace_in_file for {path}"),
                    None,
                );
            }
            Err(error) => {
                self.record_execution_finished(
                    format!("workspace editor replace_in_file for {path} failed"),
                    Some(error.to_string()),
                );
            }
        }
        result
    }

    fn apply_patch(&self, patch: &str) -> Result<WorkspaceActionResult> {
        let request = ExecutionPermissionRequest::new(
            ExecutionHandKind::WorkspaceEditor,
            ExecutionPermissionRequirement::new(
                "apply a workspace patch",
                vec![
                    ExecutionPermission::ReadWorkspace,
                    ExecutionPermission::WriteWorkspace,
                ],
            ),
        )
        .with_bounded_reuse(
            crate::domain::model::ExecutionPermissionReuseScope::Turn,
            Vec::new(),
        );
        let policy_input = ExecutionPolicyEvaluationInput::tool("apply_patch");
        let governance = self.evaluate_request(&request, &policy_input);
        if !matches!(governance.kind, ExecutionGovernanceOutcomeKind::Allowed) {
            return Ok(self.blocked_result(
                "apply_patch",
                "workspace editor blocked apply_patch",
                request,
                governance,
            ));
        }
        self.record_execution_started("workspace editor applying patch");
        let result: Result<WorkspaceActionResult> = (|| {
            let patch_paths = extract_diff_paths(patch);
            if patch_paths.is_empty() {
                bail!("patch does not target an authored workspace file");
            }
            let path_policy = WorkspacePathPolicy::new(&self.workspace_root);
            for path in &patch_paths {
                ensure_authored_workspace_path(&path_policy, path)?;
            }
            let resolved_paths = patch_paths
                .iter()
                .map(|path| resolve_workspace_path(&self.workspace_root, path, true))
                .collect::<Result<Vec<_>>>()?;
            let _edit_locks = acquire_workspace_edit_locks(resolved_paths.clone())?;
            let original_formats = resolved_paths
                .iter()
                .filter(|path| path.exists())
                .map(|path| {
                    read_workspace_file_text(path).map(|file_text| (path.clone(), file_text.format))
                })
                .collect::<Result<Vec<_>>>()?;

            let mut command = Command::new("git");
            command
                .arg("apply")
                .arg("--whitespace=nowarn")
                .arg("-")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .current_dir(&self.workspace_root);
            self.protect_command_env(&mut command, "workspace apply_patch command");
            let mut child = command.spawn().context("failed to spawn git apply")?;

            if let Some(stdin) = child.stdin.as_mut() {
                stdin
                    .write_all(patch.as_bytes())
                    .context("failed to write patch to git apply")?;
            }

            let output = child
                .wait_with_output()
                .context("failed to wait for git apply")?;
            let summary =
                summarize_apply_patch_result(patch, "git apply --whitespace=nowarn -", &output);
            if !output.status.success() {
                bail!("{summary}");
            }
            for (path, format) in original_formats {
                if !path.exists() {
                    continue;
                }
                let patched = read_workspace_file_text(&path)?;
                fs::write(&path, encode_workspace_file_text(&patched.text, format))
                    .with_context(|| format!("failed to preserve format for {}", path.display()))?;
            }
            let evidence = self.collect_edit_evidence(&patch_paths);
            let mut applied_edit = build_patch_applied_edit(patch);
            applied_edit.evidence = evidence;
            Ok(WorkspaceActionResult {
                name: "apply_patch".to_string(),
                summary: summarize_applied_edit("apply_patch", &applied_edit),
                applied_edit: Some(applied_edit),
                governance_request: Some(request.clone()),
                governance_outcome: Some(governance.clone()),
            })
        })();
        match &result {
            Ok(_) => {
                self.record_execution_finished("workspace editor completed apply_patch", None);
            }
            Err(error) => {
                self.record_execution_finished(
                    "workspace editor apply_patch failed",
                    Some(error.to_string()),
                );
            }
        }
        result
    }
}

fn summarize_apply_patch_result(patch: &str, command: &str, output: &Output) -> String {
    let patch_preview = trim_for_summary(patch, MAX_EDITOR_OUTPUT_CHARS / 2);
    let mut summary = String::new();
    summary.push_str("Applied patch:\n");
    summary.push_str(&patch_preview);
    summary.push('\n');
    summary.push('\n');
    summary.push_str(&format_command_summary("git apply", command, output));
    trim_for_summary(&summary, MAX_EDITOR_OUTPUT_CHARS)
}

fn run_edit_evidence_hook(
    hook: &WorkspaceEditHookCommand,
    paths: &[String],
    workspace_root: &Path,
    transport_mediator: &TransportToolMediator,
) -> AppliedEditEvidence {
    let args = hook.rendered_args(paths);
    let display_command = hook.display_command(&args);
    let mut command = Command::new(&hook.program);
    command
        .args(&args)
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    transport_mediator.protect_command_env(&mut command, "workspace edit evidence hook");

    match command.output() {
        Ok(output) if output.status.success() => applied_edit_evidence(
            hook.kind,
            AppliedEditEvidenceStatus::Passed,
            summarize_edit_evidence_output(
                hook.kind,
                AppliedEditEvidenceStatus::Passed,
                &display_command,
                &output,
            ),
        ),
        Ok(output) => applied_edit_evidence(
            hook.kind,
            AppliedEditEvidenceStatus::Warning,
            summarize_edit_evidence_output(
                hook.kind,
                AppliedEditEvidenceStatus::Warning,
                &display_command,
                &output,
            ),
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => applied_edit_evidence(
            hook.kind,
            AppliedEditEvidenceStatus::Unavailable,
            format!(
                "{}: unavailable ({display_command}): {error}",
                hook.kind.label()
            ),
        ),
        Err(error) => applied_edit_evidence(
            hook.kind,
            AppliedEditEvidenceStatus::Warning,
            format!(
                "{}: warning ({display_command}): {error}",
                hook.kind.label()
            ),
        ),
    }
}

fn applied_edit_evidence(
    kind: AppliedEditEvidenceKind,
    status: AppliedEditEvidenceStatus,
    summary: String,
) -> AppliedEditEvidence {
    AppliedEditEvidence {
        kind,
        status,
        summary: trim_for_summary(&summary, MAX_EDITOR_OUTPUT_CHARS / 3),
    }
}

fn summarize_edit_evidence_output(
    kind: AppliedEditEvidenceKind,
    status: AppliedEditEvidenceStatus,
    command: &str,
    output: &Output,
) -> String {
    let mut summary = format!("{}: {} ({command})", kind.label(), status.label());
    if !output.stdout.is_empty() {
        summary.push_str("\nstdout:\n");
        summary.push_str(&String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        summary.push_str("\nstderr:\n");
        summary.push_str(&String::from_utf8_lossy(&output.stderr));
    }
    trim_for_summary(&summary, MAX_EDITOR_OUTPUT_CHARS / 3)
}

fn format_command_summary(header: &str, command: &str, output: &Output) -> String {
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

    trim_for_summary(&summary, MAX_EDITOR_OUTPUT_CHARS)
}

fn trim_for_summary(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }

    let trimmed = input.chars().take(max_chars).collect::<String>();
    format!("{trimmed}...")
}

fn build_applied_edit(
    path: &str,
    before: &str,
    after: &str,
    transport_mediator: &TransportToolMediator,
) -> Result<AppliedEdit> {
    let diff = unified_diff(path, before, after, transport_mediator)?;
    let (insertions, deletions) = diff_change_counts(&diff);
    Ok(AppliedEdit {
        files: vec![path.to_string()],
        diff,
        insertions,
        deletions,
        evidence: Vec::new(),
    })
}

fn build_patch_applied_edit(patch: &str) -> AppliedEdit {
    let diff = patch.trim().to_string();
    let (insertions, deletions) = diff_change_counts(&diff);
    AppliedEdit {
        files: extract_diff_paths(&diff),
        diff,
        insertions,
        deletions,
        evidence: Vec::new(),
    }
}

fn summarize_applied_edit(tool_name: &str, edit: &AppliedEdit) -> String {
    let files = if edit.files.is_empty() {
        "(unknown file)".to_string()
    } else {
        edit.files.join(", ")
    };
    let mut summary = format!(
        "Applied {tool_name} to {files} (+{} -{}).",
        edit.insertions, edit.deletions
    );
    if !edit.diff.trim().is_empty() {
        summary.push_str("\n\n");
        summary.push_str(&edit.diff);
    }
    if !edit.evidence.is_empty() {
        summary.push_str("\n\nEvidence:");
        for evidence in &edit.evidence {
            summary.push_str("\n- ");
            summary.push_str(&evidence.summary);
        }
    }
    trim_for_summary(&summary, MAX_EDITOR_OUTPUT_CHARS)
}

fn unified_diff(
    path: &str,
    before: &str,
    after: &str,
    transport_mediator: &TransportToolMediator,
) -> Result<String> {
    let temp_root = std::env::temp_dir().join(format!(
        "paddles-workspace-edit-{}-{}",
        std::process::id(),
        unique_nonce()
    ));
    fs::create_dir_all(&temp_root)
        .with_context(|| format!("failed to create {}", temp_root.display()))?;
    let before_path = temp_root.join("before.txt");
    let after_path = temp_root.join("after.txt");
    fs::write(&before_path, before)
        .with_context(|| format!("failed to write {}", before_path.display()))?;
    fs::write(&after_path, after)
        .with_context(|| format!("failed to write {}", after_path.display()))?;

    let command = format!("diff -u --label a/{path} --label b/{path} <before> <after>");
    let mut command_runner = Command::new("diff");
    command_runner
        .arg("-u")
        .arg("--label")
        .arg(format!("a/{path}"))
        .arg("--label")
        .arg(format!("b/{path}"))
        .arg(&before_path)
        .arg(&after_path);
    transport_mediator.protect_command_env(&mut command_runner, "workspace unified diff command");
    let output = command_runner.output().context("failed to run diff")?;

    let _ = fs::remove_dir_all(&temp_root);

    match output.status.code() {
        Some(0) => Ok(format!("--- a/{path}\n+++ b/{path}")),
        Some(1) => Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string()),
        _ => bail!("{}", format_command_summary("diff", &command, &output)),
    }
}

fn unique_nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn diff_change_counts(diff: &str) -> (usize, usize) {
    let mut insertions = 0;
    let mut deletions = 0;
    for line in diff.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            insertions += 1;
        } else if line.starts_with('-') {
            deletions += 1;
        }
    }
    (insertions, deletions)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReplacementCandidateContext {
    line: usize,
    excerpt: String,
}

fn replacement_candidates(text: &str, pattern: &str) -> Vec<ReplacementCandidateContext> {
    if pattern.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    let mut search_start = 0;
    while let Some(offset) = text[search_start..].find(pattern) {
        let index = search_start + offset;
        candidates.push(ReplacementCandidateContext {
            line: line_number_at(text, index),
            excerpt: line_excerpt_at(text, index),
        });
        search_start = index + pattern.len();
    }
    candidates
}

fn line_number_at(text: &str, index: usize) -> usize {
    text[..index].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn line_excerpt_at(text: &str, index: usize) -> String {
    let line_start = text[..index]
        .rfind('\n')
        .map(|start| start + 1)
        .unwrap_or(0);
    let line_end = text[index..]
        .find('\n')
        .map(|offset| index + offset)
        .unwrap_or(text.len());
    trim_for_summary(text[line_start..line_end].trim_end_matches('\r'), 160)
}

fn format_ambiguous_replacement_error(
    path: &Path,
    candidates: &[ReplacementCandidateContext],
) -> String {
    let context = candidates
        .iter()
        .take(5)
        .map(|candidate| format!("line {}: {}", candidate.line, candidate.excerpt))
        .collect::<Vec<_>>()
        .join("; ");
    let suffix = if candidates.len() > 5 {
        format!("; {} more matches omitted", candidates.len() - 5)
    } else {
        String::new()
    };
    format!(
        "ambiguous replacement in {}: pattern matched {} locations. Candidate context: {context}{suffix}",
        path.display(),
        candidates.len()
    )
}

fn extract_diff_paths(diff: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in diff.lines() {
        let Some(path) = line.strip_prefix("+++ ") else {
            continue;
        };
        let candidate = path.split_whitespace().next().unwrap_or(path);
        let candidate = candidate
            .strip_prefix("b/")
            .or_else(|| candidate.strip_prefix("a/"))
            .unwrap_or(candidate);
        if candidate.is_empty() || candidate == "/dev/null" {
            continue;
        }
        if !paths.iter().any(|existing| existing == candidate) {
            paths.push(candidate.to_string());
        }
    }
    paths
}

pub(crate) fn resolve_workspace_path(
    workspace_root: &Path,
    requested: &str,
    allow_missing: bool,
) -> Result<PathBuf> {
    let requested_path = Path::new(requested);
    if requested_path.is_absolute() {
        bail!("absolute paths are not allowed: {requested}");
    }

    let canonical_root = workspace_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", workspace_root.display()))?;
    let normalized = normalize_relative_path(&canonical_root, requested_path);
    let resolved = resolve_existing_path(&canonical_root, &normalized)?;
    if !resolved.starts_with(&canonical_root) {
        bail!("path escapes workspace root: {requested}");
    }
    let path_policy = WorkspacePathPolicy::new(&canonical_root);
    ensure_authored_workspace_path(
        &path_policy,
        &relative_path(&canonical_root, &resolved).replace('\\', "/"),
    )?;
    if !allow_missing && !resolved.exists() {
        bail!("path does not exist: {}", resolved.display());
    }
    Ok(resolved)
}

fn resolve_existing_path(workspace_root: &Path, candidate: &Path) -> Result<PathBuf> {
    let mut existing = candidate.to_path_buf();
    let mut missing_components = Vec::new();

    while !existing.exists() {
        let missing = existing
            .file_name()
            .ok_or_else(|| anyhow!("path escapes workspace root: {}", candidate.display()))?
            .to_os_string();
        missing_components.push(missing);
        if !existing.pop() {
            bail!("path escapes workspace root: {}", candidate.display());
        }
    }

    let mut resolved = existing
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", existing.display()))?;
    if !resolved.starts_with(workspace_root) {
        bail!("path escapes workspace root: {}", candidate.display());
    }

    for component in missing_components.iter().rev() {
        resolved.push(component);
    }

    Ok(resolved)
}

fn normalize_relative_path(workspace_root: &Path, requested: &Path) -> PathBuf {
    let mut normalized = workspace_root.to_path_buf();
    for component in requested.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::Prefix(_) | Component::RootDir => {}
        }
    }
    normalized
}

fn ensure_authored_workspace_path(
    path_policy: &WorkspacePathPolicy,
    requested: &str,
) -> Result<()> {
    if !path_policy.allows_relative_file(requested) {
        bail!("path is outside the authored workspace boundary: {requested}");
    }
    Ok(())
}

pub(crate) fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{LocalWorkspaceEditor, WorkspaceEditEvidenceHooks, WorkspaceEditHookCommand};
    use crate::domain::model::{AppliedEditEvidenceKind, AppliedEditEvidenceStatus};
    use crate::domain::model::{
        ExecutionApprovalPolicy, ExecutionGovernanceOutcomeKind, ExecutionGovernanceProfile,
        ExecutionHandKind, ExecutionHandOperation, ExecutionHandPhase,
        ExecutionPermissionReuseScope, ExecutionSandboxMode,
    };
    use crate::domain::ports::{ExecutionHand, WorkspaceEditor};
    use crate::infrastructure::execution_hand::ExecutionHandRegistry;
    use std::fs;
    use std::sync::Arc;

    #[test]
    fn edit_actions_return_structured_applied_edit_artifacts() {
        let workspace = tempfile::tempdir().expect("workspace");
        let editor = LocalWorkspaceEditor::new(workspace.path());

        let write = editor
            .write_file("notes.txt", "alpha\n")
            .expect("write_file result");
        let write_edit = write.applied_edit.expect("write_file applied edit");
        assert_eq!(write_edit.files, vec!["notes.txt".to_string()]);
        assert!(write_edit.diff.contains("+++ b/notes.txt"));
        assert_eq!(write_edit.insertions, 1);
        assert_eq!(write_edit.deletions, 0);

        fs::write(workspace.path().join("guide.txt"), "before\n").expect("seed guide");
        let replace = editor
            .replace_in_file("guide.txt", "before", "after", false)
            .expect("replace_in_file result");
        let replace_edit = replace.applied_edit.expect("replace applied edit");
        assert_eq!(replace_edit.files, vec!["guide.txt".to_string()]);
        assert!(replace_edit.diff.contains("--- a/guide.txt"));
        assert!(replace_edit.diff.contains("+++ b/guide.txt"));
        assert!(replace_edit.diff.contains("-before"));
        assert!(replace_edit.diff.contains("+after"));
        assert_eq!(replace_edit.insertions, 1);
        assert_eq!(replace_edit.deletions, 1);

        let patch = "diff --git a/guide.txt b/guide.txt\n--- a/guide.txt\n+++ b/guide.txt\n@@ -1 +1 @@\n-after\n+done\n";
        let apply = editor.apply_patch(patch).expect("apply_patch result");
        let apply_edit = apply.applied_edit.expect("apply_patch applied edit");
        assert_eq!(apply_edit.files, vec!["guide.txt".to_string()]);
        assert!(
            apply_edit
                .diff
                .contains("diff --git a/guide.txt b/guide.txt")
        );
        assert!(apply_edit.diff.contains("+done"));
        assert_eq!(apply_edit.insertions, 1);
        assert_eq!(apply_edit.deletions, 1);
    }

    #[test]
    fn workspace_editor_rejects_non_authored_paths() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(
            workspace
                .path()
                .join("apps/docs/node_modules/playwright-core/lib"),
        )
        .expect("create vendored dir");
        fs::write(
            workspace
                .path()
                .join("apps/docs/node_modules/playwright-core/lib/compare.js"),
            "before\n",
        )
        .expect("seed vendored file");
        let editor = LocalWorkspaceEditor::new(workspace.path());

        let write_error = editor
            .write_file(
                "apps/docs/node_modules/playwright-core/lib/new-file.js",
                "export const value = 1;\n",
            )
            .expect_err("write_file should reject vendored paths");
        assert!(
            write_error
                .to_string()
                .contains("outside the authored workspace boundary")
        );

        let patch = "diff --git a/apps/docs/node_modules/playwright-core/lib/compare.js b/apps/docs/node_modules/playwright-core/lib/compare.js\n--- a/apps/docs/node_modules/playwright-core/lib/compare.js\n+++ b/apps/docs/node_modules/playwright-core/lib/compare.js\n@@ -1 +1 @@\n-before\n+after\n";
        let patch_error = editor
            .apply_patch(patch)
            .expect_err("apply_patch should reject vendored paths");
        assert!(
            patch_error
                .to_string()
                .contains("outside the authored workspace boundary")
        );
    }

    #[test]
    fn workspace_editor_rejects_gitignored_paths() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated docs dir");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        fs::write(
            workspace
                .path()
                .join("apps/docs/.docusaurus/client-modules.js"),
            "before\n",
        )
        .expect("seed generated docs file");
        let editor = LocalWorkspaceEditor::new(workspace.path());

        let write_error = editor
            .write_file("apps/docs/.docusaurus/new-file.js", "export default [];\n")
            .expect_err("write_file should reject gitignored paths");
        assert!(
            write_error
                .to_string()
                .contains("outside the authored workspace boundary")
        );

        let patch = "diff --git a/apps/docs/.docusaurus/client-modules.js b/apps/docs/.docusaurus/client-modules.js\n--- a/apps/docs/.docusaurus/client-modules.js\n+++ b/apps/docs/.docusaurus/client-modules.js\n@@ -1 +1 @@\n-before\n+after\n";
        let patch_error = editor
            .apply_patch(patch)
            .expect_err("apply_patch should reject gitignored paths");
        assert!(
            patch_error
                .to_string()
                .contains("outside the authored workspace boundary")
        );
    }

    #[test]
    fn workspace_editor_reports_hand_execution_diagnostics_after_successful_write() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::with_default_local_governance());
        let editor = LocalWorkspaceEditor::with_execution_hand_registry(
            workspace.path(),
            Arc::clone(&registry),
        );

        editor
            .write_file("notes.txt", "alpha\n")
            .expect("write_file result");

        let diagnostic = editor.diagnostic();
        assert_eq!(diagnostic.hand, ExecutionHandKind::WorkspaceEditor);
        assert_eq!(diagnostic.phase, ExecutionHandPhase::Ready);
        assert_eq!(
            diagnostic.last_operation,
            Some(ExecutionHandOperation::Execute)
        );
        assert!(diagnostic.summary.contains("write_file"));

        let registry_diagnostic = registry
            .diagnostic(ExecutionHandKind::WorkspaceEditor)
            .expect("workspace editor diagnostic");
        assert_eq!(registry_diagnostic, diagnostic);
    }

    #[test]
    fn workspace_editor_returns_structured_escalation_without_writing_when_policy_blocks() {
        let workspace = tempfile::tempdir().expect("workspace");
        let registry = Arc::new(ExecutionHandRegistry::default());
        registry.set_governance_profile(ExecutionGovernanceProfile::new(
            ExecutionSandboxMode::ReadOnly,
            ExecutionApprovalPolicy::OnRequest,
            vec![ExecutionPermissionReuseScope::Turn],
            None,
        ));
        let editor = LocalWorkspaceEditor::with_execution_hand_registry(
            workspace.path(),
            Arc::clone(&registry),
        );

        let result = editor
            .write_file("notes.txt", "alpha\n")
            .expect("blocked write_file still returns a structured result");

        assert!(!workspace.path().join("notes.txt").exists());
        assert!(result.applied_edit.is_none());
        let outcome = result
            .governance_outcome
            .expect("structured governance outcome");
        assert_eq!(
            outcome.kind,
            ExecutionGovernanceOutcomeKind::EscalationRequired
        );
        let escalation = outcome.escalation_request.expect("escalation request");
        assert_eq!(
            escalation.reuse_scope,
            Some(ExecutionPermissionReuseScope::Turn)
        );
    }

    #[test]
    fn workspace_edit_preserves_format_for_write_replace_and_patch() {
        let workspace = tempfile::tempdir().expect("workspace");
        let editor = LocalWorkspaceEditor::new(workspace.path());

        fs::write(
            workspace.path().join("write.txt"),
            b"\xEF\xBB\xBFold\r\nvalue\r\n",
        )
        .expect("seed write file");
        editor
            .write_file("write.txt", "new\nvalue\n")
            .expect("write_file result");
        assert_eq!(
            fs::read(workspace.path().join("write.txt")).expect("read write file"),
            b"\xEF\xBB\xBFnew\r\nvalue\r\n"
        );

        fs::write(
            workspace.path().join("replace.txt"),
            b"\xEF\xBB\xBFhello\r\nold\r\n",
        )
        .expect("seed replace file");
        editor
            .replace_in_file("replace.txt", "old", "new\nline", false)
            .expect("replace_in_file result");
        assert_eq!(
            fs::read(workspace.path().join("replace.txt")).expect("read replace file"),
            b"\xEF\xBB\xBFhello\r\nnew\r\nline\r\n"
        );

        fs::write(
            workspace.path().join("patch.txt"),
            b"\xEF\xBB\xBFalpha\r\nbeta\r\n",
        )
        .expect("seed patch file");
        let patch = "diff --git a/patch.txt b/patch.txt\n--- a/patch.txt\n+++ b/patch.txt\n@@ -2 +2 @@\n-beta\r\n+gamma\r\n";
        editor.apply_patch(patch).expect("apply_patch result");
        assert_eq!(
            fs::read(workspace.path().join("patch.txt")).expect("read patch file"),
            b"\xEF\xBB\xBFalpha\r\ngamma\r\n"
        );
    }

    #[test]
    fn workspace_edit_locking_rejects_overlapping_file_edits_with_clear_evidence() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("locked.txt"), "old\n").expect("seed locked file");
        let editor = LocalWorkspaceEditor::new(workspace.path());
        let resolved = super::resolve_workspace_path(workspace.path(), "locked.txt", false)
            .expect("resolve file");
        let _held_lock = super::acquire_workspace_edit_lock(&resolved).expect("hold edit lock");

        let error = editor
            .write_file("locked.txt", "new\n")
            .expect_err("overlapping edit should be rejected");

        assert!(
            error.to_string().contains("workspace edit lock"),
            "lock rejection should explain the locked file"
        );
        assert_eq!(
            fs::read_to_string(workspace.path().join("locked.txt")).expect("read locked file"),
            "old\n"
        );
    }

    #[test]
    fn workspace_replace_ambiguous_returns_candidate_context_without_editing() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("ambiguous.txt"),
            "alpha\nmiddle\nalpha\n",
        )
        .expect("seed ambiguous file");
        let editor = LocalWorkspaceEditor::new(workspace.path());

        let error = editor
            .replace_in_file("ambiguous.txt", "alpha", "beta", false)
            .expect_err("ambiguous replacement should be rejected");

        let message = error.to_string();
        assert!(message.contains("ambiguous replacement"));
        assert!(message.contains("line 1"));
        assert!(message.contains("line 3"));
        assert_eq!(
            fs::read_to_string(workspace.path().join("ambiguous.txt")).expect("read file"),
            "alpha\nmiddle\nalpha\n"
        );
    }

    #[test]
    fn workspace_edit_diagnostics_attach_formatter_and_diagnostic_outcomes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let editor = LocalWorkspaceEditor::new(workspace.path()).with_edit_evidence_hooks(
            WorkspaceEditEvidenceHooks {
                formatter: Some(WorkspaceEditHookCommand::new(
                    AppliedEditEvidenceKind::Formatter,
                    "rustc",
                    vec!["--version"],
                )),
                diagnostics: Some(WorkspaceEditHookCommand::new(
                    AppliedEditEvidenceKind::Diagnostics,
                    "paddles-missing-diagnostic-command",
                    Vec::<&str>::new(),
                )),
            },
        );

        let result = editor
            .write_file("diagnostics.txt", "hello\n")
            .expect("write file");
        let applied_edit = result.applied_edit.expect("applied edit");

        assert!(applied_edit.evidence.iter().any(|evidence| {
            evidence.kind == AppliedEditEvidenceKind::Formatter
                && evidence.status == AppliedEditEvidenceStatus::Passed
                && evidence.summary.contains("rustc --version")
        }));
        assert!(applied_edit.evidence.iter().any(|evidence| {
            evidence.kind == AppliedEditEvidenceKind::Diagnostics
                && evidence.status == AppliedEditEvidenceStatus::Unavailable
                && evidence.summary.contains("unavailable")
        }));
        assert!(result.summary.contains("formatter: passed"));
        assert!(result.summary.contains("diagnostics: unavailable"));
    }
}
