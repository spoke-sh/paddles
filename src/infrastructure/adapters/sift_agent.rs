use crate::infrastructure::adapters::sift_registry::qwen_spec_for;
use anyhow::{Context, Result, anyhow, bail};
use candle_core::{DType, Device, Tensor};
use candle_nn::{Linear, VarBuilder};
use candle_transformers::models::qwen2::{Config as QwenConfig, Model as QwenModel};
use serde::Deserialize;
use sift::internal::cache::cache_dir;
use sift::internal::search::adapters::llm_utils::{
    QwenConfigPartial, get_device_for, load_mmaped_safetensors_with_repair,
};
use sift::internal::search::adapters::qwen::QwenModelSpec;
use sift::{
    AgentTurnInput, ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse,
    Conversation, EnvironmentFactInput, LocalContextSource, RetainedArtifact, SearchPlan, Sift,
    ToolOutputInput,
};
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use tokenizers::Tokenizer;

const MAX_MODEL_TOKENS: usize = 256;
const MAX_TOOL_STEPS: usize = 4;
const MAX_LOCAL_CONTEXT_ITEMS: usize = 24;
const MAX_TOOL_OUTPUT_CHARS: usize = 12_000;
const MAX_FILE_CHARS: usize = 16_000;
const MAX_LISTED_FILES: usize = 200;
const MAX_CONTEXT_HITS: usize = 5;
const RETAINED_ARTIFACT_LIMIT: usize = 5;
const QWEN_SYSTEM_PROMPT: &str = "<|im_start|>system\nYou are Paddles, a helpful AI assistant and mech suit operator. You provide concise and accurate technical advice.<|im_end|>\n";

pub struct SiftAgentAdapter {
    workspace_root: PathBuf,
    sift: Sift,
    conversation_factory: Box<dyn ConversationFactory>,
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

trait ConversationFactory: Send + Sync {
    fn start_conversation(&self) -> Result<Box<dyn Conversation>>;
}

struct ReusableQwenConversationFactory {
    runtime: Arc<Mutex<PaddlesQwenRuntime>>,
}

impl ReusableQwenConversationFactory {
    fn load(spec: QwenModelSpec) -> Result<Self> {
        Ok(Self {
            runtime: Arc::new(Mutex::new(PaddlesQwenRuntime::load(spec)?)),
        })
    }
}

impl ConversationFactory for ReusableQwenConversationFactory {
    fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
        self.runtime
            .lock()
            .map_err(|_| anyhow!("Qwen runtime lock poisoned"))?
            .reset()?;

        Ok(Box::new(ReusableQwenConversation {
            runtime: Arc::clone(&self.runtime),
            history: Vec::new(),
        }))
    }
}

struct ReusableQwenConversation {
    runtime: Arc<Mutex<PaddlesQwenRuntime>>,
    history: Vec<String>,
}

impl Conversation for ReusableQwenConversation {
    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String> {
        self.history.push(message.to_string());
        let response = self
            .runtime
            .lock()
            .map_err(|_| anyhow!("Qwen runtime lock poisoned"))?
            .send(message, max_tokens)?;
        self.history.push(response.clone());
        Ok(response)
    }

    fn history(&self) -> &[String] {
        &self.history
    }
}

struct PaddlesQwenRuntime {
    session: PaddlesQwenSession,
    tokenizer: Tokenizer,
}

impl PaddlesQwenRuntime {
    fn load(spec: QwenModelSpec) -> Result<Self> {
        let root = cache_dir("models")?
            .join(Path::new(&spec.model_id))
            .join(Path::new(&spec.revision));

        let config_path = root.join("config.json");
        let tokenizer_path = root.join("tokenizer.json");
        let weights_path = root.join("model.safetensors");

        let config_partial: QwenConfigPartial =
            serde_json::from_str(&fs::read_to_string(&config_path)?)?;
        let config = config_partial.into_config()?;
        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|err| anyhow!("failed to load tokenizer: {err}"))?;
        let device = get_device_for("QWEN")?;
        let vb = load_mmaped_safetensors_with_repair(
            &spec.model_id,
            &spec.revision,
            &weights_path,
            DType::F32,
            &device,
        )?;

        Ok(Self {
            session: PaddlesQwenSession::new(&config, &vb, &device, spec.max_length)?,
            tokenizer,
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.session.reset();
        Ok(())
    }

    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String> {
        self.reset()?;
        let prompted = format!(
            "{QWEN_SYSTEM_PROMPT}<|im_start|>user\n{message}<|im_end|>\n<|im_start|>assistant\n"
        );
        self.session
            .generate(&prompted, max_tokens, &self.tokenizer)
    }
}

struct PaddlesQwenSession {
    model: QwenModel,
    lm_head: Linear,
    tokens: Vec<u32>,
    device: Device,
    max_length: usize,
}

impl PaddlesQwenSession {
    fn new(
        config: &QwenConfig,
        vb: &VarBuilder<'static>,
        device: &Device,
        max_length: usize,
    ) -> Result<Self> {
        let model = QwenModel::new(config, vb.clone())?;
        let lm_head = if config.tie_word_embeddings {
            Linear::new(
                vb.pp("model.embed_tokens")
                    .get((config.vocab_size, config.hidden_size), "weight")?,
                None,
            )
        } else {
            candle_nn::linear_no_bias(config.hidden_size, config.vocab_size, vb.pp("lm_head"))?
        };

        Ok(Self {
            model,
            lm_head,
            tokens: Vec::new(),
            device: device.clone(),
            max_length,
        })
    }

    fn reset(&mut self) {
        self.tokens.clear();
        self.model.clear_kv_cache();
    }

    fn select_next_token(logits: &Tensor) -> Result<u32> {
        Ok(logits
            .to_vec1::<f32>()?
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index as u32)
            .unwrap_or_default())
    }

    fn generate(
        &mut self,
        prompt: &str,
        max_tokens: usize,
        tokenizer: &Tokenizer,
    ) -> Result<String> {
        let encoding = tokenizer
            .encode(prompt, self.tokens.is_empty())
            .map_err(|err| anyhow!("encoding failed: {err}"))?;

        let all_new_tokens = encoding.get_ids();
        let available_input_tokens = self
            .max_length
            .saturating_sub(self.tokens.len().saturating_add(max_tokens));
        let new_tokens = if available_input_tokens == 0 {
            &[][..]
        } else if all_new_tokens.len() > available_input_tokens {
            &all_new_tokens[all_new_tokens.len() - available_input_tokens..]
        } else {
            all_new_tokens
        };

        let mut generated_tokens = Vec::new();
        let eos_token_id = tokenizer.token_to_id("<|im_end|>").unwrap_or(151645);
        let mut pending_next_token = None;

        if !new_tokens.is_empty() {
            let tokens_tensor = Tensor::new(new_tokens, &self.device)?.unsqueeze(0)?;
            let pos = self.tokens.len();
            let hidden_states = self.model.forward(&tokens_tensor, pos, None)?;
            self.tokens.extend_from_slice(new_tokens);

            if max_tokens > 0 {
                let last_hidden = hidden_states.narrow(1, new_tokens.len() - 1, 1)?;
                let logits = last_hidden.apply(&self.lm_head)?;
                let last_logit = logits.get(0)?.get(0)?;
                pending_next_token = Some(Self::select_next_token(&last_logit)?);
            }
        } else if max_tokens > 0 && self.tokens.is_empty() {
            return Ok(String::new());
        }

        for _ in 0..max_tokens {
            let next_token = match pending_next_token.take() {
                Some(token) => token,
                None => {
                    let last_token = *self.tokens.last().unwrap();
                    let tokens_tensor = Tensor::new(&[last_token], &self.device)?.unsqueeze(0)?;
                    let hidden_states =
                        self.model
                            .forward(&tokens_tensor, self.tokens.len() - 1, None)?;
                    let last_hidden = hidden_states.narrow(1, 0, 1)?;
                    let logits = last_hidden.apply(&self.lm_head)?;
                    let last_logit = logits.get(0)?.get(0)?;
                    Self::select_next_token(&last_logit)?
                }
            };

            if next_token == eos_token_id || self.tokens.len() >= self.max_length {
                break;
            }

            self.tokens.push(next_token);
            generated_tokens.push(next_token);

            let current_text = tokenizer
                .decode(&generated_tokens, true)
                .map_err(|err| anyhow!("decoding failed: {err}"))?;
            if current_text.contains("<|im_end|>")
                || current_text.contains("Human:")
                || current_text.contains("User:")
                || current_text.contains("<|im_start|>")
            {
                break;
            }
        }

        let mut decoded = tokenizer
            .decode(&generated_tokens, true)
            .map_err(|err| anyhow!("decoding failed: {err}"))?;
        for stop_seq in ["<|im_end|>", "Human:", "User:", "<|im_start|>"] {
            if let Some(index) = decoded.find(stop_seq) {
                decoded.truncate(index);
            }
        }

        Ok(decoded.trim().to_string())
    }
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
        let model = ReusableQwenConversationFactory::load(qwen_spec_for(model_id))?;
        Ok(Self::from_factory(
            workspace_root,
            model_id,
            Box::new(model),
        ))
    }

    fn from_factory(
        workspace_root: PathBuf,
        model_id: &str,
        conversation_factory: Box<dyn ConversationFactory>,
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
            conversation_factory,
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
        Self::from_factory(
            workspace_root.into(),
            model_id,
            Box::new(StaticConversationFactory::new(vec![conversation])),
        )
    }

    #[cfg(test)]
    fn new_for_test_with_conversations(
        workspace_root: impl Into<PathBuf>,
        model_id: &str,
        conversations: Vec<Box<dyn Conversation>>,
    ) -> Self {
        Self::from_factory(
            workspace_root.into(),
            model_id,
            Box::new(StaticConversationFactory::new(conversations)),
        )
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
        let recent_turns = format_recent_turns(&state.local_context);

        let mut working_retained = state.retained_artifacts.clone();
        let mut working_local_context = state.local_context.clone();
        push_local_context(
            &mut working_local_context,
            LocalContextSource::AgentTurn(
                AgentTurnInput::new(&turn_id, "user", prompt).with_session_id(&state.session_id),
            ),
        );
        let casual_turn = is_casual_prompt(prompt);
        let prefer_tools = should_prefer_tools(prompt);
        let follow_up_execution = is_follow_up_execution_request(prompt);
        let mut pending_tool_call = if casual_turn {
            None
        } else {
            infer_tool_call(prompt, &state.local_context)
        };
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut reply = if casual_turn {
            self.send_to_model(conversation.as_mut(), &build_direct_turn_prompt(prompt))?
        } else if pending_tool_call.is_some() {
            String::new()
        } else {
            let initial_local_context = self.combined_local_context(&working_local_context);
            let initial_context =
                self.assemble_context(prompt, None, &initial_local_context, &working_retained)?;
            working_retained = initial_context.retained_artifacts.clone();
            self.log_context_assembly("initial", &initial_context);

            self.send_to_model(
                conversation.as_mut(),
                &build_turn_prompt(
                    &self.workspace_root,
                    prompt,
                    &recent_turns,
                    &initial_context,
                    prefer_tools,
                    follow_up_execution,
                ),
            )?
        };

        if casual_turn {
            if parse_tool_call(&reply)?.is_some() {
                reply =
                    self.send_to_model(conversation.as_mut(), &build_direct_retry_prompt(prompt))?;
            }
            if parse_tool_call(&reply)?.is_some() {
                reply = fallback_casual_reply(prompt);
            }
        } else {
            if prefer_tools && pending_tool_call.is_none() && parse_tool_call(&reply)?.is_none() {
                reply = self.send_to_model(
                    conversation.as_mut(),
                    &build_tool_retry_prompt(prompt, &recent_turns),
                )?;
            }

            for _ in 0..MAX_TOOL_STEPS {
                let (tool_call, controller_inferred) =
                    if let Some(tool_call) = pending_tool_call.take() {
                        (tool_call, true)
                    } else {
                        let Some(tool_call) = parse_tool_call(&reply)? else {
                            break;
                        };
                        (tool_call, false)
                    };

                state.tool_counter += 1;
                let call_id = format!("tool-{}", state.tool_counter);
                let combined_context = self.combined_local_context(&working_local_context);
                let result = match self.execute_tool(
                    &tool_call,
                    &call_id,
                    &combined_context,
                    &working_retained,
                ) {
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

                if controller_inferred {
                    reply = result.summary.clone();
                    break;
                }

                reply = self.send_to_model(
                    conversation.as_mut(),
                    &build_tool_follow_up_prompt(prompt, &call_id, result.name, &result.summary),
                )?;
            }

            if parse_tool_call(&reply)?.is_some() {
                bail!("tool step limit exceeded after {MAX_TOOL_STEPS} tool call(s)");
            }
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

    fn send_to_model(&self, conversation: &mut dyn Conversation, prompt: &str) -> Result<String> {
        let verbose = self.verbose.load(Ordering::Relaxed);

        if verbose >= 1 {
            println!("[INFO] SiftAgentAdapter sending prompt to local model...");
        }
        if verbose >= 3 {
            println!("[TRACE] Prompt payload:\n{prompt}");
        }

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
    recent_turns: &str,
    context: &ContextAssemblyResponse,
    prefer_tools: bool,
    follow_up_execution: bool,
) -> String {
    let routing_guidance = if follow_up_execution {
        "This request refers to a previous turn. Resolve words like `it` or `that` using the recent conversation, then perform the implied action with a tool when possible."
    } else if prefer_tools {
        "This request is action-oriented. If a workspace tool can answer it, call the tool instead of explaining a command the user could run."
    } else {
        "Use tools when they materially improve accuracy; otherwise answer directly."
    };

    format!(
        "You are Paddles, a local-first coding assistant operating inside the workspace `{}`.\n\
Use the provided workspace evidence first. If you have enough information, answer directly.\n\
Final answers should stay concise and focus on the requested result.\n\
Routing guidance: {}\n\
If you need a tool, respond with ONLY a single JSON object and no markdown or prose.\n\
\n\
When the user asks you to inspect repository state, run a command, read a file, search the workspace, or apply a change, prefer a tool call over describing how they could do it themselves.\n\
Examples:\n\
- `show me the git status` -> {{\"tool\":\"shell\",\"command\":\"git status --short\"}}\n\
- `open src/main.rs` -> {{\"tool\":\"read_file\",\"path\":\"src/main.rs\"}}\n\
- `find heartbeat references` -> {{\"tool\":\"search\",\"query\":\"heartbeat references\"}}\n\
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
Recent conversation:\n\
{}\n\
\n\
Current workspace evidence:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        workspace_root.display(),
        routing_guidance,
        recent_turns,
        format_context_digest(context),
        user_prompt
    )
}

fn build_direct_turn_prompt(user_prompt: &str) -> String {
    format!(
        "You are Paddles, a local-first coding assistant.\n\
The user is making a conversational request that does not require workspace tools.\n\
Answer directly in plain text.\n\
Do not emit JSON, code fences, or tool calls.\n\
Do not modify files or suggest workspace actions unless the user explicitly asks for them.\n\
\n\
Current user request:\n\
{user_prompt}\n"
    )
}

fn build_direct_retry_prompt(user_prompt: &str) -> String {
    format!(
        "Your last reply tried to call a workspace tool for a conversational message.\n\
Answer the user directly in plain text.\n\
Do not emit JSON, code fences, or tool calls.\n\
\n\
Current user request:\n\
{user_prompt}\n"
    )
}

fn build_tool_retry_prompt(user_prompt: &str, recent_turns: &str) -> String {
    format!(
        "The user asked for a workspace action and your last reply used prose instead of a tool.\n\
Reply with ONLY one JSON tool call and no prose.\n\
If the request refers to `it` or `that`, resolve it from the recent conversation first.\n\
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
Recent conversation:\n\
{recent_turns}\n\
\n\
Current user request:\n\
{user_prompt}\n"
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

fn format_recent_turns(local_context: &[LocalContextSource]) -> String {
    let turns = local_context
        .iter()
        .rev()
        .filter_map(|item| match item {
            LocalContextSource::AgentTurn(turn) => Some(format!(
                "- {}: {}",
                turn.role,
                trim_for_context(&turn.content, 240)
            )),
            _ => None,
        })
        .take(4)
        .collect::<Vec<_>>();

    if turns.is_empty() {
        return "No prior conversation in this session.".to_string();
    }

    turns.into_iter().rev().collect::<Vec<_>>().join("\n")
}

fn is_casual_prompt(prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);

    matches!(
        normalized.as_str(),
        "hi" | "hello"
            | "hey"
            | "yo"
            | "sup"
            | "whats up"
            | "what's up"
            | "how are you"
            | "who are you"
            | "thanks"
            | "thank you"
            | "good morning"
            | "good afternoon"
            | "good evening"
            | "bye"
            | "goodbye"
    ) || normalized.starts_with("hello ")
        || normalized.starts_with("hi ")
        || normalized.starts_with("hey ")
        || normalized.starts_with("thanks ")
        || normalized.starts_with("thank you ")
}

fn fallback_casual_reply(prompt: &str) -> String {
    let normalized = normalize_prompt(prompt);

    if matches!(normalized.as_str(), "hi" | "hello" | "hey" | "yo") {
        return "Hello.".to_string();
    }
    if matches!(normalized.as_str(), "whats up" | "what's up" | "sup") {
        return "Not much. What do you want to work on?".to_string();
    }
    if normalized == "how are you" {
        return "I am here and ready to help.".to_string();
    }
    if matches!(normalized.as_str(), "thanks" | "thank you") {
        return "You're welcome.".to_string();
    }

    "I am here. What do you want to work on?".to_string()
}

fn should_prefer_tools(prompt: &str) -> bool {
    let normalized = normalize_prompt(prompt);
    if is_casual_prompt(prompt) || is_follow_up_execution_request(prompt) {
        return !is_casual_prompt(prompt);
    }

    normalized.contains("git status")
        || normalized.contains("git diff")
        || normalized.split_whitespace().any(|word| {
            matches!(
                word,
                "run"
                    | "show"
                    | "check"
                    | "inspect"
                    | "list"
                    | "find"
                    | "search"
                    | "open"
                    | "read"
                    | "edit"
                    | "write"
                    | "replace"
                    | "diff"
                    | "patch"
                    | "apply"
                    | "create"
                    | "delete"
            )
        })
}

fn infer_tool_call(prompt: &str, local_context: &[LocalContextSource]) -> Option<ToolCall> {
    infer_shell_command(prompt)
        .map(|command| ToolCall::Shell {
            command: command.to_string(),
        })
        .or_else(|| {
            if is_follow_up_execution_request(prompt) {
                infer_recent_shell_command(local_context).map(|command| ToolCall::Shell {
                    command: command.to_string(),
                })
            } else {
                None
            }
        })
}

fn infer_shell_command(prompt: &str) -> Option<&'static str> {
    let normalized = normalize_prompt(prompt);
    if normalized.contains("git status") {
        return Some("git status");
    }
    if normalized.contains("git diff") {
        return Some("git diff --stat");
    }
    None
}

fn infer_recent_shell_command(local_context: &[LocalContextSource]) -> Option<&'static str> {
    local_context.iter().rev().find_map(|item| match item {
        LocalContextSource::AgentTurn(turn) => infer_shell_command(&turn.content),
        _ => None,
    })
}

fn is_follow_up_execution_request(prompt: &str) -> bool {
    matches!(
        normalize_prompt(prompt).as_str(),
        "run it"
            | "i mean run it"
            | "do it"
            | "i mean do it"
            | "execute it"
            | "i mean execute it"
            | "run that"
            | "execute that"
            | "do that"
    )
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

    if response.starts_with("```") && response.ends_with("```") {
        let inner = response
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();
        if inner.starts_with('{') && inner.ends_with('}') {
            return Some(inner);
        }
    }

    let start = response.find('{')?;
    let end = response.rfind('}')?;
    let candidate = response.get(start..=end)?.trim();
    if candidate.starts_with('{') && candidate.ends_with('}') {
        Some(candidate)
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

fn normalize_prompt(prompt: &str) -> String {
    prompt
        .trim()
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .to_ascii_lowercase()
}

fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
struct StaticConversationFactory {
    conversations: Mutex<std::collections::VecDeque<Box<dyn Conversation>>>,
}

#[cfg(test)]
impl StaticConversationFactory {
    fn new(conversations: Vec<Box<dyn Conversation>>) -> Self {
        Self {
            conversations: Mutex::new(conversations.into()),
        }
    }
}

#[cfg(test)]
impl ConversationFactory for StaticConversationFactory {
    fn start_conversation(&self) -> Result<Box<dyn Conversation>> {
        self.conversations
            .lock()
            .map_err(|_| anyhow!("Static conversation factory lock poisoned"))?
            .pop_front()
            .ok_or_else(|| anyhow!("no test conversation available"))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AgentTurnInput, LocalContextSource, MAX_TOOL_STEPS, SiftAgentAdapter, ToolCall,
        extract_json_payload, infer_tool_call, is_follow_up_execution_request,
        normalize_relative_path, should_prefer_tools, trim_for_context,
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
    fn extracts_tool_call_from_embedded_json() {
        let payload =
            extract_json_payload("Sure, running it now.\n{\"tool\":\"shell\",\"command\":\"pwd\"}")
                .expect("embedded json payload");
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
    fn casual_prompts_retry_for_plain_text_instead_of_executing_tools() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"tool":"replace_in_file","path":"PROTOCOL.md","old":"hello","new":"Hello","replace_all":true}"#
                    .to_string(),
                "Hello.".to_string(),
            ])),
        );

        let reply = adapter.respond("Hello").expect("response");
        assert_eq!(reply, "Hello.");

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 0);
        assert!(
            !state
                .local_context
                .iter()
                .any(|item| matches!(item, LocalContextSource::ToolOutput(_)))
        );
    }

    #[test]
    fn casual_prompts_fall_back_to_plain_text_after_repeated_tool_calls() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"tool":"replace_in_file","path":"PROTOCOL.md","old":"hello","new":"Hello","replace_all":true}"#
                    .to_string(),
                r#"{"tool":"replace_in_file","path":"PROTOCOL.md","old":"hello","new":"Hello","replace_all":true}"#
                    .to_string(),
            ])),
        );

        let reply = adapter.respond("What's up?").expect("response");
        assert_eq!(reply, "Not much. What do you want to work on?");

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 0);
    }

    #[test]
    fn action_requests_prefer_tools() {
        assert!(should_prefer_tools("Can you show me the git status?"));
        assert!(should_prefer_tools("Open src/main.rs"));
        assert!(is_follow_up_execution_request("I mean run it"));
        assert!(!should_prefer_tools("Do you know how to use tools?"));
    }

    #[test]
    fn follow_up_execution_infers_recent_shell_command() {
        let local_context = vec![LocalContextSource::AgentTurn(AgentTurnInput::new(
            "turn-1",
            "assistant",
            "Run `git status` to inspect the repo.",
        ))];

        let inferred = infer_tool_call("I mean run it", &local_context).expect("tool call");
        assert_eq!(
            inferred,
            ToolCall::Shell {
                command: "git status".to_string()
            }
        );
    }

    #[test]
    fn respond_starts_a_fresh_conversation_each_turn() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test_with_conversations(
            workspace.path(),
            "qwen-1.5b",
            vec![
                Box::new(MockConversation::new(vec!["Hello.".to_string()])),
                Box::new(MockConversation::new(vec![
                    r#"{"tool":"shell","command":"git status --short"}"#.to_string(),
                    "Working tree is clean.".to_string(),
                ])),
            ],
        );

        let first = adapter.respond("Hello").expect("first response");
        let second = adapter
            .respond("Inspect the repository status")
            .expect("second response");

        assert_eq!(first, "Hello.");
        assert_eq!(second, "Working tree is clean.");
    }

    #[test]
    fn action_prompts_retry_for_tool_calls_after_prose() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(workspace.path())
            .status()
            .expect("git init");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "You can run `git status` to inspect the working tree.".to_string(),
                r#"{"tool":"shell","command":"git status"}"#.to_string(),
                "Working tree is clean.".to_string(),
            ])),
        );

        let reply = adapter
            .respond("Inspect the repository status")
            .expect("response");

        assert_eq!(reply, "Working tree is clean.");
        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 1);
    }

    #[test]
    fn action_prompts_can_use_controller_inferred_tool_calls() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        std::process::Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(workspace.path())
            .status()
            .expect("git init");

        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "Working tree is clean.".to_string(),
            ])),
        );

        let reply = adapter.respond("Show me the git status").expect("response");

        assert!(reply.contains("git status"));
        assert!(reply.contains("Exit status"));
        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 1);
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
