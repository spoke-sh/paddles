use crate::domain::model::{
    ThreadDecision, ThreadDecisionId, ThreadDecisionKind, TurnEvent, TurnEventSink, TurnIntent,
};
use crate::domain::ports::{
    EvidenceBundle, InitialAction, InitialActionDecision, InterpretationContext,
    InterpretationRequest, PlannerAction, PlannerCapability, PlannerRequest,
    RecursivePlannerDecision, RetrievalMode, RetrievalStrategy, SynthesizerEngine,
    ThreadDecisionRequest, WorkspaceAction, WorkspaceActionResult,
};
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde::Deserialize;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

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
    ) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            client: reqwest::Client::new(),
            api_key: api_key.into(),
            base_url: base_url.into(),
            model_id: model_id.into(),
            format,
            verbose: AtomicU8::new(0),
            turn_history: Mutex::new(Vec::new()),
        }
    }

    fn send_blocking(&self, system: &str, user: &str) -> Result<String> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow!("no tokio runtime for HTTP provider"))?;
        tokio::task::block_in_place(|| rt.block_on(self.send_async(system, user)))
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

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            bail!("OpenAI API error {status}: {text}");
        }

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

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            bail!("Anthropic API error {status}: {text}");
        }

        let parsed: AnthropicResponse = serde_json::from_str(&text)?;
        parsed
            .content
            .first()
            .and_then(|b| b.text.clone())
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

        let resp = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            bail!("Gemini API error {status}: {text}");
        }

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

Respond ONLY with the JSON object, no prose.
"#,
        );
        system
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
        let system = "You are Paddles, a helpful AI assistant. Provide concise, accurate answers.";
        let mut user_msg = prompt.to_string();
        if let Some(evidence) = gathered_evidence {
            user_msg.push_str("\n\n## Evidence\n");
            user_msg.push_str(&evidence.summary);
            for item in &evidence.items {
                user_msg.push_str(&format!("\n- {}: {}", item.source, item.snippet));
            }
        }

        let response = self.send_blocking(system, &user_msg)?;

        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: gathered_evidence.is_some(),
            citations: gathered_evidence
                .map(|e| e.items.iter().map(|i| i.source.clone()).collect())
                .unwrap_or_default(),
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
                })
                .collect(),
            tool_hints: Vec::new(),
            decision_framework: Default::default(),
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
        let mut user = format!("User prompt: {}\n\n", request.user_prompt);
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
        user.push_str("\nSelect your next action. Respond with JSON only.");

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
    text: Option<String>,
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
