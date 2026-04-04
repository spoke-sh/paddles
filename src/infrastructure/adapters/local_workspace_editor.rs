use crate::domain::ports::{WorkspaceActionResult, WorkspaceEditor};
use anyhow::{Context, Result, anyhow, bail};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Output, Stdio};

const MAX_EDITOR_OUTPUT_CHARS: usize = 12_000;

#[derive(Clone, Debug)]
pub struct LocalWorkspaceEditor {
    workspace_root: PathBuf,
}

impl LocalWorkspaceEditor {
    pub fn new(workspace_root: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
        }
    }
}

impl WorkspaceEditor for LocalWorkspaceEditor {
    fn diff(&self, path: Option<&str>) -> Result<WorkspaceActionResult> {
        let mut command = Command::new("git");
        command
            .arg("diff")
            .arg("--no-ext-diff")
            .current_dir(&self.workspace_root);
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
        })
    }

    fn write_file(&self, path: &str, content: &str) -> Result<WorkspaceActionResult> {
        let resolved = resolve_workspace_path(&self.workspace_root, path, true)?;
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("failed to create parent directory {}", parent.display())
            })?;
        }
        fs::write(&resolved, content)
            .with_context(|| format!("failed to write {}", resolved.display()))?;
        Ok(WorkspaceActionResult {
            name: "write_file".to_string(),
            summary: format!(
                "Wrote {} byte(s) to {}.",
                content.len(),
                relative_path(&self.workspace_root, &resolved)
            ),
        })
    }

    fn replace_in_file(
        &self,
        path: &str,
        old: &str,
        new: &str,
        replace_all: bool,
    ) -> Result<WorkspaceActionResult> {
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
        fs::write(&resolved, updated)
            .with_context(|| format!("failed to write {}", resolved.display()))?;
        Ok(WorkspaceActionResult {
            name: "replace_in_file".to_string(),
            summary: format!(
                "Updated {} by replacing {} occurrence(s) of the requested text.",
                relative_path(&self.workspace_root, &resolved),
                if replace_all { "all" } else { "one" }
            ),
        })
    }

    fn apply_patch(&self, patch: &str) -> Result<WorkspaceActionResult> {
        let mut child = Command::new("git")
            .arg("apply")
            .arg("--whitespace=nowarn")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .current_dir(&self.workspace_root)
            .spawn()
            .context("failed to spawn git apply")?;

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
        Ok(WorkspaceActionResult {
            name: "apply_patch".to_string(),
            summary,
        })
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

fn relative_path(workspace_root: &Path, path: &Path) -> String {
    path.strip_prefix(workspace_root)
        .unwrap_or(path)
        .display()
        .to_string()
}
