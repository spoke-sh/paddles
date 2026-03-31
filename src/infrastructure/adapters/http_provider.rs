use crate::domain::model::{
    CompactionDecision, CompactionPlan, CompactionRequest, ThreadDecision, ThreadDecisionId,
    ThreadDecisionKind, TurnEvent, TurnEventSink, TurnIntent,
};
use crate::domain::ports::{
    EvidenceBundle, InitialAction, InitialActionDecision, InterpretationContext,
    InterpretationRequest, PlannerAction, PlannerCapability, PlannerRequest,
    RecursivePlannerDecision, RetrievalMode, RetrievalStrategy, SynthesizerEngine,
    ThreadDecisionRequest, WorkspaceAction, WorkspaceActionResult,
};
use crate::infrastructure::rendering::{
    ANTHROPIC_RENDER_TOOL_NAME, RenderCapability, assistant_response_json_schema,
    ensure_citation_section, final_answer_contract_prompt, normalize_assistant_response,
};
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

const MAX_CITATIONS: usize = 4;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 2000;

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error()
}

/// Check an HTTP response, retrying the request on transient 429/5xx errors.
async fn send_with_retry(
    provider: &str,
    build_request: impl Fn() -> reqwest::RequestBuilder,
) -> Result<String> {
    for attempt in 0..=MAX_RETRIES {
        let resp = build_request().send().await?;
        let status = resp.status();
        let text = resp.text().await?;
        if status.is_success() {
            return Ok(text);
        }
        if !is_retryable_status(status) || attempt == MAX_RETRIES {
            bail!("{provider} API error {status}: {text}");
        }
        let delay = RETRY_BASE_DELAY_MS * 2u64.pow(attempt);
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    unreachable!()
}

/// Which HTTP API format to use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiFormat {
    OpenAi,
    Anthropic,
    Gemini,
}

/// HTTP-based model provider implementing SynthesizerEngine.
pub struct HttpProviderAdapter {
    workspace_root: PathBuf,
    client: reqwest::Client,
    api_key: String,
    base_url: String,
    model_id: String,
    format: ApiFormat,
    render_capability: RenderCapability,
    verbose: AtomicU8,
    turn_history: Mutex<Vec<String>>,
}

impl HttpProviderAdapter {
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        model_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        format: ApiFormat,
        render_capability: RenderCapability,
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            model_id: model_id.into(),
            format,
            render_capability,
            verbose: AtomicU8::new(0),
            turn_history: Mutex::new(Vec::new()),
        }
    }

    async fn send_async(&self, system: &str, user: &str) -> Result<String> {
        let verbose = self.verbose.load(Ordering::Relaxed);
        if verbose >= 2 {
            eprintln!("[HTTP] Sending to {} ({})", self.base_url, self.model_id);
        }
        if verbose >= 3 {
            eprintln!("[HTTP] System: {system}");
            eprintln!("[HTTP] User: {user}");
        }

        let response = match self.format {
            ApiFormat::OpenAi => self.send_openai(system, user).await?,
            ApiFormat::Anthropic => self.send_anthropic(system, user).await?,
            ApiFormat::Gemini => self.send_gemini(system, user).await?,
        };

        if verbose >= 2 {
            eprintln!(
                "[HTTP] Response: {}",
                if response.len() > 200 {
                    format!("{}...", &response[..200])
                } else {
                    response.clone()
                }
            );
        }

        Ok(response)
    }

    fn send_structured_answer_blocking(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
    ) -> Result<String> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow!("no tokio runtime for HTTP provider"))?;
        tokio::task::block_in_place(|| {
            rt.block_on(self.send_structured_answer_async(system, user, require_citations))
        })
    }

    async fn send_structured_answer_async(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
    ) -> Result<String> {
        match self.render_capability {
            RenderCapability::OpenAiJsonSchema => {
                self.send_openai_structured_answer(system, user, require_citations)
                    .await
            }
            RenderCapability::AnthropicToolUse => {
                self.send_anthropic_structured_answer(system, user, require_citations)
                    .await
            }
            RenderCapability::GeminiJsonSchema => {
                self.send_gemini_structured_answer(system, user, require_citations)
                    .await
            }
            RenderCapability::PromptEnvelope => self.send_async(system, user).await,
        }
    }

    async fn send_openai(&self, system: &str, user: &str) -> Result<String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = serde_json::json!({
            "model": self.model_id,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "max_tokens": 4096,
        });

        let api_key = self.api_key.clone();
        let text = send_with_retry("OpenAI", || {
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: OpenAiResponse = serde_json::from_str(&text)?;
        parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty OpenAI response"))
    }

    async fn send_openai_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
    ) -> Result<String> {
        let url = format!(
            "{}/v1/chat/completions",
            self.base_url.trim_end_matches('/')
        );
        let body = json!({
            "model": self.model_id,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
            "max_tokens": 4096,
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "assistant_response",
                    "strict": true,
                    "schema": assistant_response_json_schema(require_citations),
                }
            }
        });

        let api_key = self.api_key.clone();
        let text = send_with_retry("OpenAI", || {
            self.client
                .post(&url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: OpenAiResponse = serde_json::from_str(&text)?;
        parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty OpenAI response"))
    }

    async fn send_anthropic(&self, system: &str, user: &str) -> Result<String> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model_id,
            "max_tokens": 4096,
            "system": system,
            "messages": [
                { "role": "user", "content": user },
            ],
        });

        let api_key = self.api_key.clone();
        let text = send_with_retry("Anthropic", || {
            self.client
                .post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: AnthropicResponse = serde_json::from_str(&text)?;
        parsed
            .content
            .first()
            .and_then(|b| b.text.clone())
            .ok_or_else(|| anyhow!("empty Anthropic response"))
    }

    async fn send_anthropic_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
    ) -> Result<String> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let body = json!({
            "model": self.model_id,
            "max_tokens": 4096,
            "system": system,
            "messages": [
                { "role": "user", "content": user },
            ],
            "tools": [json!({
                "name": ANTHROPIC_RENDER_TOOL_NAME,
                "description": "Return the final answer as structured render blocks for the Paddles transcript. Use it exactly once per response and include only the content needed for the final rendered answer.",
                "input_schema": assistant_response_json_schema(require_citations),
                "strict": true
            })],
            "tool_choice": {
                "type": "tool",
                "name": ANTHROPIC_RENDER_TOOL_NAME
            }
        });

        let api_key = self.api_key.clone();
        let text = send_with_retry("Anthropic", || {
            self.client
                .post(&url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: AnthropicResponse = serde_json::from_str(&text)?;
        if let Some(input) = parsed.content.iter().find_map(|block| {
            (block.kind.as_deref() == Some("tool_use")
                && block.name.as_deref() == Some(ANTHROPIC_RENDER_TOOL_NAME))
            .then(|| block.input.clone())
            .flatten()
        }) {
            return Ok(input.to_string());
        }

        parsed
            .content
            .iter()
            .find_map(|block| block.text.clone())
            .ok_or_else(|| anyhow!("empty Anthropic response"))
    }

    async fn send_gemini(&self, system: &str, user: &str) -> Result<String> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            self.model_id,
            self.api_key
        );
        let body = serde_json::json!({
            "system_instruction": { "parts": [{ "text": system }] },
            "contents": [{ "parts": [{ "text": user }] }],
        });

        let text = send_with_retry("Gemini", || {
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: GeminiResponse = serde_json::from_str(&text)?;
        parsed
            .candidates
            .and_then(|c| c.first().cloned())
            .and_then(|c| c.content.parts.first().cloned())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow!("empty Gemini response"))
    }

    async fn send_gemini_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
    ) -> Result<String> {
        let url = format!(
            "{}/v1beta/models/{}:generateContent?key={}",
            self.base_url.trim_end_matches('/'),
            self.model_id,
            self.api_key
        );
        let body = json!({
            "system_instruction": { "parts": [{ "text": system }] },
            "contents": [{ "parts": [{ "text": user }] }],
            "generationConfig": {
                "responseMimeType": "application/json",
                "responseSchema": assistant_response_json_schema(require_citations),
            }
        });

        let text = send_with_retry("Gemini", || {
            self.client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&body)
        })
        .await?;

        let parsed: GeminiResponse = serde_json::from_str(&text)?;
        parsed
            .candidates
            .and_then(|c| c.first().cloned())
            .and_then(|c| c.content.parts.first().cloned())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow!("empty Gemini response"))
    }

    fn build_system_prompt(&self, interpretation: &InterpretationContext) -> String {
        let mut system = String::from(
            "You are Paddles, a recursive in-context planning harness. \
             You provide concise, accurate technical assistance.\n\n",
        );
        if !interpretation.is_empty() {
            system.push_str("## Interpretation Context\n");
            system.push_str(&interpretation.render());
            system.push('\n');
        }
        system
    }

    fn build_planner_system_prompt(&self, interpretation: &InterpretationContext) -> String {
        let mut system = self.build_system_prompt(interpretation);
        system.push_str(
            r#"
## Action Schema

You must respond with a single JSON object selecting your next action. Available actions:

{"action":"answer","rationale":"..."}
{"action":"search","query":"...","mode":"graph","strategy":"hybrid","intent":"...","rationale":"..."}
{"action":"list_files","pattern":"...","rationale":"..."}
{"action":"read","path":"...","rationale":"..."}
{"action":"inspect","command":"...","rationale":"..."}
{"action":"shell","command":"...","rationale":"..."}
{"action":"stop","reason":"...","rationale":"..."}

Rules:
- Choose "answer" or "stop" as soon as you have sufficient evidence. Do not use remaining budget for redundant or confirmatory searches.
- When the user requests a code change, use write_file, replace_in_file, or apply_patch to make the edit directly.
- Respond ONLY with the JSON object, no prose.
"#,
        );
        system
    }

    fn build_answer_system_prompt(&self, require_citations: bool) -> String {
        format!(
            "You are Paddles, a helpful AI assistant. Provide concise, accurate answers.\n\n{}",
            final_answer_contract_prompt(self.render_capability, require_citations)
        )
    }

    fn parse_planner_action(&self, response: &str) -> Result<RecursivePlannerDecision> {
        let json = extract_json(response).unwrap_or(response);
        let envelope: PlannerEnvelope = serde_json::from_str(json)
            .map_err(|e| anyhow!("failed to parse planner action: {e}\nresponse: {response}"))?;
        let rationale = envelope.rationale.unwrap_or_default();
        let action = match envelope.action.as_str() {
            "answer" => PlannerAction::Stop {
                reason: "model selected answer".to_string(),
            },
            "stop" => PlannerAction::Stop {
                reason: envelope.reason.unwrap_or_else(|| "stop".to_string()),
            },
            "search" => PlannerAction::Workspace {
                action: WorkspaceAction::Search {
                    query: envelope.query.unwrap_or_default(),
                    mode: RetrievalMode::Graph,
                    strategy: RetrievalStrategy::Hybrid,
                    intent: envelope.intent,
                },
            },
            "list_files" => PlannerAction::Workspace {
                action: WorkspaceAction::ListFiles {
                    pattern: envelope.pattern,
                },
            },
            "read" => PlannerAction::Workspace {
                action: WorkspaceAction::Read {
                    path: envelope.path.unwrap_or_default(),
                },
            },
            "inspect" => PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: envelope.command.unwrap_or_default(),
                },
            },
            "shell" => PlannerAction::Workspace {
                action: WorkspaceAction::Shell {
                    command: envelope.command.unwrap_or_default(),
                },
            },
            other => PlannerAction::Stop {
                reason: format!("unknown action: {other}"),
            },
        };
        Ok(RecursivePlannerDecision { action, rationale })
    }

    fn execute_local_action(&self, action: &WorkspaceAction) -> Result<WorkspaceActionResult> {
        match action {
            WorkspaceAction::Read { path } => {
                let full = self.workspace_root.join(path);
                let content = std::fs::read_to_string(&full)
                    .unwrap_or_else(|e| format!("failed to read {path}: {e}"));
                Ok(WorkspaceActionResult {
                    name: "read".to_string(),
                    summary: truncate(&content, 4000),
                })
            }
            WorkspaceAction::ListFiles { pattern } => {
                let pat = pattern.as_deref().unwrap_or("*");
                let output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("find . -name '{pat}' -type f | head -100"))
                    .current_dir(&self.workspace_root)
                    .output()?;
                Ok(WorkspaceActionResult {
                    name: "list_files".to_string(),
                    summary: String::from_utf8_lossy(&output.stdout).to_string(),
                })
            }
            WorkspaceAction::Inspect { command } | WorkspaceAction::Shell { command } => {
                let output = std::process::Command::new("sh")
                    .arg("-lc")
                    .arg(command)
                    .current_dir(&self.workspace_root)
                    .output()?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let name = if matches!(action, WorkspaceAction::Inspect { .. }) {
                    "inspect"
                } else {
                    "shell"
                };
                Ok(WorkspaceActionResult {
                    name: name.to_string(),
                    summary: if stderr.trim().is_empty() {
                        truncate(&stdout, 4000)
                    } else {
                        truncate(&format!("{stdout}\n{stderr}"), 4000)
                    },
                })
            }
            WorkspaceAction::Search { query, .. } => Ok(WorkspaceActionResult {
                name: "search".to_string(),
                summary: format!("search not available via HTTP provider for: {query}"),
            }),
            WorkspaceAction::Diff { path } => {
                let cmd = match path {
                    Some(p) if !p.trim().is_empty() => format!("git diff --no-ext-diff -- {p}"),
                    _ => "git diff --no-ext-diff".to_string(),
                };
                let output = std::process::Command::new("sh")
                    .arg("-lc")
                    .arg(&cmd)
                    .current_dir(&self.workspace_root)
                    .output()?;
                Ok(WorkspaceActionResult {
                    name: "diff".to_string(),
                    summary: truncate(&String::from_utf8_lossy(&output.stdout), 4000),
                })
            }
            WorkspaceAction::WriteFile { path, content } => {
                let full = self.workspace_root.join(path);
                std::fs::write(&full, content)?;
                Ok(WorkspaceActionResult {
                    name: "write_file".to_string(),
                    summary: format!("wrote {path}"),
                })
            }
            WorkspaceAction::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => {
                let full = self.workspace_root.join(path);
                let content = std::fs::read_to_string(&full)?;
                let updated = if *replace_all {
                    content.replace(old, new)
                } else {
                    content.replacen(old, new, 1)
                };
                std::fs::write(&full, updated)?;
                Ok(WorkspaceActionResult {
                    name: "replace_in_file".to_string(),
                    summary: format!("replaced text in {path}"),
                })
            }
            WorkspaceAction::ApplyPatch { patch } => {
                let mut child = std::process::Command::new("git")
                    .args(["apply", "--whitespace=nowarn", "-"])
                    .current_dir(&self.workspace_root)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped())
                    .spawn()?;
                if let Some(stdin) = child.stdin.as_mut() {
                    use std::io::Write;
                    stdin.write_all(patch.as_bytes())?;
                }
                let output = child.wait_with_output()?;
                Ok(WorkspaceActionResult {
                    name: "apply_patch".to_string(),
                    summary: if output.status.success() {
                        "patch applied".to_string()
                    } else {
                        format!("patch failed: {}", String::from_utf8_lossy(&output.stderr))
                    },
                })
            }
        }
    }
}

impl SynthesizerEngine for HttpProviderAdapter {
    fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    fn respond_for_turn(
        &self,
        prompt: &str,
        _turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let system = self.build_answer_system_prompt(gathered_evidence.is_some());
        let mut user_msg = prompt.to_string();
        if let Some(evidence) = gathered_evidence {
            user_msg.push_str("\n\n## Evidence\n");
            user_msg.push_str(&evidence.summary);
            for item in &evidence.items {
                user_msg.push_str(&format!("\n- {}: {}", item.source, item.snippet));
            }
        }

        let mut response = normalize_assistant_response(&self.send_structured_answer_blocking(
            &system,
            &user_msg,
            gathered_evidence.is_some(),
        )?);
        let citations = gathered_evidence.map(citation_sources).unwrap_or_default();
        response = ensure_citation_section(&response, &citations);

        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: gathered_evidence.is_some(),
            citations: citations.clone(),
            insufficient_evidence: false,
        });

        self.turn_history
            .lock()
            .expect("turn history lock")
            .push(format!("Q: {prompt} A: {}", truncate(&response, 100)));

        Ok(response)
    }

    fn recent_turn_summaries(&self) -> Result<Vec<String>> {
        Ok(self.turn_history.lock().expect("turn history lock").clone())
    }

    fn execute_workspace_action(&self, action: &WorkspaceAction) -> Result<WorkspaceActionResult> {
        self.execute_local_action(action)
    }
}

/// Wraps HttpProviderAdapter as a RecursivePlanner.
pub struct HttpPlannerAdapter {
    engine: Arc<HttpProviderAdapter>,
}

impl HttpPlannerAdapter {
    pub fn new(engine: Arc<HttpProviderAdapter>) -> Self {
        Self { engine }
    }
}

#[async_trait]
impl crate::domain::ports::RecursivePlanner for HttpPlannerAdapter {
    fn capability(&self) -> PlannerCapability {
        PlannerCapability::Available
    }

    async fn derive_interpretation_context(
        &self,
        request: &InterpretationRequest,
        _event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<InterpretationContext> {
        Ok(InterpretationContext {
            summary: format!(
                "HTTP provider interpretation from {} operator-memory document(s).",
                request.operator_memory.len()
            ),
            documents: request
                .operator_memory
                .iter()
                .map(|doc| crate::domain::ports::InterpretationDocument {
                    source: doc.source.clone(),
                    excerpt: truncate(&doc.contents, 500),
                    category: crate::domain::ports::GuidanceCategory::Convention,
                })
                .collect(),
            tool_hints: Vec::new(),
            decision_framework: Default::default(),
            ..InterpretationContext::default()
        })
    }

    async fn select_initial_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<InitialActionDecision> {
        let system = self
            .engine
            .build_planner_system_prompt(&request.interpretation);
        let user = format!(
            "User prompt: {}\n\nSelect your first action. Respond with JSON only.",
            request.user_prompt
        );
        let response = self.engine.send_async(&system, &user).await?;

        match self.engine.parse_planner_action(&response) {
            Ok(decision) => {
                let action = match &decision.action {
                    PlannerAction::Stop { .. } => InitialAction::Answer,
                    PlannerAction::Workspace { action } => InitialAction::Workspace {
                        action: action.clone(),
                    },
                    _ => InitialAction::Answer,
                };
                Ok(InitialActionDecision {
                    action,
                    rationale: decision.rationale,
                })
            }
            Err(_) => Ok(InitialActionDecision {
                action: InitialAction::Answer,
                rationale: "failed to parse planner response, answering directly".to_string(),
            }),
        }
    }

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
    ) -> Result<RecursivePlannerDecision> {
        let system = self
            .engine
            .build_planner_system_prompt(&request.interpretation);
        let steps_used = request.loop_state.steps.len();
        let steps_remaining = request.budget.max_steps.saturating_sub(steps_used);
        let mut user = format!("User prompt: {}\n\n", request.user_prompt);
        user.push_str(&format!(
            "Budget: {steps_used}/{} steps used, {steps_remaining} remaining.\n\n",
            request.budget.max_steps
        ));
        if !request.loop_state.steps.is_empty() {
            user.push_str("## Previous steps\n");
            for step in &request.loop_state.steps {
                user.push_str(&format!(
                    "- Step {}: {} -> {}\n",
                    step.sequence,
                    step.action.summary(),
                    step.outcome
                ));
            }
        }
        user.push_str("\nSelect your next action. Choose \"answer\" or \"stop\" as soon as you have enough evidence — do not use remaining budget for redundant investigation. Respond with JSON only.");

        let response = self.engine.send_async(&system, &user).await?;
        self.engine.parse_planner_action(&response)
    }

    async fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
    ) -> Result<ThreadDecision> {
        Ok(ThreadDecision {
            decision_id: ThreadDecisionId::new(format!(
                "{}.decision",
                request.candidate.candidate_id.as_str()
            ))?,
            candidate_id: request.candidate.candidate_id.clone(),
            kind: ThreadDecisionKind::ContinueCurrent,
            rationale: "HTTP provider continues on active thread".to_string(),
            target_thread: request.active_thread.thread_ref.clone(),
            new_thread_label: None,
            merge_mode: None,
            merge_summary: None,
        })
    }

    async fn assess_context_relevance(
        &self,
        request: &CompactionRequest,
    ) -> Result<CompactionPlan> {
        // HTTP adapter uses heuristic-driven self-assessment for now.
        let mut decisions = std::collections::HashMap::new();

        for (i, artifact_id) in request.target_scope.iter().enumerate() {
            let decision = if i == 0 {
                CompactionDecision::Keep { priority: 1 }
            } else if i < 3 {
                CompactionDecision::Compact {
                    summary: format!("Summary of artifact {}", artifact_id.as_str()),
                }
            } else {
                CompactionDecision::Discard {
                    reason: "Archived due to context pressure".to_string(),
                }
            };
            decisions.insert(artifact_id.clone(), decision);
        }

        Ok(CompactionPlan { decisions })
    }
}

// --- API response types ---

#[derive(Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiMessage,
}

#[derive(Deserialize)]
struct OpenAiMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicBlock>,
}

#[derive(Deserialize)]
struct AnthropicBlock {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    name: Option<String>,
    text: Option<String>,
    #[serde(default)]
    input: Option<Value>,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize, Clone)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[derive(Deserialize, Clone)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize, Clone)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Deserialize)]
struct PlannerEnvelope {
    action: String,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    intent: Option<String>,
}

fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') {
        return Some(trimmed.split_once('\n').map(|(l, _)| l).unwrap_or(trimmed));
    }
    if let Some(start) = trimmed.find("```json") {
        let after = &trimmed[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    if let Some(start) = trimmed.find('{') {
        let candidate = &trimmed[start..];
        if let Some(end) = candidate.rfind('}') {
            return Some(&candidate[..=end]);
        }
    }
    None
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}...[truncated]", &s[..n])
    } else {
        s.to_string()
    }
}

fn citation_sources(evidence: &EvidenceBundle) -> Vec<String> {
    let mut citations = Vec::new();
    for item in &evidence.items {
        if !citations.contains(&item.source) {
            citations.push(item.source.clone());
        }
    }
    if citations.len() > MAX_CITATIONS {
        citations.truncate(MAX_CITATIONS);
    }
    citations
}

#[cfg(test)]
mod tests {
    use super::{ApiFormat, HttpPlannerAdapter, HttpProviderAdapter};
    use crate::application::{MechSuitService, RuntimeLaneConfig};
    use crate::domain::model::{TurnEvent, TurnEventSink};
    use crate::domain::ports::{ModelPaths, ModelRegistry, RecursivePlanner, SynthesizerEngine};
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::rendering::{ANTHROPIC_RENDER_TOOL_NAME, RenderCapability};
    use anyhow::{Result, anyhow};
    use async_trait::async_trait;
    use axum::{
        Router,
        body::Bytes,
        extract::State,
        http::{HeaderMap, StatusCode, Uri},
        response::{IntoResponse, Response},
        routing::any,
    };
    use serde_json::{Value, json};
    use std::collections::{BTreeMap, VecDeque};
    use std::fs;
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use tokio::task::JoinHandle;

    #[derive(Default)]
    struct StaticRegistry;

    #[async_trait]
    impl ModelRegistry for StaticRegistry {
        async fn get_model_paths(&self, _model_id: &str) -> Result<ModelPaths> {
            Err(anyhow!(
                "static registry is not used by HTTP provider tests"
            ))
        }
    }

    #[derive(Default)]
    struct RecordingTurnEventSink {
        events: Mutex<Vec<TurnEvent>>,
    }

    impl RecordingTurnEventSink {
        fn recorded(&self) -> Vec<TurnEvent> {
            self.events.lock().expect("event lock").clone()
        }
    }

    impl TurnEventSink for RecordingTurnEventSink {
        fn emit(&self, event: TurnEvent) {
            self.events.lock().expect("event lock").push(event);
        }
    }

    #[derive(Clone, Debug)]
    struct MockResponse {
        status: StatusCode,
        body: String,
    }

    #[derive(Clone, Debug, PartialEq)]
    struct RecordedRequest {
        uri: String,
        headers: BTreeMap<String, String>,
        body: Value,
    }

    struct MockServerState {
        responses: Mutex<VecDeque<MockResponse>>,
        requests: Mutex<Vec<RecordedRequest>>,
    }

    struct MockServerHandle {
        base_url: String,
        state: Arc<MockServerState>,
        task: JoinHandle<()>,
    }

    impl MockServerHandle {
        fn recorded_requests(&self) -> Vec<RecordedRequest> {
            self.state.requests.lock().expect("request lock").clone()
        }
    }

    impl Drop for MockServerHandle {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn mock_handler(
        State(state): State<Arc<MockServerState>>,
        headers: HeaderMap,
        uri: Uri,
        body: Bytes,
    ) -> Response {
        let body_text = String::from_utf8(body.to_vec()).expect("utf8 body");
        let body_json =
            serde_json::from_str(&body_text).unwrap_or_else(|_| Value::String(body_text.clone()));
        let recorded = RecordedRequest {
            uri: uri.to_string(),
            headers: headers_to_map(&headers),
            body: body_json,
        };
        state.requests.lock().expect("request lock").push(recorded);

        let response = state
            .responses
            .lock()
            .expect("response lock")
            .pop_front()
            .expect("queued response");
        (
            response.status,
            [("content-type", "application/json")],
            response.body,
        )
            .into_response()
    }

    fn headers_to_map(headers: &HeaderMap) -> BTreeMap<String, String> {
        headers
            .iter()
            .map(|(name, value)| {
                (
                    name.as_str().to_string(),
                    value.to_str().unwrap_or("<non-utf8>").to_string(),
                )
            })
            .collect()
    }

    async fn start_mock_server(responses: Vec<MockResponse>) -> MockServerHandle {
        let state = Arc::new(MockServerState {
            responses: Mutex::new(VecDeque::from(responses)),
            requests: Mutex::new(Vec::new()),
        });
        let app = Router::new()
            .fallback(any(mock_handler))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock server");
        let addr = listener.local_addr().expect("local addr");
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("run mock server");
        });
        MockServerHandle {
            base_url: format!("http://{}", addr),
            state,
            task,
        }
    }

    fn provider_response(format: ApiFormat, content: &str) -> String {
        match format {
            ApiFormat::OpenAi => json!({
                "choices": [
                    {
                        "message": {
                            "content": content
                        }
                    }
                ]
            })
            .to_string(),
            ApiFormat::Anthropic => json!({
                "content": [
                    {
                        "text": content
                    }
                ]
            })
            .to_string(),
            ApiFormat::Gemini => json!({
                "candidates": [
                    {
                        "content": {
                            "parts": [
                                {
                                    "text": content
                                }
                            ]
                        }
                    }
                ]
            })
            .to_string(),
        }
    }

    fn planner_json_answer() -> String {
        json!({
            "action": "answer",
            "rationale": "the mock planner can answer directly"
        })
        .to_string()
    }

    fn http_test_service(
        workspace: &Path,
        base_url: String,
        api_key: String,
        format: ApiFormat,
    ) -> MechSuitService {
        let operator_memory = Arc::new(AgentMemory::load(workspace));

        let synth_base_url = base_url.clone();
        let synth_api_key = api_key.clone();
        let synthesizer_factory: Box<crate::application::SynthesizerFactory> =
            Box::new(move |workspace: &Path, model_id: &str| {
                Ok(Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    model_id.to_string(),
                    synth_api_key.clone(),
                    synth_base_url.clone(),
                    format,
                    render_capability_for(format),
                )) as Arc<dyn SynthesizerEngine>)
            });

        let planner_base_url = base_url;
        let planner_api_key = api_key;
        let planner_factory: Box<crate::application::PlannerFactory> =
            Box::new(move |workspace: &Path, model_id: &str| {
                let engine = Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    model_id.to_string(),
                    planner_api_key.clone(),
                    planner_base_url.clone(),
                    format,
                    render_capability_for(format),
                ));
                Ok(Arc::new(HttpPlannerAdapter::new(engine)) as Arc<dyn RecursivePlanner>)
            });

        let gatherer_factory: Box<crate::application::GathererFactory> =
            Box::new(|_, _, _, _| Ok::<Option<_>, anyhow::Error>(None));

        MechSuitService::new(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            synthesizer_factory,
            planner_factory,
            gatherer_factory,
        )
    }

    async fn run_mocked_turn(
        format: ApiFormat,
        model_id: &str,
        final_answer: &str,
    ) -> (Vec<RecordedRequest>, Vec<TurnEvent>, String) {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse the remote provider planner before answering.\n",
        )
        .expect("write AGENTS.md");

        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: provider_response(format, &planner_json_answer()),
            },
            MockResponse {
                status: StatusCode::OK,
                body: final_answer_response(format, final_answer),
            },
        ])
        .await;

        let service = http_test_service(
            workspace.path(),
            server.base_url.clone(),
            "test-key".to_string(),
            format,
        );
        let runtime_lanes =
            RuntimeLaneConfig::new(model_id.to_string(), None).with_requires_local_models(false);
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare runtime lanes");

        let sink = Arc::new(RecordingTurnEventSink::default());
        let response = service
            .process_prompt_with_sink("Sup dawg", sink.clone())
            .await
            .expect("process prompt");

        (server.recorded_requests(), sink.recorded(), response)
    }

    fn render_capability_for(format: ApiFormat) -> RenderCapability {
        match format {
            ApiFormat::OpenAi => RenderCapability::OpenAiJsonSchema,
            ApiFormat::Anthropic => RenderCapability::AnthropicToolUse,
            ApiFormat::Gemini => RenderCapability::GeminiJsonSchema,
        }
    }

    fn structured_answer_json(text: &str) -> String {
        json!({
            "render_types": ["paragraph"],
            "blocks": [
                { "type": "paragraph", "text": text }
            ]
        })
        .to_string()
    }

    fn final_answer_response(format: ApiFormat, content: &str) -> String {
        match format {
            ApiFormat::OpenAi | ApiFormat::Gemini => provider_response(format, content),
            ApiFormat::Anthropic => {
                let input: Value =
                    serde_json::from_str(content).expect("structured final answer json");
                json!({
                    "content": [
                        {
                            "type": "tool_use",
                            "id": "toolu_test",
                            "name": ANTHROPIC_RENDER_TOOL_NAME,
                            "input": input
                        }
                    ]
                })
                .to_string()
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_provider_executes_a_full_turn_against_a_mock_server() {
        let model_id = "gpt-4o-mini";
        let (requests, events, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            model_id,
            &structured_answer_json("Mocked final answer."),
        )
        .await;

        assert_eq!(response, "Mocked final answer.");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].uri, "/v1/chat/completions");
        assert_eq!(requests[1].uri, "/v1/chat/completions");
        assert_eq!(
            requests[0].headers.get("authorization").map(String::as_str),
            Some("Bearer test-key")
        );
        assert_eq!(requests[0].body["model"].as_str(), Some(model_id));
        assert!(
            requests[0].body["messages"][0]["content"]
                .as_str()
                .expect("planner system prompt")
                .contains("Action Schema")
        );
        assert!(
            requests[0].body["messages"][1]["content"]
                .as_str()
                .expect("planner user prompt")
                .contains("User prompt: Sup dawg")
        );
        let synth_system = requests[1].body["messages"][0]["content"]
            .as_str()
            .expect("synth system prompt");
        assert!(synth_system.contains("You are Paddles, a helpful AI assistant."));
        assert!(synth_system.contains("Final answer rendering contract"));
        assert!(synth_system.contains("paragraph, bullet_list, code_block, citations"));
        assert_eq!(
            requests[1].body["response_format"]["type"].as_str(),
            Some("json_schema")
        );
        assert_eq!(
            requests[1].body["response_format"]["json_schema"]["name"].as_str(),
            Some("assistant_response")
        );
        assert_eq!(
            requests[1].body["response_format"]["json_schema"]["strict"].as_bool(),
            Some(true)
        );
        assert!(
            requests[1].body["messages"][1]["content"]
                .as_str()
                .expect("synth user prompt")
                .contains("Sup dawg")
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::InterpretationContext { context, .. }
                if context.summary.contains("HTTP provider interpretation")
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerCapability { provider, .. } if provider == model_id
        )));
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::SynthesisReady {
                grounded: false,
                ..
            }
        )));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn anthropic_provider_executes_a_full_turn_against_a_mock_server() {
        let model_id = "claude-sonnet-4-20250514";
        let (requests, events, response) = run_mocked_turn(
            ApiFormat::Anthropic,
            model_id,
            &structured_answer_json("Mocked final answer."),
        )
        .await;

        assert_eq!(response, "Mocked final answer.");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].uri, "/v1/messages");
        assert_eq!(requests[1].uri, "/v1/messages");
        assert_eq!(
            requests[0].headers.get("x-api-key").map(String::as_str),
            Some("test-key")
        );
        assert_eq!(
            requests[0]
                .headers
                .get("anthropic-version")
                .map(String::as_str),
            Some("2023-06-01")
        );
        assert_eq!(requests[0].body["model"].as_str(), Some(model_id));
        assert!(
            requests[0].body["system"]
                .as_str()
                .expect("planner system prompt")
                .contains("Action Schema")
        );
        assert!(
            requests[0].body["messages"][0]["content"]
                .as_str()
                .expect("planner user prompt")
                .contains("User prompt: Sup dawg")
        );
        let synth_system = requests[1].body["system"]
            .as_str()
            .expect("synth system prompt");
        assert!(synth_system.contains("You are Paddles, a helpful AI assistant."));
        assert!(synth_system.contains("Final answer rendering contract"));
        assert_eq!(
            requests[1].body["tool_choice"]["type"].as_str(),
            Some("tool")
        );
        assert_eq!(
            requests[1].body["tool_choice"]["name"].as_str(),
            Some(ANTHROPIC_RENDER_TOOL_NAME)
        );
        assert_eq!(
            requests[1].body["tools"][0]["name"].as_str(),
            Some(ANTHROPIC_RENDER_TOOL_NAME)
        );
        assert_eq!(requests[1].body["tools"][0]["strict"].as_bool(), Some(true));
        assert!(
            requests[1].body["messages"][0]["content"]
                .as_str()
                .expect("synth user prompt")
                .contains("Sup dawg")
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerCapability { provider, .. } if provider == model_id
        )));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn gemini_provider_executes_a_full_turn_against_a_mock_server() {
        let model_id = "gemini-2.5-flash";
        let (requests, events, response) = run_mocked_turn(
            ApiFormat::Gemini,
            model_id,
            &structured_answer_json("Mocked final answer."),
        )
        .await;

        assert_eq!(response, "Mocked final answer.");
        assert_eq!(requests.len(), 2);
        assert_eq!(
            requests[0].uri,
            format!("/v1beta/models/{model_id}:generateContent?key=test-key")
        );
        assert_eq!(
            requests[1].uri,
            format!("/v1beta/models/{model_id}:generateContent?key=test-key")
        );
        assert!(
            requests[0].body["system_instruction"]["parts"][0]["text"]
                .as_str()
                .expect("planner system prompt")
                .contains("Action Schema")
        );
        assert!(
            requests[0].body["contents"][0]["parts"][0]["text"]
                .as_str()
                .expect("planner user prompt")
                .contains("User prompt: Sup dawg")
        );
        let synth_system = requests[1].body["system_instruction"]["parts"][0]["text"]
            .as_str()
            .expect("synth system prompt");
        assert!(synth_system.contains("You are Paddles, a helpful AI assistant."));
        assert!(synth_system.contains("Final answer rendering contract"));
        assert_eq!(
            requests[1].body["generationConfig"]["responseMimeType"].as_str(),
            Some("application/json")
        );
        assert_eq!(
            requests[1].body["generationConfig"]["responseSchema"]["type"].as_str(),
            Some("object")
        );
        assert!(
            requests[1].body["contents"][0]["parts"][0]["text"]
                .as_str()
                .expect("synth user prompt")
                .contains("Sup dawg")
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::SynthesisReady {
                grounded: false,
                ..
            }
        )));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_provider_normalizes_structured_final_answers() {
        let model_id = "gpt-4o-mini";
        let structured = json!({
            "render_types": ["paragraph", "bullet_list"],
            "blocks": [
                { "type": "paragraph", "text": "The next bearing is HTTP API Design For Paddles." },
                { "type": "bullet_list", "items": ["status: decision-ready", "EV: 10.31"] }
            ]
        })
        .to_string();

        let (_, _, response) = run_mocked_turn(ApiFormat::OpenAi, model_id, &structured).await;

        assert_eq!(
            response,
            "The next bearing is HTTP API Design For Paddles.\n\n- status: decision-ready\n- EV: 10.31"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_provider_surfaces_mock_server_errors_through_full_turns() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(workspace.path().join("AGENTS.md"), "# Operator Memory\n")
            .expect("write AGENTS.md");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::NOT_FOUND,
            body: json!({
                "error": {
                    "message": "Not found the model kimi-2.5 or Permission denied",
                    "type": "resource_not_found_error"
                }
            })
            .to_string(),
        }])
        .await;
        let service = http_test_service(
            workspace.path(),
            server.base_url.clone(),
            "test-key".to_string(),
            ApiFormat::OpenAi,
        );
        let runtime_lanes =
            RuntimeLaneConfig::new("kimi-k2.5".to_string(), None).with_requires_local_models(false);
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare runtime lanes");

        let err = service
            .process_prompt("Sup dawg")
            .await
            .expect_err("planner request should fail");
        let rendered = format!("{err:#}");
        assert!(rendered.contains("OpenAI API error 404 Not Found"));
        assert!(rendered.contains("Not found the model kimi-2.5 or Permission denied"));
    }

    #[test]
    fn retryable_status_identifies_429_and_5xx() {
        assert!(super::is_retryable_status(StatusCode::TOO_MANY_REQUESTS));
        assert!(super::is_retryable_status(
            StatusCode::INTERNAL_SERVER_ERROR
        ));
        assert!(super::is_retryable_status(StatusCode::BAD_GATEWAY));
        assert!(super::is_retryable_status(StatusCode::SERVICE_UNAVAILABLE));
        assert!(super::is_retryable_status(StatusCode::GATEWAY_TIMEOUT));

        assert!(!super::is_retryable_status(StatusCode::OK));
        assert!(!super::is_retryable_status(StatusCode::BAD_REQUEST));
        assert!(!super::is_retryable_status(StatusCode::UNAUTHORIZED));
        assert!(!super::is_retryable_status(StatusCode::NOT_FOUND));
    }

    #[tokio::test]
    async fn send_with_retry_retries_on_429_then_succeeds() {
        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::TOO_MANY_REQUESTS,
                body: "rate limited".to_string(),
            },
            MockResponse {
                status: StatusCode::OK,
                body: r#"{"ok": true}"#.to_string(),
            },
        ])
        .await;

        let client = reqwest::Client::new();
        let url = format!("{}/test", server.base_url);
        let result = super::send_with_retry("Test", || client.post(&url)).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), r#"{"ok": true}"#);
        assert_eq!(server.recorded_requests().len(), 2);
    }

    #[tokio::test]
    async fn send_with_retry_fails_immediately_on_non_retryable_status() {
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::UNAUTHORIZED,
            body: "unauthorized".to_string(),
        }])
        .await;

        let client = reqwest::Client::new();
        let url = format!("{}/test", server.base_url);
        let result = super::send_with_retry("Test", || client.post(&url)).await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("401"));
        assert!(err.contains("unauthorized"));
        assert_eq!(server.recorded_requests().len(), 1);
    }
}
