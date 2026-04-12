use crate::domain::model::{
    AppliedEdit, ExecutionGovernanceOutcome, ExecutionGovernanceOutcomeKind,
    ExecutionHandDescriptor, ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation,
    ExecutionHandPhase, ExecutionPermission, ExecutionPermissionRequest,
    ExecutionPermissionRequirement,
};
use crate::domain::ports::{ExecutionHand, WorkspaceActionResult, WorkspaceEditor};
use crate::infrastructure::execution_governance::{
    ExecutionPermissionGate, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::transport_mediator::TransportToolMediator;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_EDITOR_OUTPUT_CHARS: usize = 12_000;

#[derive(Clone, Debug)]
pub struct LocalWorkspaceEditor {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
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
        }
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

    fn evaluate_request(&self, request: &ExecutionPermissionRequest) -> ExecutionGovernanceOutcome {
        ExecutionPermissionGate::evaluate(
            self.execution_hand_registry.governance_profile().as_ref(),
            request,
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
        let governance = self.evaluate_request(&request);
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
        let governance = self.evaluate_request(&request);
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
            let before = if resolved.exists() {
                fs::read_to_string(&resolved)
                    .with_context(|| format!("failed to read {}", resolved.display()))?
            } else {
                String::new()
            };
            if let Some(parent) = resolved.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("failed to create parent directory {}", parent.display())
                })?;
            }
            fs::write(&resolved, content)
                .with_context(|| format!("failed to write {}", resolved.display()))?;
            let rel = relative_path(&self.workspace_root, &resolved);
            let applied_edit =
                build_applied_edit(&rel, &before, content, &self.transport_mediator)?;
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
        let governance = self.evaluate_request(&request);
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
            let original = fs::read_to_string(&resolved)
                .with_context(|| format!("failed to read {}", resolved.display()))?;
            if !original.contains(old) {
                bail!("pattern not found in {}", resolved.display());
            }
            let updated = if replace_all {
                original.replace(old, new)
            } else {
                original.replacen(old, new, 1)
            };
            fs::write(&resolved, &updated)
                .with_context(|| format!("failed to write {}", resolved.display()))?;
            let rel = relative_path(&self.workspace_root, &resolved);
            let applied_edit =
                build_applied_edit(&rel, &original, &updated, &self.transport_mediator)?;
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
        let governance = self.evaluate_request(&request);
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
            let applied_edit = build_patch_applied_edit(patch);
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

fn resolve_workspace_path(
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

fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::LocalWorkspaceEditor;
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
}
