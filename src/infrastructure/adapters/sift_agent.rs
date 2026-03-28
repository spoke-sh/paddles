use crate::infrastructure::adapters::sift_registry::qwen_spec_for;
use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use sift::internal::search::adapters::qwen::QwenReranker;
use sift::{
    AgentTurnInput, ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse,
    Conversation, EnvironmentFactInput, GenerativeModel, LocalContextSource, RetainedArtifact,
    SearchPlan, Sift, ToolOutputInput,
};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_MODEL_TOKENS: usize = 256;
const MAX_TOOL_STEPS: usize = 4;
const MAX_LOCAL_CONTEXT_ITEMS: usize = 24;
const MAX_TOOL_OUTPUT_CHARS: usize = 12_000;
const MAX_FILE_CHARS: usize = 16_000;
const MAX_LISTED_FILES: usize = 200;
const MAX_CONTEXT_HITS: usize = 5;
const RETAINED_ARTIFACT_LIMIT: usize = 5;

pub struct SiftAgentAdapter {
    workspace_root: PathBuf,
    sift: Sift,
    conversation: Mutex<Box<dyn Conversation>>,
    base_context: Vec<LocalContextSource>,
    state: Mutex<SessionState>,
    verbose: AtomicU8,
}

#[derive(Clone)]
struct SessionState {
    session_id: String,
    turn_counter: usize,
    tool_counter: usize,
    retained_artifacts: Vec<RetainedArtifact>,
    local_context: Vec<LocalContextSource>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "tool", rename_all = "snake_case")]
enum ToolCall {
    Search {
        query: String,
        #[serde(default)]
        intent: Option<String>,
    },
    ListFiles {
        #[serde(default)]
        pattern: Option<String>,
    },
    ReadFile {
        path: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    ReplaceInFile {
        path: String,
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
    },
    Shell {
        command: String,
    },
    Diff {
        #[serde(default)]
        path: Option<String>,
    },
    ApplyPatch {
        patch: String,
    },
}

#[derive(Debug)]
struct ToolResult {
    name: &'static str,
    summary: String,
    retained_artifacts: Option<Vec<RetainedArtifact>>,
}

impl ToolCall {
    fn name(&self) -> &'static str {
        match self {
            Self::Search { .. } => "search",
            Self::ListFiles { .. } => "list_files",
            Self::ReadFile { .. } => "read_file",
            Self::WriteFile { .. } => "write_file",
            Self::ReplaceInFile { .. } => "replace_in_file",
            Self::Shell { .. } => "shell",
            Self::Diff { .. } => "diff",
            Self::ApplyPatch { .. } => "apply_patch",
        }
    }
}

impl SiftAgentAdapter {
    pub fn new(workspace_root: impl Into<PathBuf>, model_id: &str) -> Result<Self> {
        let workspace_root = workspace_root.into();
        let model = QwenReranker::load(qwen_spec_for(model_id))?;
        let conversation = model.start_conversation()?;
        Ok(Self::from_conversation(
            workspace_root,
            model_id,
            conversation,
        ))
    }

    fn from_conversation(
        workspace_root: PathBuf,
        model_id: &str,
        conversation: Box<dyn Conversation>,
    ) -> Self {
        let session_id = format!("paddles-{}", unix_timestamp());
        let base_context = vec![
            LocalContextSource::EnvironmentFact(EnvironmentFactInput::new(
                "workspace_root",
                workspace_root.display().to_string(),
            )),
            LocalContextSource::EnvironmentFact(EnvironmentFactInput::new("model_id", model_id)),
            LocalContextSource::EnvironmentFact(EnvironmentFactInput::new(
                "runtime",
                "sift-native",
            )),
        ];

        Self {
            workspace_root: workspace_root.clone(),
            sift: Sift::builder().build(),
            conversation: Mutex::new(conversation),
            base_context,
            state: Mutex::new(SessionState {
                session_id,
                turn_counter: 0,
                tool_counter: 0,
                retained_artifacts: Vec::new(),
                local_context: Vec::new(),
            }),
            verbose: AtomicU8::new(0),
        }
    }

    #[cfg(test)]
    fn new_for_test(
        workspace_root: impl Into<PathBuf>,
        model_id: &str,
        conversation: Box<dyn Conversation>,
    ) -> Self {
        Self::from_conversation(workspace_root.into(), model_id, conversation)
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    pub fn respond(&self, prompt: &str) -> Result<String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("Sift session state lock poisoned"))?;

        state.turn_counter += 1;
        let turn_id = format!("turn-{}", state.turn_counter);
        let assistant_turn_id = format!("{turn_id}-assistant");

        let mut working_retained = state.retained_artifacts.clone();
        let mut working_local_context = state.local_context.clone();
        push_local_context(
            &mut working_local_context,
            LocalContextSource::AgentTurn(
                AgentTurnInput::new(&turn_id, "user", prompt).with_session_id(&state.session_id),
            ),
        );
        let initial_local_context = self.combined_local_context(&working_local_context);

        let initial_context =
            self.assemble_context(prompt, None, &initial_local_context, &working_retained)?;
        working_retained = initial_context.retained_artifacts.clone();
        self.log_context_assembly("initial", &initial_context);

        let mut reply = self.send_to_model(&build_turn_prompt(
            &self.workspace_root,
            prompt,
            &initial_context,
        ))?;

        for _ in 0..MAX_TOOL_STEPS {
            let Some(tool_call) = parse_tool_call(&reply)? else {
                break;
            };

            state.tool_counter += 1;
            let call_id = format!("tool-{}", state.tool_counter);
            let combined_context = self.combined_local_context(&working_local_context);
            let result =
                match self.execute_tool(&tool_call, &call_id, &combined_context, &working_retained)
                {
                    Ok(result) => result,
                    Err(err) => ToolResult {
                        name: tool_call.name(),
                        summary: format!("Tool `{}` failed: {err:#}", tool_call.name()),
                        retained_artifacts: None,
                    },
                };

            if let Some(retained) = result.retained_artifacts {
                working_retained = retained;
            }

            push_local_context(
                &mut working_local_context,
                LocalContextSource::ToolOutput(ToolOutputInput::new(
                    result.name,
                    &call_id,
                    result.summary.clone(),
                )),
            );

            reply = self.send_to_model(&build_tool_follow_up_prompt(
                prompt,
                &call_id,
                result.name,
                &result.summary,
            ))?;
        }

        if parse_tool_call(&reply)?.is_some() {
            bail!("tool step limit exceeded after {MAX_TOOL_STEPS} tool call(s)");
        }

        push_local_context(
            &mut working_local_context,
            LocalContextSource::AgentTurn(
                AgentTurnInput::new(&assistant_turn_id, "assistant", &reply)
                    .with_session_id(&state.session_id),
            ),
        );

        state.retained_artifacts = working_retained;
        state.local_context = working_local_context;

        Ok(reply)
    }

    fn combined_local_context(
        &self,
        rolling_context: &[LocalContextSource],
    ) -> Vec<LocalContextSource> {
        let mut combined = self.base_context.clone();
        combined.extend_from_slice(rolling_context);
        combined
    }

    fn send_to_model(&self, prompt: &str) -> Result<String> {
        let verbose = self.verbose.load(Ordering::Relaxed);

        if verbose >= 1 {
            println!("[INFO] SiftAgentAdapter sending prompt to local model...");
        }
        if verbose >= 3 {
            println!("[TRACE] Prompt payload:\n{prompt}");
        }

        let mut conversation = self
            .conversation
            .lock()
            .map_err(|_| anyhow!("Sift conversation lock poisoned"))?;

        let response = conversation.send(prompt, MAX_MODEL_TOKENS)?;

        if verbose >= 2 {
            println!("[DEBUG] Model response: {}", response);
        }

        Ok(response)
    }

    fn log_context_assembly(&self, label: &str, response: &ContextAssemblyResponse) {
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            println!(
                "[INFO] Context assembly ({label}) produced {} hit(s), retained {} artifact(s), pruned {}.",
                response.response.hits.len(),
                response.retained_artifacts.len(),
                response.pruned_artifacts
            );
        }
    }

    fn assemble_context(
        &self,
        query: &str,
        intent: Option<String>,
        local_context: &[LocalContextSource],
        retained_artifacts: &[RetainedArtifact],
    ) -> Result<ContextAssemblyResponse> {
        self.sift.assemble_context(
            ContextAssemblyRequest::new(&self.workspace_root, query)
                .with_plan(SearchPlan::default_lexical())
                .with_intent_opt(intent)
                .with_limit(MAX_CONTEXT_HITS)
                .with_shortlist(MAX_CONTEXT_HITS)
                .with_budget(ContextAssemblyBudget::new(RETAINED_ARTIFACT_LIMIT))
                .with_local_context(local_context.to_vec())
                .with_retained_artifacts(retained_artifacts.to_vec()),
        )
    }

    fn execute_tool(
        &self,
        tool_call: &ToolCall,
        call_id: &str,
        local_context: &[LocalContextSource],
        retained_artifacts: &[RetainedArtifact],
    ) -> Result<ToolResult> {
        let verbose = self.verbose.load(Ordering::Relaxed);
        if verbose >= 1 {
            println!("[INFO] Executing tool call {call_id}: {:?}", tool_call);
        }

        match tool_call {
            ToolCall::Search { query, intent } => {
                let assembly = self.assemble_context(
                    query,
                    intent.clone(),
                    local_context,
                    retained_artifacts,
                )?;
                self.log_context_assembly("search", &assembly);
                Ok(ToolResult {
                    name: "search",
                    summary: format_search_summary(query, &assembly),
                    retained_artifacts: Some(assembly.retained_artifacts),
                })
            }
            ToolCall::ListFiles { pattern } => {
                let files = list_files(&self.workspace_root, pattern.as_deref())?;
                let summary = if files.is_empty() {
                    "No matching files found.".to_string()
                } else {
                    format!("Listed {} file(s):\n{}", files.len(), files.join("\n"))
                };
                Ok(ToolResult {
                    name: "list_files",
                    summary: trim_for_context(&summary, MAX_TOOL_OUTPUT_CHARS),
                    retained_artifacts: None,
                })
            }
            ToolCall::ReadFile { path } => {
                let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
                let content = fs::read(&resolved)
                    .with_context(|| format!("failed to read {}", resolved.display()))?;
                let content = String::from_utf8_lossy(&content).to_string();
                let rel = relative_path(&self.workspace_root, &resolved);
                let summary = format!(
                    "Read file {rel}:\n{}",
                    trim_for_context(&content, MAX_FILE_CHARS)
                );
                Ok(ToolResult {
                    name: "read_file",
                    summary,
                    retained_artifacts: None,
                })
            }
            ToolCall::WriteFile { path, content } => {
                let resolved = resolve_workspace_path(&self.workspace_root, path, true)?;
                if let Some(parent) = resolved.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!("failed to create parent directory {}", parent.display())
                    })?;
                }
                fs::write(&resolved, content)
                    .with_context(|| format!("failed to write {}", resolved.display()))?;
                Ok(ToolResult {
                    name: "write_file",
                    summary: format!(
                        "Wrote {} byte(s) to {}.",
                        content.len(),
                        relative_path(&self.workspace_root, &resolved)
                    ),
                    retained_artifacts: None,
                })
            }
            ToolCall::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => {
                let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
                let original = fs::read_to_string(&resolved)
                    .with_context(|| format!("failed to read {}", resolved.display()))?;
                if !original.contains(old) {
                    bail!("pattern not found in {}", resolved.display());
                }
                let updated = if *replace_all {
                    original.replace(old, new)
                } else {
                    original.replacen(old, new, 1)
                };
                fs::write(&resolved, updated)
                    .with_context(|| format!("failed to write {}", resolved.display()))?;
                Ok(ToolResult {
                    name: "replace_in_file",
                    summary: format!(
                        "Updated {} by replacing {} occurrence(s) of the requested text.",
                        relative_path(&self.workspace_root, &resolved),
                        if *replace_all { "all" } else { "one" }
                    ),
                    retained_artifacts: None,
                })
            }
            ToolCall::Shell { command } => {
                let output = Command::new("sh")
                    .arg("-lc")
                    .arg(command)
                    .current_dir(&self.workspace_root)
                    .output()
                    .with_context(|| format!("failed to execute shell command `{command}`"))?;
                let summary = format_command_summary("Shell command", command, &output);
                if !output.status.success() {
                    bail!("{summary}");
                }
                Ok(ToolResult {
                    name: "shell",
                    summary,
                    retained_artifacts: None,
                })
            }
            ToolCall::Diff { path } => {
                let mut command = Command::new("git");
                command
                    .arg("diff")
                    .arg("--no-ext-diff")
                    .current_dir(&self.workspace_root);
                if let Some(path) = path {
                    let resolved = resolve_workspace_path(&self.workspace_root, path, false)?;
                    let rel = relative_path(&self.workspace_root, &resolved);
                    command.arg("--").arg(rel);
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
                        trim_for_context(&diff, MAX_TOOL_OUTPUT_CHARS),
                        trim_for_context(&stderr, MAX_TOOL_OUTPUT_CHARS / 2)
                    )
                };
                Ok(ToolResult {
                    name: "diff",
                    summary,
                    retained_artifacts: None,
                })
            }
            ToolCall::ApplyPatch { patch } => {
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
                    format_command_summary("git apply", "git apply --whitespace=nowarn -", &output);
                if !output.status.success() {
                    bail!("{summary}");
                }
                Ok(ToolResult {
                    name: "apply_patch",
                    summary,
                    retained_artifacts: None,
                })
            }
        }
    }
}

fn build_turn_prompt(
    workspace_root: &Path,
    user_prompt: &str,
    context: &ContextAssemblyResponse,
) -> String {
    format!(
        "You are Paddles, a local-first coding assistant operating inside the workspace `{}`.\n\
Use the provided workspace evidence first. If you have enough information, answer directly.\n\
Final answers should stay concise and focus on the requested result.\n\
If you need a tool, respond with ONLY a single JSON object and no markdown or prose.\n\
\n\
Available tools:\n\
- {{\"tool\":\"search\",\"query\":\"...\",\"intent\":\"optional\"}}\n\
- {{\"tool\":\"list_files\",\"pattern\":\"optional substring\"}}\n\
- {{\"tool\":\"read_file\",\"path\":\"relative/path\"}}\n\
- {{\"tool\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\"}}\n\
- {{\"tool\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false}}\n\
- {{\"tool\":\"shell\",\"command\":\"local shell command\"}}\n\
- {{\"tool\":\"diff\",\"path\":\"optional relative/path\"}}\n\
- {{\"tool\":\"apply_patch\",\"patch\":\"unified diff text\"}}\n\
\n\
Current workspace evidence:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        workspace_root.display(),
        format_context_digest(context),
        user_prompt
    )
}

fn build_tool_follow_up_prompt(
    user_prompt: &str,
    call_id: &str,
    tool_name: &str,
    summary: &str,
) -> String {
    format!(
        "Tool call {call_id} ({tool_name}) completed.\n\
Tool result:\n\
{summary}\n\
\n\
Original user request:\n\
{user_prompt}\n\
\n\
If you need another tool, respond with ONLY one JSON tool call.\n\
Otherwise answer the user directly and concisely."
    )
}

fn format_context_digest(context: &ContextAssemblyResponse) -> String {
    if context.response.hits.is_empty() {
        return "No relevant workspace context was assembled.".to_string();
    }

    let mut lines = vec![format!(
        "Retained artifacts: {} (pruned: {})",
        context.retained_artifacts.len(),
        context.pruned_artifacts
    )];

    for hit in context.response.hits.iter().take(MAX_CONTEXT_HITS) {
        let location = hit
            .location
            .as_ref()
            .map(|value| format!(" @ {value}"))
            .unwrap_or_default();
        lines.push(format!(
            "- {}{}: {}",
            hit.path,
            location,
            trim_for_context(&hit.snippet, 280)
        ));
    }

    lines.join("\n")
}

fn format_search_summary(query: &str, assembly: &ContextAssemblyResponse) -> String {
    if assembly.response.hits.is_empty() {
        return format!("Search `{query}` returned no matching hits.");
    }

    let mut lines = vec![format!(
        "Search `{query}` returned {} hit(s); retained {} artifact(s), pruned {}.",
        assembly.response.hits.len(),
        assembly.retained_artifacts.len(),
        assembly.pruned_artifacts
    )];

    for hit in assembly.response.hits.iter().take(MAX_CONTEXT_HITS) {
        let location = hit
            .location
            .as_ref()
            .map(|value| format!(" @ {value}"))
            .unwrap_or_default();
        lines.push(format!(
            "- {}{}: {}",
            hit.path,
            location,
            trim_for_context(&hit.snippet, 320)
        ));
    }

    lines.join("\n")
}

fn parse_tool_call(response: &str) -> Result<Option<ToolCall>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

fn extract_json_payload(response: &str) -> Option<&str> {
    if response.starts_with('{') && response.ends_with('}') {
        return Some(response);
    }

    if !response.starts_with("```") || !response.ends_with("```") {
        return None;
    }

    let inner = response
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();
    if inner.starts_with('{') && inner.ends_with('}') {
        Some(inner)
    } else {
        None
    }
}

fn push_local_context(context: &mut Vec<LocalContextSource>, item: LocalContextSource) {
    context.push(item);
    if context.len() > MAX_LOCAL_CONTEXT_ITEMS {
        let overflow = context.len() - MAX_LOCAL_CONTEXT_ITEMS;
        context.drain(0..overflow);
    }
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

fn list_files(workspace_root: &Path, pattern: Option<&str>) -> Result<Vec<String>> {
    let mut files = Vec::new();
    visit_files(workspace_root, workspace_root, pattern, &mut files)?;
    files.sort();
    if files.len() > MAX_LISTED_FILES {
        files.truncate(MAX_LISTED_FILES);
    }
    Ok(files)
}

fn visit_files(
    dir: &Path,
    workspace_root: &Path,
    pattern: Option<&str>,
    files: &mut Vec<String>,
) -> Result<()> {
    if files.len() >= MAX_LISTED_FILES {
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            if matches!(name.as_ref(), ".git" | "target" | ".direnv") {
                continue;
            }
            visit_files(&path, workspace_root, pattern, files)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let rel = relative_path(workspace_root, &path);
        if pattern.is_none_or(|needle| rel.contains(needle)) {
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

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{
        LocalContextSource, MAX_TOOL_STEPS, SiftAgentAdapter, ToolCall, extract_json_payload,
        normalize_relative_path, trim_for_context,
    };
    use anyhow::{Result, anyhow};
    use sift::Conversation;
    use std::collections::VecDeque;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    use std::path::{Path, PathBuf};

    struct MockConversation {
        responses: VecDeque<String>,
        history: Vec<String>,
    }

    impl MockConversation {
        fn new(responses: Vec<String>) -> Self {
            Self {
                responses: VecDeque::from(responses),
                history: Vec::new(),
            }
        }
    }

    impl Conversation for MockConversation {
        fn send(&mut self, message: &str, _max_tokens: usize) -> Result<String> {
            self.history.push(message.to_string());
            self.responses
                .pop_front()
                .ok_or_else(|| anyhow!("mock conversation exhausted"))
        }

        fn history(&self) -> &[String] {
            &self.history
        }
    }

    #[test]
    fn extracts_tool_call_from_raw_json() {
        let payload = extract_json_payload("{\"tool\":\"read_file\",\"path\":\"src/main.rs\"}")
            .expect("json payload");
        let parsed: ToolCall = serde_json::from_str(payload).expect("tool call");
        assert_eq!(
            parsed,
            ToolCall::ReadFile {
                path: "src/main.rs".to_string()
            }
        );
    }

    #[test]
    fn extracts_tool_call_from_fenced_json() {
        let payload =
            extract_json_payload("```json\n{\"tool\":\"shell\",\"command\":\"pwd\"}\n```")
                .expect("fenced json payload");
        let parsed: ToolCall = serde_json::from_str(payload).expect("tool call");
        assert_eq!(
            parsed,
            ToolCall::Shell {
                command: "pwd".to_string()
            }
        );
    }

    #[test]
    fn normalizes_relative_paths_without_leaving_workspace() {
        let root = PathBuf::from("/workspace/project");
        let normalized = normalize_relative_path(&root, Path::new("./src/../src/main.rs"));
        assert_eq!(normalized, PathBuf::from("/workspace/project/src/main.rs"));
    }

    #[test]
    fn trims_large_context_payloads() {
        let input = "a".repeat(40);
        let trimmed = trim_for_context(&input, 10);
        assert!(trimmed.starts_with("aaaaaaaaaa"));
        assert!(trimmed.contains("[truncated]"));
    }

    #[test]
    fn respond_records_tool_outputs_and_turns_in_local_context() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::create_dir_all(workspace.path().join("src")).expect("create src");
        fs::write(workspace.path().join("src/main.rs"), "fn main() {}\n").expect("write file");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"tool":"list_files","pattern":"main.rs"}"#.to_string(),
                "The entrypoint is src/main.rs.".to_string(),
            ])),
        );

        let reply = adapter
            .respond("Where is the entrypoint?")
            .expect("response");
        assert_eq!(reply, "The entrypoint is src/main.rs.");

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 1);
        assert!(state.local_context.iter().any(|item| matches!(
            item,
            LocalContextSource::ToolOutput(output)
                if output.tool_name == "list_files" && output.content.contains("src/main.rs")
        )));
        assert!(state.local_context.iter().any(|item| matches!(
            item,
            LocalContextSource::AgentTurn(turn)
                if turn.role == "user" && turn.content == "Where is the entrypoint?"
        )));
        assert!(state.local_context.iter().any(|item| matches!(
            item,
            LocalContextSource::AgentTurn(turn)
                if turn.role == "assistant" && turn.content == "The entrypoint is src/main.rs."
        )));
    }

    #[test]
    fn search_tool_uses_sift_context_assembly() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::write(
            workspace.path().join("guide.txt"),
            "telemetry waterfall architecture\n",
        )
        .expect("write guide");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let result = adapter
            .execute_tool(
                &ToolCall::Search {
                    query: "telemetry".to_string(),
                    intent: None,
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
            )
            .expect("search tool");

        assert_eq!(result.name, "search");
        assert!(result.summary.contains("guide.txt"));
        assert!(result.retained_artifacts.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn read_file_rejects_symlink_escape() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let outside = tempfile::tempdir().expect("outside workspace");
        fs::write(outside.path().join("secret.txt"), "classified\n").expect("write secret");
        unix_fs::symlink(outside.path(), workspace.path().join("vault")).expect("create symlink");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let err = adapter
            .execute_tool(
                &ToolCall::ReadFile {
                    path: "vault/secret.txt".to_string(),
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
            )
            .expect_err("symlink escape should fail");

        assert!(err.to_string().contains("path escapes workspace root"));
    }

    #[cfg(unix)]
    #[test]
    fn list_files_skips_symlinked_directories() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let outside = tempfile::tempdir().expect("outside workspace");
        fs::write(outside.path().join("secret.txt"), "classified\n").expect("write secret");
        unix_fs::symlink(outside.path(), workspace.path().join("vault")).expect("create symlink");
        fs::write(workspace.path().join("main.rs"), "fn main() {}\n")
            .expect("write workspace file");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let result = adapter
            .execute_tool(
                &ToolCall::ListFiles { pattern: None },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
            )
            .expect("list files");

        assert!(result.summary.contains("main.rs"));
        assert!(!result.summary.contains("secret.txt"));
        assert!(!result.summary.contains("vault"));
    }

    #[test]
    fn tool_failures_are_recorded_and_can_recover() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"tool":"read_file","path":"missing.txt"}"#.to_string(),
                "Recovered after the failed read.".to_string(),
            ])),
        );

        let reply = adapter
            .respond("Try reading the missing file.")
            .expect("response");
        assert_eq!(reply, "Recovered after the failed read.");

        let state = adapter.state.lock().expect("state");
        assert!(state.local_context.iter().any(|item| matches!(
            item,
            LocalContextSource::ToolOutput(output)
                if output.tool_name == "read_file" && output.content.contains("failed")
        )));
    }

    #[test]
    fn shell_tool_returns_error_on_non_zero_exit() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let err = adapter
            .execute_tool(
                &ToolCall::Shell {
                    command: "exit 7".to_string(),
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
            )
            .expect_err("shell failure should propagate");

        assert!(err.to_string().contains("Exit status"));
        assert!(err.to_string().contains("7"));
    }

    #[test]
    fn apply_patch_returns_error_on_failure() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let err = adapter
            .execute_tool(
                &ToolCall::ApplyPatch {
                    patch: "not a patch".to_string(),
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
            )
            .expect_err("apply_patch failure should propagate");

        assert!(err.to_string().contains("git apply"));
        assert!(err.to_string().contains("Exit status"));
    }

    #[test]
    fn exhausting_the_tool_budget_returns_an_error() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let responses = (0..=MAX_TOOL_STEPS)
            .map(|_| r#"{"tool":"list_files"}"#.to_string())
            .collect::<Vec<_>>();
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(responses)),
        );

        let err = adapter
            .respond("Keep listing files forever.")
            .expect_err("tool budget error");
        assert!(err.to_string().contains("tool step limit exceeded"));
    }
}
