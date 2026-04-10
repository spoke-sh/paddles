use crate::domain::model::ForensicArtifactCapture;
use crate::domain::model::{
    CompactionDecision, CompactionPlan, CompactionRequest, NullTurnEventSink, ThreadDecision,
    ThreadDecisionId, ThreadDecisionKind, TraceArtifactId, TraceModelExchangeCategory,
    TraceModelExchangeLane, TraceModelExchangePhase, TurnEvent, TurnEventSink, TurnIntent,
};
use crate::domain::ports::{
    EvidenceBundle, GroundingDomain, GroundingRequirement, InitialAction, InitialActionDecision,
    InitialEditInstruction, InterpretationContext, InterpretationRequest, PlannerAction,
    PlannerCapability, PlannerRequest, RecursivePlannerDecision, RetrievalMode, RetrievalStrategy,
    RetrieverOption, SynthesisHandoff, SynthesizerEngine, ThreadDecisionRequest, WorkspaceAction,
    WorkspaceActionResult, WorkspaceEditor,
};
use crate::infrastructure::adapters::local_workspace_editor::LocalWorkspaceEditor;
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::providers::{
    ApiFormat, ModelCapabilitySurface, ModelProvider, PlannerToolCallCapability,
    ProviderTransportSupport,
};
use crate::infrastructure::rendering::{
    ANTHROPIC_RENDER_TOOL_NAME, RenderCapability, assistant_response_json_schema,
    ensure_citation_section, extract_http_urls, final_answer_contract_prompt,
    normalize_assistant_response,
};
use crate::infrastructure::terminal::run_background_terminal_command_with_runtime_mediator;
use crate::infrastructure::transport_mediator::TransportToolMediator;
use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

const MAX_CITATIONS: usize = 4;
const MAX_RETRIES: u32 = 3;
const RETRY_BASE_DELAY_MS: u64 = 2000;
const OPENAI_PLANNER_TOOL_NAME: &str = "select_planner_action";
const OPENAI_MAX_COMPLETION_TOKENS: u32 = 4096;

#[derive(Clone)]
struct ExchangeCapture<'a> {
    event_sink: &'a dyn TurnEventSink,
    exchange_id: String,
    lane: TraceModelExchangeLane,
    category: TraceModelExchangeCategory,
}

struct ExchangeArtifactRecord {
    phase: TraceModelExchangePhase,
    summary: String,
    content: String,
    mime_type: String,
    parent_artifact_id: Option<TraceArtifactId>,
    labels: BTreeMap<String, String>,
}

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

/// HTTP-based model provider implementing SynthesizerEngine.
pub struct HttpProviderAdapter {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    client: reqwest::Client,
    provider_name: String,
    api_key: String,
    base_url: String,
    model_id: String,
    format: ApiFormat,
    capabilities: ModelCapabilitySurface,
    verbose: AtomicU8,
    turn_history: Mutex<Vec<String>>,
}

fn negotiated_capability_surface(
    provider_name: &str,
    model_id: &str,
    format: ApiFormat,
    render_capability: RenderCapability,
) -> ModelCapabilitySurface {
    if let Some(provider) = ModelProvider::from_name(provider_name) {
        return provider.capability_surface(model_id);
    }

    ModelCapabilitySurface {
        http_format: Some(format),
        render_capability,
        planner_tool_call: match format {
            ApiFormat::OpenAi => PlannerToolCallCapability::NativeFunctionTool,
            ApiFormat::Gemini => PlannerToolCallCapability::StructuredJsonEnvelope,
            ApiFormat::Anthropic => PlannerToolCallCapability::PromptEnvelope,
        },
        transport_support: ProviderTransportSupport::Supported,
    }
}

impl HttpProviderAdapter {
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        provider_name: impl Into<String>,
        model_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        format: ApiFormat,
        render_capability: RenderCapability,
    ) -> Self {
        Self::new_with_execution_hand_registry(
            workspace_root,
            Arc::new(ExecutionHandRegistry::default()),
            provider_name,
            model_id,
            api_key,
            base_url,
            format,
            render_capability,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_execution_hand_registry(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        provider_name: impl Into<String>,
        model_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        format: ApiFormat,
        render_capability: RenderCapability,
    ) -> Self {
        let transport_mediator = Arc::new(TransportToolMediator::with_execution_hand_registry(
            Arc::clone(&execution_hand_registry),
        ));
        Self::new_with_runtime_mediator(
            workspace_root,
            execution_hand_registry,
            transport_mediator,
            provider_name,
            model_id,
            api_key,
            base_url,
            format,
            render_capability,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn new_with_runtime_mediator(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        transport_mediator: Arc<TransportToolMediator>,
        provider_name: impl Into<String>,
        model_id: impl Into<String>,
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        format: ApiFormat,
        render_capability: RenderCapability,
    ) -> Self {
        let provider_name = provider_name.into();
        let model_id = model_id.into();
        let capabilities =
            negotiated_capability_surface(&provider_name, &model_id, format, render_capability);
        Self {
            workspace_root: workspace_root.into(),
            execution_hand_registry,
            transport_mediator,
            client: reqwest::Client::new(),
            provider_name,
            api_key: api_key.into(),
            base_url: base_url.into(),
            model_id,
            format,
            capabilities,
            verbose: AtomicU8::new(0),
            turn_history: Mutex::new(Vec::new()),
        }
    }

    async fn send_async(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        let verbose = self.verbose.load(Ordering::Relaxed);
        if verbose >= 2 {
            eprintln!("[HTTP] Sending to {} ({})", self.base_url, self.model_id);
        }
        if verbose >= 3 {
            eprintln!("[HTTP] System: {system}");
            eprintln!("[HTTP] User: {user}");
        }

        let response = match self.format {
            ApiFormat::OpenAi => self.send_openai(system, user, capture).await?,
            ApiFormat::Anthropic => self.send_anthropic(system, user, capture).await?,
            ApiFormat::Gemini => self.send_gemini(system, user, capture).await?,
        };

        if verbose >= 2 {
            eprintln!(
                "[HTTP] Response: {}",
                if response.0.len() > 200 {
                    format!("{}...", &response.0[..200])
                } else {
                    response.0.clone()
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
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        let rt = tokio::runtime::Handle::try_current()
            .map_err(|_| anyhow!("no tokio runtime for HTTP provider"))?;
        tokio::task::block_in_place(|| {
            rt.block_on(self.send_structured_answer_async(system, user, require_citations, capture))
        })
    }

    async fn send_structured_answer_async(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        match self.capabilities.render_capability {
            RenderCapability::OpenAiJsonSchema => {
                self.send_openai_structured_answer(system, user, require_citations, capture)
                    .await
            }
            RenderCapability::AnthropicToolUse => {
                self.send_anthropic_structured_answer(system, user, require_citations, capture)
                    .await
            }
            RenderCapability::GeminiJsonSchema => {
                self.send_gemini_structured_answer(system, user, require_citations, capture)
                    .await
            }
            RenderCapability::PromptEnvelope => self.send_async(system, user, capture).await,
        }
    }

    fn provider_label(&self) -> &'static str {
        match self.provider_name.as_str() {
            "openai" => "openai",
            "inception" => "inception",
            "anthropic" => "anthropic",
            "google" => "google",
            "moonshot" => "moonshot",
            "ollama" => "ollama",
            _ => match self.format {
                ApiFormat::OpenAi => "openai",
                ApiFormat::Anthropic => "anthropic",
                ApiFormat::Gemini => "gemini",
            },
        }
    }

    fn record_exchange_artifact(
        &self,
        capture: &ExchangeCapture<'_>,
        record: ExchangeArtifactRecord,
    ) -> Option<TraceArtifactId> {
        capture.event_sink.forensic_trace_sink().and_then(|sink| {
            sink.record_forensic_artifact(ForensicArtifactCapture {
                exchange_id: capture.exchange_id.clone(),
                lane: capture.lane,
                category: capture.category,
                phase: record.phase,
                provider: self.provider_label().to_string(),
                model: self.model_id.clone(),
                parent_artifact_id: record.parent_artifact_id,
                summary: record.summary,
                content: record.content,
                mime_type: record.mime_type,
                labels: record.labels,
            })
        })
    }

    fn record_assembled_context(
        &self,
        capture: &ExchangeCapture<'_>,
        system: &str,
        user: &str,
    ) -> Option<TraceArtifactId> {
        let mut labels = BTreeMap::new();
        labels.insert("format".to_string(), self.provider_label().to_string());
        self.record_exchange_artifact(
            capture,
            ExchangeArtifactRecord {
                phase: TraceModelExchangePhase::AssembledContext,
                summary: format!(
                    "{} {} assembled context",
                    capture.lane.label(),
                    capture.category.label()
                ),
                content: json!({
                    "system": system,
                    "user": user,
                })
                .to_string(),
                mime_type: "application/json".to_string(),
                parent_artifact_id: None,
                labels,
            },
        )
    }

    fn record_rendered_response(
        &self,
        capture: &ExchangeCapture<'_>,
        rendered: &str,
        parent_artifact_id: Option<TraceArtifactId>,
    ) -> Option<TraceArtifactId> {
        self.record_exchange_artifact(
            capture,
            ExchangeArtifactRecord {
                phase: TraceModelExchangePhase::RenderedResponse,
                summary: format!(
                    "{} {} rendered response",
                    capture.lane.label(),
                    capture.category.label()
                ),
                content: rendered.to_string(),
                mime_type: "text/plain".to_string(),
                parent_artifact_id,
                labels: BTreeMap::new(),
            },
        )
    }

    async fn post_json_with_capture(
        &self,
        provider_name: &str,
        url: &str,
        headers: &[(String, String)],
        body: &Value,
        capture: Option<ExchangeCapture<'_>>,
        assembled_context_id: Option<TraceArtifactId>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        let request_artifact_id = capture.as_ref().and_then(|capture| {
            let borrowed = headers
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect::<Vec<_>>();
            self.record_exchange_artifact(
                capture,
                ExchangeArtifactRecord {
                    phase: TraceModelExchangePhase::ProviderRequest,
                    summary: format!(
                        "{} {} provider request",
                        capture.lane.label(),
                        capture.category.label()
                    ),
                    content: redacted_http_request_snapshot(url, &borrowed, body),
                    mime_type: "application/json".to_string(),
                    parent_artifact_id: assembled_context_id,
                    labels: BTreeMap::new(),
                },
            )
        });

        let text = send_with_retry(provider_name, || {
            let mut request = self.client.post(url);
            for (name, value) in headers {
                request = request.header(name, value);
            }
            request.json(body)
        })
        .await?;

        let raw_response_artifact_id = capture.as_ref().and_then(|capture| {
            self.record_exchange_artifact(
                capture,
                ExchangeArtifactRecord {
                    phase: TraceModelExchangePhase::RawProviderResponse,
                    summary: format!(
                        "{} {} raw provider response",
                        capture.lane.label(),
                        capture.category.label()
                    ),
                    content: text.clone(),
                    mime_type: "application/json".to_string(),
                    parent_artifact_id: request_artifact_id,
                    labels: BTreeMap::new(),
                },
            )
        });

        Ok((text, raw_response_artifact_id))
    }

    async fn send_openai(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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
            "max_completion_tokens": OPENAI_MAX_COMPLETION_TOKENS,
        });

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "OpenAI",
                &url,
                &[
                    (
                        "Authorization".to_string(),
                        format!("Bearer {}", self.api_key),
                    ),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: OpenAiResponse = serde_json::from_str(&text)?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty OpenAI response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_openai_json_schema(
        &self,
        system: &str,
        user: &str,
        schema_name: &str,
        schema: Value,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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
            "max_completion_tokens": OPENAI_MAX_COMPLETION_TOKENS,
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": schema_name,
                    "strict": true,
                    "schema": schema,
                }
            }
        });

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "OpenAI",
                &url,
                &[
                    (
                        "Authorization".to_string(),
                        format!("Bearer {}", self.api_key),
                    ),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: OpenAiResponse = serde_json::from_str(&text)?;
        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty OpenAI response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_openai_planner_tool_call(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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
            "max_completion_tokens": OPENAI_MAX_COMPLETION_TOKENS,
            "tools": [{
                "type": "function",
                "function": {
                    "name": OPENAI_PLANNER_TOOL_NAME,
                    "description": "Select the next bounded planner action for local execution in the Paddles harness.",
                    "parameters": planner_action_json_schema(),
                    "strict": true
                }
            }],
            "tool_choice": {
                "type": "function",
                "function": {
                    "name": OPENAI_PLANNER_TOOL_NAME
                }
            }
        });

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "OpenAI",
                &url,
                &[
                    (
                        "Authorization".to_string(),
                        format!("Bearer {}", self.api_key),
                    ),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: OpenAiResponse = serde_json::from_str(&text)?;
        if let Some(arguments) = parsed
            .choices
            .first()
            .and_then(|choice| choice.message.tool_calls.as_ref())
            .and_then(|calls| {
                calls.iter().find(|call| {
                    call.kind.as_deref().unwrap_or("function") == "function"
                        && call.function.name == OPENAI_PLANNER_TOOL_NAME
                })
            })
            .map(|call| call.function.arguments.clone())
        {
            return Ok((arguments, raw_response_artifact_id));
        }

        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .ok_or_else(|| anyhow!("empty OpenAI response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_openai_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        self.send_openai_json_schema(
            system,
            user,
            "assistant_response",
            assistant_response_json_schema(require_citations),
            capture,
        )
        .await
    }

    async fn send_anthropic(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        let url = format!("{}/v1/messages", self.base_url.trim_end_matches('/'));
        let body = serde_json::json!({
            "model": self.model_id,
            "max_tokens": 4096,
            "system": system,
            "messages": [
                { "role": "user", "content": user },
            ],
        });

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "Anthropic",
                &url,
                &[
                    ("x-api-key".to_string(), self.api_key.clone()),
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: AnthropicResponse = serde_json::from_str(&text)?;
        let content = parsed
            .content
            .first()
            .and_then(|b| b.text.clone())
            .ok_or_else(|| anyhow!("empty Anthropic response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_anthropic_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "Anthropic",
                &url,
                &[
                    ("x-api-key".to_string(), self.api_key.clone()),
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                    ("Content-Type".to_string(), "application/json".to_string()),
                ],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: AnthropicResponse = serde_json::from_str(&text)?;
        if let Some(input) = parsed.content.iter().find_map(|block| {
            (block.kind.as_deref() == Some("tool_use")
                && block.name.as_deref() == Some(ANTHROPIC_RENDER_TOOL_NAME))
            .then(|| block.input.clone())
            .flatten()
        }) {
            return Ok((input.to_string(), raw_response_artifact_id));
        }

        let content = parsed
            .content
            .iter()
            .find_map(|block| block.text.clone())
            .ok_or_else(|| anyhow!("empty Anthropic response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_gemini(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "Gemini",
                &url,
                &[("Content-Type".to_string(), "application/json".to_string())],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: GeminiResponse = serde_json::from_str(&text)?;
        let content = parsed
            .candidates
            .and_then(|c| c.first().cloned())
            .and_then(|c| c.content.parts.first().cloned())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow!("empty Gemini response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_gemini_json_schema(
        &self,
        system: &str,
        user: &str,
        schema: Value,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
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
                "responseSchema": schema,
            }
        });

        let assembled_context_id = capture
            .as_ref()
            .and_then(|capture| self.record_assembled_context(capture, system, user));
        let (text, raw_response_artifact_id) = self
            .post_json_with_capture(
                "Gemini",
                &url,
                &[("Content-Type".to_string(), "application/json".to_string())],
                &body,
                capture,
                assembled_context_id,
            )
            .await?;

        let parsed: GeminiResponse = serde_json::from_str(&text)?;
        let content = parsed
            .candidates
            .and_then(|c| c.first().cloned())
            .and_then(|c| c.content.parts.first().cloned())
            .and_then(|p| p.text)
            .ok_or_else(|| anyhow!("empty Gemini response"))?;
        Ok((content, raw_response_artifact_id))
    }

    async fn send_gemini_structured_answer(
        &self,
        system: &str,
        user: &str,
        require_citations: bool,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        self.send_gemini_json_schema(
            system,
            user,
            assistant_response_json_schema(require_citations),
            capture,
        )
        .await
    }

    fn build_system_prompt(&self, interpretation: &InterpretationContext) -> String {
        let mut system = String::from(
            "You are Paddles, a recursive in-context planning harness. \
             You provide concise, accurate technical assistance.\n\n",
        );
        system.push_str(
            "Core mission: when the user asks for a safe, reasonable repository change, \
             Paddles should make the workspace edit in this turn. Do not stop at diagnosis, \
             plans, or prose once local evidence is sufficient to act.\n\n",
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
        let transport_rule = match self.capabilities.planner_tool_call {
            PlannerToolCallCapability::NativeFunctionTool => {
                "The transport exposes a native tool named `select_planner_action`; call it exactly once and put the complete action envelope in the tool arguments."
            }
            PlannerToolCallCapability::StructuredJsonEnvelope => {
                "The transport enforces a JSON schema, but you must still produce one complete action envelope."
            }
            PlannerToolCallCapability::PromptEnvelope => {
                "Your response is parsed directly, so the action envelope must be valid JSON on the first try."
            }
        };
        system.push_str(&format!(
            r#"
## Harness Reality

- You are the remote planner model inside Paddles.
- Paddles executes your selected action locally inside the operator's repository workspace.
- Treat user-reported failures or broken states as working hypotheses until local evidence confirms them.
- As evidence accumulates, revise the premise explicitly when commands weaken or contradict it.
- Do not ask the user for logs, file contents, or repository state that the harness can inspect locally.

## Action Schema

You must respond with a single JSON object selecting your next action. Available actions:

{{"action":"answer","answer":"...","edit":"no","candidate_files":[],"rationale":"..."}}
{{"action":"search","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"intent":"...","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"list_files","pattern":"...","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"read","path":"...","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"inspect","command":"...","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"shell","command":"...","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"diff","path":"optional","edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"write_file","path":"relative/path","content":"full file contents","edit":"yes","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"replace_in_file","path":"relative/path","old":"exact old text","new":"replacement text","replace_all":false,"edit":"yes","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"apply_patch","patch":"unified diff text","edit":"yes","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"refine","query":"...","mode":"linear|graph","strategy":"bm25|vector","retrievers":["path-fuzzy","segment-fuzzy"],"edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"branch","branches":["...","..."],"edit":"yes|no","candidate_files":["path1","path2"],"rationale":"..."}}
{{"action":"stop","reason":"...","edit":"no","candidate_files":[],"rationale":"..."}}

Rules:
- {transport_rule}
- Return exactly one complete JSON object.
- The first key must be `action`.
- Do not wrap the JSON in markdown fences, prose, or commentary.
- Do not emit partial, truncated, or streaming JSON fragments.
- `answer` requires `answer` and `rationale`.
- `answer` is the user-facing reply text; `rationale` explains why this control action is correct.
- `search` requires `query`; include `strategy` when you know whether to use `bm25` or `vector`.
- `retrievers` is optional; supported values are `path-fuzzy` and `segment-fuzzy`.
- `list_files` requires `pattern`.
- `read` requires `path`.
- `inspect` and `shell` require `command`.
- `write_file` requires `path` and `content`.
- `replace_in_file` requires `path`, `old`, and `new`.
- `apply_patch` requires `patch` and it must be a unified diff.
- `refine` requires `query`.
- `branch` requires `branches`.
- `stop` requires `reason`.
- If `stop.reason` is `model selected answer`, include `answer` with the user-facing reply instead of putting the reply in `rationale`.
- When the user is clearly asking for a code or file edit, set `edit` to `yes` and include up to 3 plausible relative paths in `candidate_files`.
- When the user is not asking for an edit, set `edit` to `no` and use an empty `candidate_files` array.
- Add optional top-level `grounding` when the answer requires verified evidence before it can be trusted.
- `grounding` must be `{{"domain":"repository|external|mixed","reason":"..."}}`.
- Use `grounding.domain = "external"` for websites, docs links, package pages, or any request to read about something on the web.
- Use `grounding.domain = "repository"` for repository claims that require local evidence.
- Use `grounding.domain = "mixed"` when both repository and external evidence are required.
- Do not invent action names outside the schema above.
- Choose "answer" or "stop" as soon as you have sufficient evidence. Do not use remaining budget for redundant or confirmatory searches.
- When the user requests a code change, choose the nearest supported workspace action that advances the edit, usually `read`, `inspect`, or `shell`.
- When the user requests a concrete code change, prefer `write_file`, `replace_in_file`, or `apply_patch` over describing the edit in prose.
- Add `retrievers=["path-fuzzy"]` when the query names a likely file, path, selector, or symbol and structural path similarity should steer retrieval.
- Add `retrievers=["path-fuzzy","segment-fuzzy"]` when you need definition-oriented fuzzy lookup for a code shape, component, or structural snippet.
- If `edit` is `yes` and one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.
- If the loop-state notes contain a `Steering review`, judge the proposed move against the gathered sources and return the action that should actually execute next.
- Respond ONLY with the JSON object, no prose.
"#
        ));
        system
    }

    fn build_answer_system_prompt(&self, require_citations: bool) -> String {
        format!(
            "You are Paddles, a helpful AI assistant. Provide concise, accurate answers. Do not invent package names, crate names, websites, or URLs.\n\n{}",
            final_answer_contract_prompt(self.capabilities.render_capability, require_citations)
        )
    }

    async fn send_planner_action_async(
        &self,
        system: &str,
        user: &str,
        capture: Option<ExchangeCapture<'_>>,
    ) -> Result<(String, Option<TraceArtifactId>)> {
        match self.capabilities.planner_tool_call {
            PlannerToolCallCapability::NativeFunctionTool => {
                self.send_openai_planner_tool_call(system, user, capture)
                    .await
            }
            PlannerToolCallCapability::StructuredJsonEnvelope => {
                self.send_gemini_json_schema(system, user, planner_action_json_schema(), capture)
                    .await
            }
            PlannerToolCallCapability::PromptEnvelope => {
                self.send_anthropic(system, user, capture).await
            }
        }
    }

    fn parse_planner_action(&self, response: &str) -> Result<RecursivePlannerDecision> {
        let json = extract_json(response).unwrap_or(response);
        let envelope: PlannerEnvelope = serde_json::from_str(json)
            .map_err(|e| anyhow!("failed to parse planner action: {e}\nresponse: {response}"))?;
        let edit = edit_instruction_from_http_envelope(&envelope)?;
        let action_name = infer_planner_action_name(&envelope).ok_or_else(|| {
            anyhow!(
                "failed to parse planner action: missing action field and no inferrable selector\nresponse: {response}"
            )
        })?;
        let rationale = envelope.rationale.unwrap_or_default();
        let answer = envelope
            .answer
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string);
        let action = match action_name.as_str() {
            "answer" => PlannerAction::Stop {
                reason: "model selected answer".to_string(),
            },
            "stop" => PlannerAction::Stop {
                reason: envelope.reason.unwrap_or_else(|| "stop".to_string()),
            },
            "search" => PlannerAction::Workspace {
                action: WorkspaceAction::Search {
                    query: envelope.query.unwrap_or_default(),
                    mode: envelope.mode.unwrap_or(RetrievalMode::Graph),
                    strategy: envelope.strategy.unwrap_or_default(),
                    retrievers: envelope.retrievers.unwrap_or_default(),
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
            "diff" => PlannerAction::Workspace {
                action: WorkspaceAction::Diff {
                    path: envelope.path,
                },
            },
            "write_file" => PlannerAction::Workspace {
                action: WorkspaceAction::WriteFile {
                    path: envelope.path.unwrap_or_default(),
                    content: envelope.content.unwrap_or_default(),
                },
            },
            "replace_in_file" => PlannerAction::Workspace {
                action: WorkspaceAction::ReplaceInFile {
                    path: envelope.path.unwrap_or_default(),
                    old: envelope.old.unwrap_or_default(),
                    new: envelope.new.unwrap_or_default(),
                    replace_all: envelope.replace_all.unwrap_or(false),
                },
            },
            "apply_patch" => PlannerAction::Workspace {
                action: WorkspaceAction::ApplyPatch {
                    patch: envelope.patch.unwrap_or_default(),
                },
            },
            "refine" => PlannerAction::Refine {
                query: envelope.query.unwrap_or_default(),
                mode: envelope.mode.unwrap_or(RetrievalMode::Graph),
                strategy: envelope.strategy.unwrap_or_default(),
                retrievers: envelope.retrievers.unwrap_or_default(),
                rationale: (!rationale.is_empty()).then_some(rationale.clone()),
            },
            "branch" => PlannerAction::Branch {
                branches: envelope.branches.unwrap_or_default(),
                rationale: (!rationale.is_empty()).then_some(rationale.clone()),
            },
            other => PlannerAction::Stop {
                reason: format!("unknown action: {other}"),
            },
        };
        Ok(RecursivePlannerDecision {
            action,
            rationale,
            answer,
            edit,
            grounding: envelope.grounding,
        })
    }

    fn parse_initial_action_decision(&self, response: &str) -> Result<InitialActionDecision> {
        let json = extract_json(response).unwrap_or(response);
        let envelope: PlannerEnvelope = serde_json::from_str(json)
            .map_err(|e| anyhow!("failed to parse planner action: {e}\nresponse: {response}"))?;
        let action_name = infer_planner_action_name(&envelope);
        let decision = self.parse_planner_action(response)?;
        Ok(InitialActionDecision {
            action: match decision.action {
                PlannerAction::Stop { reason } => {
                    if action_name.as_deref() == Some("answer") || reason == "model selected answer"
                    {
                        InitialAction::Answer
                    } else {
                        InitialAction::Stop { reason }
                    }
                }
                PlannerAction::Workspace { action } => InitialAction::Workspace { action },
                PlannerAction::Refine {
                    query,
                    mode,
                    strategy,
                    retrievers,
                    rationale,
                } => InitialAction::Refine {
                    query,
                    mode,
                    strategy,
                    retrievers,
                    rationale,
                },
                PlannerAction::Branch {
                    branches,
                    rationale,
                } => InitialAction::Branch {
                    branches,
                    rationale,
                },
            },
            rationale: decision.rationale,
            answer: decision.answer,
            edit: edit_instruction_from_http_envelope(&envelope)?,
            grounding: envelope.grounding,
        })
    }

    fn execute_local_action(&self, action: &WorkspaceAction) -> Result<WorkspaceActionResult> {
        let workspace_editor = LocalWorkspaceEditor::with_runtime_mediator(
            self.workspace_root.clone(),
            Arc::clone(&self.execution_hand_registry),
            Arc::clone(&self.transport_mediator),
        );
        match action {
            WorkspaceAction::Read { path } => {
                let full = self.workspace_root.join(path);
                let content = std::fs::read_to_string(&full)
                    .unwrap_or_else(|e| format!("failed to read {path}: {e}"));
                Ok(WorkspaceActionResult {
                    name: "read".to_string(),
                    summary: truncate(&content, 4000),
                    applied_edit: None,
                })
            }
            WorkspaceAction::ListFiles { pattern } => {
                let pat = pattern.as_deref().unwrap_or("*");
                let output = run_background_terminal_command_with_runtime_mediator(
                    &self.workspace_root,
                    &format!("find . -name '{pat}' -type f | head -100"),
                    "list_files",
                    "http-provider-list-files",
                    &NullTurnEventSink,
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )?;
                Ok(WorkspaceActionResult {
                    name: "list_files".to_string(),
                    summary: String::from_utf8_lossy(&output.stdout).to_string(),
                    applied_edit: None,
                })
            }
            WorkspaceAction::Inspect { command } | WorkspaceAction::Shell { command } => {
                let tool_name = if matches!(action, WorkspaceAction::Inspect { .. }) {
                    "inspect"
                } else {
                    "shell"
                };
                let output = run_background_terminal_command_with_runtime_mediator(
                    &self.workspace_root,
                    command,
                    tool_name,
                    "http-provider-workspace-action",
                    &NullTurnEventSink,
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )?;
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                Ok(WorkspaceActionResult {
                    name: tool_name.to_string(),
                    summary: if stderr.trim().is_empty() {
                        truncate(&stdout, 4000)
                    } else {
                        truncate(&format!("{stdout}\n{stderr}"), 4000)
                    },
                    applied_edit: None,
                })
            }
            WorkspaceAction::Search { query, .. } => Ok(WorkspaceActionResult {
                name: "search".to_string(),
                summary: format!("search not available via HTTP provider for: {query}"),
                applied_edit: None,
            }),
            WorkspaceAction::Diff { path } => workspace_editor.diff(path.as_deref()),
            WorkspaceAction::WriteFile { path, content } => {
                workspace_editor.write_file(path, content)
            }
            WorkspaceAction::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => workspace_editor.replace_in_file(path, old, new, *replace_all),
            WorkspaceAction::ApplyPatch { patch } => workspace_editor.apply_patch(patch),
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
        handoff: &SynthesisHandoff,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        let system = self.build_answer_system_prompt(gathered_evidence.is_some());
        let mut user_msg = String::new();
        if !handoff.recent_turns.is_empty() {
            user_msg.push_str("## Recent Conversation\n");
            for turn in &handoff.recent_turns {
                user_msg.push_str("- ");
                user_msg.push_str(turn);
                user_msg.push('\n');
            }
            user_msg.push('\n');
        }
        if let Some(summary) = handoff
            .recent_thread_summary
            .as_deref()
            .filter(|summary| !summary.trim().is_empty())
        {
            user_msg.push_str("## Active Thread Summary\n");
            user_msg.push_str(summary);
            user_msg.push_str("\n\n");
        }
        if let Some(frame) = handoff.instruction_frame.as_ref() {
            user_msg.push_str("## Instruction Manifold\n");
            if frame.requires_applied_edit() && frame.requires_applied_commit() {
                user_msg.push_str(
                    "Open obligation: this turn is not complete until Paddles applies the repository change and records the requested git commit.\n",
                );
            } else if frame.requires_applied_edit() {
                user_msg.push_str(
                    "Open obligation: this turn is not complete until Paddles applies a repository edit.\n",
                );
            } else if frame.requires_applied_commit() {
                user_msg.push_str(
                    "Open obligation: this turn is not complete until Paddles records the requested git commit.\n",
                );
            } else {
                user_msg.push_str("Instruction obligations are currently satisfied.\n");
            }
            if let Some(candidates) = frame.candidate_summary() {
                user_msg.push_str("Candidate files: ");
                user_msg.push_str(&candidates);
                user_msg.push('\n');
            }
            user_msg.push('\n');
        }
        if let Some(grounding) = handoff.grounding.as_ref() {
            user_msg.push_str("## Grounding Contract\n");
            user_msg.push_str(&format_grounding_contract(grounding));
            user_msg.push_str("\n\n");
        }
        user_msg.push_str("## Current User Request\n");
        user_msg.push_str(prompt);
        if let Some(evidence) = gathered_evidence {
            user_msg.push_str("\n\n## Evidence\n");
            user_msg.push_str(&evidence.summary);
            for item in &evidence.items {
                user_msg.push_str(&format!("\n- {}: {}", item.source, item.snippet));
            }
        }

        let capture = ExchangeCapture {
            event_sink: event_sink.as_ref(),
            exchange_id: event_sink
                .forensic_trace_sink()
                .map(|sink| {
                    sink.allocate_model_exchange_id(
                        TraceModelExchangeLane::Synthesizer,
                        TraceModelExchangeCategory::TurnResponse,
                    )
                })
                .unwrap_or_else(|| "exchange:untracked".to_string()),
            lane: TraceModelExchangeLane::Synthesizer,
            category: TraceModelExchangeCategory::TurnResponse,
        };
        let (raw_response, raw_response_artifact_id) = self.send_structured_answer_blocking(
            &system,
            &user_msg,
            gathered_evidence.is_some(),
            Some(capture.clone()),
        )?;
        let mut response = normalize_assistant_response(&raw_response);
        let verified_external_urls = gathered_evidence
            .map(verified_external_urls_from_evidence)
            .unwrap_or_default();
        if external_grounding_required_without_verified_sources(handoff, &verified_external_urls) {
            event_sink.emit(TurnEvent::Fallback {
                stage: "grounding-governor".to_string(),
                reason: "planner declared external grounding, but no verified external sources were attached".to_string(),
            });
            response = external_grounding_unavailable_fallback(prompt);
        } else if let Some(unverified_url) =
            first_unverified_external_url(&response, &verified_external_urls)
        {
            event_sink.emit(TurnEvent::Fallback {
                stage: "grounding-governor".to_string(),
                reason: format!(
                    "drafted answer referenced an unverified external URL without verified external sources: {unverified_url}"
                ),
            });
            response = unverified_external_url_fallback(prompt);
        } else if let Some(evidence) = gathered_evidence
            && grounded_response_defers_executed_command_work(&response, evidence)
        {
            response = grounded_command_evidence_fallback(evidence);
        } else if let Some(evidence) = gathered_evidence
            && grounded_response_overstates_unverified_failure(&response, prompt, evidence)
        {
            response = grounded_unverified_failure_fallback(evidence);
        }
        let citations = gathered_evidence.map(citation_sources).unwrap_or_default();
        response = ensure_citation_section(&response, &citations);
        self.record_rendered_response(&capture, &response, raw_response_artifact_id);

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

    fn exchange_capture<'a>(
        &self,
        event_sink: &'a dyn TurnEventSink,
        category: TraceModelExchangeCategory,
    ) -> ExchangeCapture<'a> {
        ExchangeCapture {
            event_sink,
            exchange_id: event_sink
                .forensic_trace_sink()
                .map(|sink| {
                    sink.allocate_model_exchange_id(TraceModelExchangeLane::Planner, category)
                })
                .unwrap_or_else(|| "exchange:untracked".to_string()),
            lane: TraceModelExchangeLane::Planner,
            category,
        }
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
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<InitialActionDecision> {
        let system = self
            .engine
            .build_planner_system_prompt(&request.interpretation);
        let mut user = build_http_initial_action_prompt(request, self.engine.format);
        let mut capture = self.exchange_capture(
            event_sink.as_ref(),
            TraceModelExchangeCategory::InitialAction,
        );
        let (mut response, mut raw_response_artifact_id) = self
            .engine
            .send_planner_action_async(&system, &user, Some(capture.clone()))
            .await?;

        if is_blank_model_reply(&response)
            || self
                .engine
                .parse_initial_action_decision(&response)
                .is_err()
        {
            event_sink.emit(TurnEvent::Fallback {
                stage: "initial-action-retry".to_string(),
                reason: "missing or invalid initial action response; asking the planner to restate the action inside the harness state space".to_string(),
            });
            user = build_http_initial_action_retry_prompt(request, self.engine.format);
            capture = self.exchange_capture(
                event_sink.as_ref(),
                TraceModelExchangeCategory::InitialAction,
            );
            (response, raw_response_artifact_id) = self
                .engine
                .send_planner_action_async(&system, &user, Some(capture.clone()))
                .await?;
        }

        if is_blank_model_reply(&response)
            || self
                .engine
                .parse_initial_action_decision(&response)
                .is_err()
        {
            event_sink.emit(TurnEvent::Fallback {
                stage: "initial-action-redecision".to_string(),
                reason: "asking the planner for one final constrained initial action inside the harness state space".to_string(),
            });
            user =
                build_http_initial_action_redecision_prompt(request, &response, self.engine.format);
            capture = self.exchange_capture(
                event_sink.as_ref(),
                TraceModelExchangeCategory::InitialAction,
            );
            (response, raw_response_artifact_id) = self
                .engine
                .send_planner_action_async(&system, &user, Some(capture.clone()))
                .await?;
        }

        match self.engine.parse_initial_action_decision(&response) {
            Ok(decision) => {
                let rendered = json!({
                    "action": decision.action.summary(),
                    "rationale": decision.rationale,
                    "edit": if decision.edit.known_edit { "yes" } else { "no" },
                    "candidate_files": decision.edit.candidate_files,
                })
                .to_string();
                self.engine
                    .record_rendered_response(&capture, &rendered, raw_response_artifact_id);
                Ok(decision)
            }
            Err(_) => {
                event_sink.emit(TurnEvent::Fallback {
                    stage: "initial-action-fallback".to_string(),
                    reason:
                        "controller failed closed after repeated invalid initial-action replies"
                            .to_string(),
                });
                Ok(fail_closed_http_initial_action(request))
            }
        }
    }

    async fn select_next_action(
        &self,
        request: &PlannerRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<RecursivePlannerDecision> {
        let system = self
            .engine
            .build_planner_system_prompt(&request.interpretation);
        let mut user = build_http_planner_action_prompt(request, self.engine.format);
        let mut capture = self.exchange_capture(
            event_sink.as_ref(),
            TraceModelExchangeCategory::PlannerAction,
        );
        let (mut response, mut raw_response_artifact_id) = self
            .engine
            .send_planner_action_async(&system, &user, Some(capture.clone()))
            .await?;

        if is_blank_model_reply(&response) || self.engine.parse_planner_action(&response).is_err() {
            event_sink.emit(TurnEvent::Fallback {
                stage: "planner-retry".to_string(),
                reason: "missing or invalid planner action response; asking the planner to restate the next action inside the harness state space".to_string(),
            });
            user = build_http_planner_retry_prompt(request, self.engine.format);
            capture = self.exchange_capture(
                event_sink.as_ref(),
                TraceModelExchangeCategory::PlannerAction,
            );
            (response, raw_response_artifact_id) = self
                .engine
                .send_planner_action_async(&system, &user, Some(capture.clone()))
                .await?;
        }

        if is_blank_model_reply(&response) || self.engine.parse_planner_action(&response).is_err() {
            event_sink.emit(TurnEvent::Fallback {
                stage: "planner-redecision".to_string(),
                reason: "asking the planner for one final constrained next action inside the harness state space".to_string(),
            });
            user = build_http_planner_redecision_prompt(request, &response, self.engine.format);
            capture = self.exchange_capture(
                event_sink.as_ref(),
                TraceModelExchangeCategory::PlannerAction,
            );
            (response, raw_response_artifact_id) = self
                .engine
                .send_planner_action_async(&system, &user, Some(capture.clone()))
                .await?;
        }

        let decision = match self.engine.parse_planner_action(&response) {
            Ok(decision) => decision,
            Err(_) => {
                event_sink.emit(TurnEvent::Fallback {
                    stage: "planner-fallback".to_string(),
                    reason: "controller failed closed after repeated invalid planner replies"
                        .to_string(),
                });
                return Ok(fail_closed_http_planner_action());
            }
        };
        let rendered = json!({
            "action": decision.action.summary(),
            "rationale": decision.rationale,
        })
        .to_string();
        self.engine
            .record_rendered_response(&capture, &rendered, raw_response_artifact_id);
        Ok(decision)
    }

    async fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
        _event_sink: Arc<dyn TurnEventSink>,
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
                    reason: "Archived due to context strain".to_string(),
                }
            };
            decisions.insert(artifact_id.clone(), decision);
        }

        Ok(CompactionPlan { decisions })
    }
}

fn redacted_http_request_snapshot(url: &str, headers: &[(&str, &str)], body: &Value) -> String {
    serde_json::to_string_pretty(&json!({
        "url": redact_url_secrets(url),
        "headers": redact_headers(headers),
        "body": redact_secretish_json(body),
    }))
    .unwrap_or_else(|_| "{}".to_string())
}

fn redact_headers(headers: &[(&str, &str)]) -> BTreeMap<String, String> {
    headers
        .iter()
        .map(|(name, value)| {
            let lower = name.to_ascii_lowercase();
            let redacted = if matches!(
                lower.as_str(),
                "authorization" | "x-api-key" | "api-key" | "proxy-authorization"
            ) {
                "[redacted]".to_string()
            } else {
                (*value).to_string()
            };
            (lower, redacted)
        })
        .collect()
}

fn redact_url_secrets(url: &str) -> String {
    match reqwest::Url::parse(url) {
        Ok(parsed) => {
            let query_pairs = parsed
                .query_pairs()
                .map(|(key, value)| {
                    let key_string = key.to_string();
                    let redacted = if is_secretish_key(&key_string) {
                        "[redacted]".to_string()
                    } else {
                        value.to_string()
                    };
                    (key_string, redacted)
                })
                .collect::<Vec<_>>();

            let mut base = parsed.clone();
            base.set_query(None);
            let fragment = parsed
                .fragment()
                .map(|fragment| format!("#{fragment}"))
                .unwrap_or_default();
            let mut rendered = base.to_string();
            if !query_pairs.is_empty() {
                rendered.push('?');
                rendered.push_str(
                    &query_pairs
                        .iter()
                        .map(|(key, value)| format!("{key}={value}"))
                        .collect::<Vec<_>>()
                        .join("&"),
                );
            }
            if !fragment.is_empty() && !rendered.ends_with(&fragment) {
                rendered.push_str(&fragment);
            }
            rendered
        }
        Err(_) => url.to_string(),
    }
}

fn redact_secretish_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    let redacted = if is_secretish_key(key) {
                        Value::String("[redacted]".to_string())
                    } else {
                        redact_secretish_json(value)
                    };
                    (key.clone(), redacted)
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_secretish_json).collect()),
        _ => value.clone(),
    }
}

fn is_secretish_key(key: &str) -> bool {
    matches!(
        key.to_ascii_lowercase().as_str(),
        "authorization"
            | "x-api-key"
            | "api-key"
            | "api_key"
            | "apikey"
            | "access_token"
            | "token"
            | "secret"
            | "password"
            | "key"
    )
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
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Deserialize)]
struct OpenAiToolCall {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    function: OpenAiFunctionCall,
}

#[derive(Deserialize)]
struct OpenAiFunctionCall {
    name: String,
    arguments: String,
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
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    rationale: Option<String>,
    #[serde(default)]
    answer: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default)]
    query: Option<String>,
    #[serde(default)]
    mode: Option<RetrievalMode>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    command: Option<String>,
    #[serde(default)]
    pattern: Option<String>,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    old: Option<String>,
    #[serde(default)]
    new: Option<String>,
    #[serde(default)]
    replace_all: Option<bool>,
    #[serde(default)]
    patch: Option<String>,
    #[serde(default)]
    branches: Option<Vec<String>>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    strategy: Option<RetrievalStrategy>,
    #[serde(default)]
    retrievers: Option<Vec<RetrieverOption>>,
    #[serde(default)]
    edit: Option<String>,
    #[serde(default)]
    candidate_files: Option<Vec<String>>,
    #[serde(default)]
    grounding: Option<GroundingRequirement>,
}

fn planner_action_json_schema() -> Value {
    let mut schema = json!({
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "action": {
                "type": "string",
                "enum": [
                    "answer",
                    "search",
                    "list_files",
                    "read",
                    "inspect",
                    "shell",
                    "diff",
                    "write_file",
                    "replace_in_file",
                    "apply_patch",
                    "refine",
                    "branch",
                    "stop"
                ],
                "description": "The next planner action."
            },
            "rationale": {
                "type": "string",
                "description": "Short reason for the selected action."
            },
            "answer": {
                "type": "string",
                "description": "User-facing answer text when the planner is ending the loop with a direct answer."
            },
            "reason": {
                "type": "string",
                "description": "Required when `action` is `stop`."
            },
            "query": {
                "type": "string",
                "description": "Search query when `action` is `search`."
            },
            "mode": {
                "type": "string",
                "enum": ["linear", "graph"],
                "description": "Optional retrieval mode when `action` is `search`."
            },
            "strategy": {
                "type": "string",
                "enum": ["bm25", "vector"],
                "description": "Optional retrieval strategy when `action` is `search`."
            },
            "retrievers": {
                "type": "array",
                "items": {
                    "type": "string",
                    "enum": ["path-fuzzy", "segment-fuzzy"]
                },
                "description": "Optional structural fuzzy retriever overrides when `action` is `search` or `refine`."
            },
            "intent": {
                "type": "string",
                "description": "Optional search intent when `action` is `search`."
            },
            "pattern": {
                "type": "string",
                "description": "Required when `action` is `list_files`."
            },
            "path": {
                "type": "string",
                "description": "Required when `action` targets a specific file."
            },
            "command": {
                "type": "string",
                "description": "Required when `action` is `inspect` or `shell`."
            },
            "content": {
                "type": "string",
                "description": "Required when `action` is `write_file`."
            },
            "old": {
                "type": "string",
                "description": "Required when `action` is `replace_in_file`."
            },
            "new": {
                "type": "string",
                "description": "Required when `action` is `replace_in_file`."
            },
            "replace_all": {
                "type": "boolean",
                "description": "Optional when `action` is `replace_in_file`."
            },
            "patch": {
                "type": "string",
                "description": "Required when `action` is `apply_patch`."
            },
            "branches": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Required when `action` is `branch`."
            },
            "edit": {
                "type": "string",
                "enum": ["yes", "no"],
                "description": "Optional known-edit routing hint for initial actions."
            },
            "candidate_files": {
                "type": "array",
                "items": { "type": "string" },
                "description": "Optional likely edit targets for initial actions."
            },
            "grounding": {
                "type": "object",
                "additionalProperties": false,
                "description": "Optional grounding contract when the final answer requires verified repository or external evidence.",
                "properties": {
                    "domain": {
                        "type": "string",
                        "enum": ["repository", "external", "mixed"],
                        "description": "Evidence domain required before the final answer can be trusted."
                    },
                    "reason": {
                        "type": "string",
                        "description": "Short planner rationale for why grounding is required."
                    }
                },
                "required": ["domain", "reason"]
            }
        },
        "required": ["action", "rationale"]
    });

    for property in [
        "answer",
        "reason",
        "query",
        "mode",
        "strategy",
        "retrievers",
        "intent",
        "pattern",
        "path",
        "command",
        "content",
        "old",
        "new",
        "replace_all",
        "patch",
        "branches",
        "edit",
        "candidate_files",
        "grounding",
    ] {
        make_openai_schema_property_nullable(&mut schema, property);
    }
    make_openai_object_schema_require_all_properties(&mut schema);

    schema
}

fn make_openai_schema_property_nullable(schema: &mut Value, property_name: &str) {
    let Some(property) = schema
        .get_mut("properties")
        .and_then(Value::as_object_mut)
        .and_then(|properties| properties.get_mut(property_name))
    else {
        return;
    };

    let Some(object) = property.as_object_mut() else {
        return;
    };

    if let Some(property_type) = object.get_mut("type") {
        match property_type {
            Value::String(existing) => {
                let existing = existing.clone();
                *property_type = json!([existing, "null"]);
            }
            Value::Array(existing) => {
                if !existing.iter().any(|value| value.as_str() == Some("null")) {
                    existing.push(Value::String("null".to_string()));
                }
            }
            _ => {}
        }
    }

    if let Some(enum_values) = object.get_mut("enum").and_then(Value::as_array_mut)
        && !enum_values.iter().any(Value::is_null)
    {
        enum_values.push(Value::Null);
    }
}

fn make_openai_object_schema_require_all_properties(schema: &mut Value) {
    let Some(properties) = schema.get("properties").and_then(Value::as_object) else {
        return;
    };

    let required = properties
        .keys()
        .cloned()
        .map(Value::String)
        .collect::<Vec<_>>();

    if let Some(object) = schema.as_object_mut() {
        object.insert("required".to_string(), Value::Array(required));
    }
}

fn build_http_planner_runtime_context(request: &PlannerRequest) -> String {
    let mut context = format!(
        "Runtime context:\n\
- You are the remote planner model inside Paddles.\n\
- Paddles executes the action you choose locally in the workspace at `{}`.\n\
- Treat user-reported failures or broken states as working hypotheses until local evidence confirms them.\n\
- As evidence accumulates, revise the premise explicitly when commands weaken or contradict it.\n\
- Use local harness capabilities before asking the user for logs or repository state that the workspace can reveal.\n",
        request.workspace_root.display()
    );

    if !request.recent_turns.is_empty() {
        context.push_str("\nRecent conversation:\n");
        for turn in &request.recent_turns {
            context.push_str("- ");
            context.push_str(turn);
            context.push('\n');
        }
    }

    if let Some(summary) = request
        .recent_thread_summary
        .as_deref()
        .filter(|summary| !summary.trim().is_empty())
    {
        context.push_str("\nActive thread summary:\n");
        context.push_str(summary);
        context.push('\n');
    }

    if !request.runtime_notes.is_empty() {
        context.push_str("\nRuntime notes:\n");
        for note in &request.runtime_notes {
            context.push_str("- ");
            context.push_str(note);
            context.push('\n');
        }
    }

    context
}

fn build_http_planner_loop_state_digest(request: &PlannerRequest) -> String {
    let mut lines = vec![format!(
        "Budget remaining: steps={}, evidence_limit={}, pending_branches={}",
        request
            .budget
            .max_steps
            .saturating_sub(request.loop_state.steps.len()),
        request.budget.max_evidence_items,
        request.loop_state.pending_branches.len()
    )];

    if request.loop_state.steps.is_empty() {
        lines.push("No planner steps have executed yet.".to_string());
    } else {
        for step in &request.loop_state.steps {
            lines.push(format!(
                "- Step {}: {} -> {}",
                step.sequence,
                step.action.summary(),
                truncate(&step.outcome, 180)
            ));
        }
    }

    if !request.loop_state.evidence_items.is_empty() {
        lines.push("Current evidence:".to_string());
        for item in request
            .loop_state
            .evidence_items
            .iter()
            .take(request.budget.max_evidence_items.min(4))
        {
            lines.push(format!(
                "- {}: {}",
                item.source,
                truncate(&item.snippet, 180)
            ));
        }
    }

    if !request.loop_state.notes.is_empty() {
        lines.push("Current notes:".to_string());
        for note in request.loop_state.notes.iter().take(4) {
            lines.push(format!("- {}", truncate(note, 220)));
        }
    }

    lines.join("\n")
}

fn build_http_initial_action_prompt(request: &PlannerRequest, format: ApiFormat) -> String {
    format!(
        "User prompt: {}\n\n{}\nSelect your first action.\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
If the user is asking to debug a repository failure like CI, build, test, workflow, or lint breakage, do not answer directly before at least one local workspace action unless the interpretation context already contains the exact failure evidence.\n\
When the user is asking for a code or file change, set `edit` to `yes` and include up to 3 plausible relative paths in `candidate_files`.\n\
If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
{}",
        request.user_prompt,
        build_http_planner_runtime_context(request),
        planner_transport_reply_instruction(format)
    )
}

fn build_http_initial_action_retry_prompt(request: &PlannerRequest, format: ApiFormat) -> String {
    format!(
        "Your last planner reply was empty or invalid.\n\
{}\n\
{}\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
If the user is asking to debug a repository failure, prefer a local workspace action over `answer`.\n\
If the user is asking for a code or file change, include `edit` and `candidate_files` in the JSON envelope.\n\
If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
\n\
User prompt: {}",
        build_http_planner_runtime_context(request),
        planner_transport_retry_instruction(format, true),
        request.user_prompt
    )
}

fn build_http_initial_action_redecision_prompt(
    request: &PlannerRequest,
    invalid_reply: &str,
    format: ApiFormat,
) -> String {
    format!(
        "Your previous initial-action replies were invalid.\n\
{}\n\
Make one final constrained routing decision.\n\
{}\n\
Do not ask the user for logs or repository state that the harness can inspect locally.\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
If the user is asking to debug a repository failure, prefer a local workspace action over `answer`.\n\
If the user is asking for a code or file change, include `edit` and `candidate_files` in the JSON envelope.\n\
If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
\n\
Invalid reply to correct:\n\
{}\n\
\n\
User prompt: {}",
        build_http_planner_runtime_context(request),
        planner_transport_retry_instruction(format, true),
        truncate(invalid_reply, 800),
        request.user_prompt
    )
}

fn build_http_planner_action_prompt(request: &PlannerRequest, format: ApiFormat) -> String {
    let steps_used = request.loop_state.steps.len();
    let steps_remaining = request.budget.max_steps.saturating_sub(steps_used);
    let mut user = format!(
        "User prompt: {}\n\n{}\nBudget: {steps_used}/{} steps used, {steps_remaining} remaining.\n",
        request.user_prompt,
        build_http_planner_runtime_context(request),
        request.budget.max_steps
    );
    user.push_str("\n## Current loop state\n");
    user.push_str(&build_http_planner_loop_state_digest(request));
    user.push('\n');
    user.push_str(&format!(
        "\nSelect your next action.\n\
Choose `stop` as soon as you have enough evidence, but do not leave the harness state space by answering the user in prose.\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
Use `diff`, `write_file`, `replace_in_file`, or `apply_patch` when a concrete edit should happen now instead of more research.\n\
If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
If the loop-state notes contain a steering review, judge the proposed next move against the gathered sources and return the action that should actually execute next.\n\
{}",
        planner_transport_reply_instruction(format)
    ));
    user
}

fn build_http_planner_retry_prompt(request: &PlannerRequest, format: ApiFormat) -> String {
    format!(
        "Your last planner reply was empty or invalid.\n\
{}\n\
Current loop state:\n\
{}\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
{}\n\
\n\
Current user request: {}",
        build_http_planner_runtime_context(request),
        build_http_planner_loop_state_digest(request),
        planner_transport_retry_instruction(format, false),
        request.user_prompt
    )
}

fn build_http_planner_redecision_prompt(
    request: &PlannerRequest,
    invalid_reply: &str,
    format: ApiFormat,
) -> String {
    format!(
        "Your previous planner replies were invalid.\n\
{}\n\
Current loop state:\n\
{}\n\
Make one final constrained next-action decision.\n\
If the user asks for a safe, reasonable repository change, the purpose of this coding assistant is to make the workspace edit in this turn rather than stop at diagnosis or advice.\n\
If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
{}\n\
\n\
Invalid reply to correct:\n\
{}\n\
\n\
        Current user request: {}",
        build_http_planner_runtime_context(request),
        build_http_planner_loop_state_digest(request),
        planner_transport_retry_instruction(format, false),
        truncate(invalid_reply, 800),
        request.user_prompt
    )
}

fn is_blank_model_reply(reply: &str) -> bool {
    reply.trim().is_empty()
}

fn fail_closed_http_initial_action(request: &PlannerRequest) -> InitialActionDecision {
    InitialActionDecision {
        action: InitialAction::Stop {
            reason: format!(
                "initial-action-unavailable after invalid planner replies for `{}`",
                truncate(&request.user_prompt, 120)
            ),
        },
        rationale: "controller failed closed after repeated invalid initial-action replies"
            .to_string(),
        answer: None,
        edit: InitialEditInstruction::default(),
        grounding: None,
    }
}

fn fail_closed_http_planner_action() -> RecursivePlannerDecision {
    RecursivePlannerDecision {
        action: PlannerAction::Stop {
            reason: "planner-action-unavailable after invalid planner replies".to_string(),
        },
        rationale: "controller failed closed after repeated invalid planner replies".to_string(),
        answer: None,
        edit: InitialEditInstruction::default(),
        grounding: None,
    }
}

fn infer_planner_action_name(envelope: &PlannerEnvelope) -> Option<String> {
    if let Some(action) = envelope.action.clone() {
        return Some(action);
    }

    if let Some(command) = envelope.command.as_deref() {
        return Some(if command_looks_read_only(command) {
            "inspect".to_string()
        } else {
            "shell".to_string()
        });
    }
    if envelope.patch.is_some() {
        return Some("apply_patch".to_string());
    }
    if envelope.content.is_some() && envelope.path.is_some() {
        return Some("write_file".to_string());
    }
    if envelope.old.is_some() && envelope.new.is_some() && envelope.path.is_some() {
        return Some("replace_in_file".to_string());
    }
    if envelope.path.is_some() {
        return Some("read".to_string());
    }
    if envelope.pattern.is_some() {
        return Some("list_files".to_string());
    }
    if envelope.query.is_some() {
        return Some("search".to_string());
    }
    if envelope.branches.is_some() {
        return Some("branch".to_string());
    }
    if envelope.reason.is_some() {
        return Some("stop".to_string());
    }

    None
}

fn command_looks_read_only(command: &str) -> bool {
    let normalized = command.trim();
    !normalized.is_empty()
        && !normalized.contains("&&")
        && !normalized.contains("||")
        && !normalized.contains(';')
        && !normalized.contains('>')
        && !normalized.contains('<')
}

fn extract_json(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.starts_with('{') {
        if let Some(end) = trimmed.rfind('}') {
            return Some(&trimmed[..=end]);
        }
        return Some(trimmed);
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

fn planner_transport_reply_instruction(format: ApiFormat) -> &'static str {
    match format {
        ApiFormat::OpenAi => {
            "Use the `select_planner_action` tool exactly once. Put the action envelope in the tool arguments and do not answer in prose."
        }
        ApiFormat::Gemini | ApiFormat::Anthropic => "Respond with JSON only.",
    }
}

fn planner_transport_retry_instruction(format: ApiFormat, initial: bool) -> String {
    let label = if initial {
        "initial action"
    } else {
        "planner action"
    };
    match format {
        ApiFormat::OpenAi => format!(
            "Use the `select_planner_action` tool exactly once with one valid {label}.\nDo not answer the user directly in prose.\nThe tool arguments must include `action`."
        ),
        ApiFormat::Gemini | ApiFormat::Anthropic => format!(
            "Return ONLY one valid JSON {label}.\nDo not answer the user directly in prose.\nThe first key must be `action`."
        ),
    }
}

fn edit_instruction_from_http_envelope(
    envelope: &PlannerEnvelope,
) -> Result<InitialEditInstruction> {
    let edit_value = envelope
        .edit
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("initial action reply must include top-level `edit`"))?;
    let known_edit = match edit_value {
        "yes" | "true" => true,
        "no" | "false" => false,
        other => bail!("edit must be `yes` or `no`, got `{other}`"),
    };
    let candidate_files = envelope
        .candidate_files
        .as_ref()
        .ok_or_else(|| anyhow!("initial action reply must include top-level `candidate_files`"))?
        .iter()
        .map(|path| path.trim().replace('\\', "/"))
        .filter(|path| !path.is_empty())
        .fold(Vec::new(), |mut deduped, path| {
            if !deduped.contains(&path) {
                deduped.push(path);
            }
            deduped
        })
        .into_iter()
        .take(3)
        .collect::<Vec<_>>();
    if known_edit && candidate_files.is_empty() {
        bail!("candidate_files must contain at least one file when edit is `yes`");
    }

    Ok(InitialEditInstruction {
        known_edit,
        candidate_files,
        resolution: None,
    })
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

fn grounded_response_defers_executed_command_work(reply: &str, evidence: &EvidenceBundle) -> bool {
    if !evidence
        .items
        .iter()
        .any(|item| item.source.starts_with("command: "))
    {
        return false;
    }

    let normalized = reply.trim().to_ascii_lowercase();
    normalized.starts_with("i will ")
        || normalized.starts_with("i'll ")
        || normalized.starts_with("let me ")
        || normalized.starts_with("use the github cli")
        || normalized.contains("didn't produce any output")
        || normalized.contains("did not produce any output")
        || normalized.contains("failed silently")
        || normalized.contains("execution environment is not available")
        || normalized.contains("repository is cloned")
        || normalized.contains("i have access to the gh cli")
        || normalized.contains("i have access to the source code on this machine")
        || normalized.contains("please provide the specific error")
        || normalized.contains("please share the error")
        || normalized.contains("please provide the error log")
        || normalized.contains("could you please confirm")
}

fn prompt_asserts_failure_premise(prompt: &str) -> bool {
    let prompt_lower = prompt.to_ascii_lowercase();
    [
        "failing",
        "failure",
        "broken",
        "regression",
        "panic",
        "crash",
        "workflow",
        "pipeline",
        "ci ",
        " ci",
        "build failing",
        "tests failing",
        "test failing",
        "lint failing",
    ]
    .iter()
    .any(|signal| prompt_lower.contains(signal))
}

fn reply_asserts_failure_as_fact(reply: &str) -> bool {
    let normalized = reply.to_ascii_lowercase();
    normalized.contains(" is failing")
        || normalized.contains(" are failing")
        || normalized.contains("currently failing")
        || normalized.contains("pipeline is failing")
        || normalized.contains("workflow is failing")
        || normalized.contains("build is failing")
        || normalized.contains("test suite is failing")
        || normalized.contains("tests are failing")
}

fn evidence_confirms_failure_premise(evidence: &EvidenceBundle) -> bool {
    evidence.items.iter().any(|item| {
        let snippet = item.snippet.to_ascii_lowercase();
        snippet.contains("completed\tfailure")
            || snippet.contains("\"conclusion\":\"failure\"")
            || snippet.contains("\"status\":\"failure\"")
            || snippet.contains("test result: failed")
            || snippet.contains("build failed")
            || snippet.contains("error[")
            || (snippet.contains("thread '") && snippet.contains("panicked"))
            || snippet.contains("failing test")
            || snippet.contains("failed tests:")
            || snippet.contains("failures:")
    })
}

fn grounded_response_overstates_unverified_failure(
    reply: &str,
    prompt: &str,
    evidence: &EvidenceBundle,
) -> bool {
    prompt_asserts_failure_premise(prompt)
        && reply_asserts_failure_as_fact(reply)
        && !evidence_confirms_failure_premise(evidence)
}

fn grounded_command_evidence_fallback(evidence: &EvidenceBundle) -> String {
    let mut lines = vec![
        "I already ran local inspection commands in the harness. Here is what they showed:"
            .to_string(),
        evidence.summary.clone(),
    ];

    let mut added = 0usize;
    for item in evidence
        .items
        .iter()
        .filter(|item| item.source.starts_with("command: "))
    {
        lines.push(format!(
            "- {}: {}",
            item.source,
            truncate(item.snippet.trim(), 180)
        ));
        added += 1;
        if added >= 4 {
            break;
        }
    }

    if added == 0 {
        for item in evidence.items.iter().take(4) {
            lines.push(format!(
                "- {}: {}",
                item.source,
                truncate(item.snippet.trim(), 180)
            ));
        }
    }

    lines.join("\n")
}

fn grounded_unverified_failure_fallback(evidence: &EvidenceBundle) -> String {
    let mut lines = vec![
        "The reported failure is not yet confirmed by the gathered evidence. Here is what the harness actually observed:".to_string(),
        evidence.summary.clone(),
    ];

    for item in evidence.items.iter().take(4) {
        lines.push(format!(
            "- {}: {}",
            item.source,
            truncate(item.snippet.trim(), 180)
        ));
    }

    lines.join("\n")
}

fn format_grounding_contract(grounding: &GroundingRequirement) -> String {
    let mut lines = Vec::new();
    match grounding.domain {
        GroundingDomain::Repository => {
            lines.push("Repository grounding is required for this turn.".to_string());
            lines.push(
                "Do not invent repository-specific claims without attached evidence.".to_string(),
            );
        }
        GroundingDomain::External => {
            lines.push("External source grounding is required for this turn.".to_string());
        }
        GroundingDomain::Mixed => {
            lines.push(
                "Repository and external grounding are both required for this turn.".to_string(),
            );
            lines.push(
                "Do not invent repository-specific claims without attached evidence.".to_string(),
            );
        }
    }
    if grounding.requires_external() {
        lines.push(
            "Do not emit package names, website names, or external URLs unless they are verified by attached evidence."
                .to_string(),
        );
        lines.push(
            "If no verified external source is attached, say that you cannot verify a web link from this environment."
                .to_string(),
        );
    }
    if let Some(reason) = grounding
        .reason
        .as_deref()
        .filter(|reason| !reason.trim().is_empty())
    {
        lines.push(format!("Reason: {}", reason.trim()));
    }
    lines.join("\n")
}

fn verified_external_urls_from_evidence(evidence: &EvidenceBundle) -> Vec<String> {
    let mut urls = BTreeSet::new();
    for url in extract_http_urls(&evidence.summary) {
        urls.insert(url);
    }
    for item in &evidence.items {
        for field in [&item.source, &item.snippet, &item.rationale] {
            for url in extract_http_urls(field) {
                urls.insert(url);
            }
        }
    }
    urls.into_iter().collect()
}

fn first_unverified_external_url(reply: &str, verified_external_urls: &[String]) -> Option<String> {
    let verified = verified_external_urls
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    extract_http_urls(reply)
        .into_iter()
        .find(|url| !verified.contains(url.as_str()))
}

fn external_grounding_required_without_verified_sources(
    handoff: &SynthesisHandoff,
    verified_external_urls: &[String],
) -> bool {
    handoff
        .grounding
        .as_ref()
        .is_some_and(|grounding| grounding.requires_external() && verified_external_urls.is_empty())
}

fn external_grounding_unavailable_fallback(prompt: &str) -> String {
    format!(
        "I can't provide a verified external link or source for `{}` because this turn has no verified external sources attached.",
        prompt.trim()
    )
}

fn unverified_external_url_fallback(prompt: &str) -> String {
    format!(
        "I can't provide a verified external link for `{}` because the drafted answer included an unverified URL.",
        prompt.trim()
    )
}

#[cfg(test)]
mod tests {
    use super::{ApiFormat, HttpPlannerAdapter, HttpProviderAdapter};
    use crate::application::{MechSuitService, RuntimeLaneConfig};
    use crate::domain::model::{
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase,
        TraceRecordKind, TurnEvent, TurnEventSink, TurnIntent,
    };
    use crate::domain::ports::{
        EvidenceBundle, EvidenceItem, GroundingDomain, GroundingRequirement, InitialAction,
        InitialEditInstruction, InterpretationContext, ModelPaths, ModelRegistry, PlannerAction,
        PlannerBudget, PlannerLoopState, PlannerRequest, RecursivePlanner,
        RecursivePlannerDecision, RetrieverOption, SynthesisHandoff, SynthesizerEngine,
        TraceRecorder, WorkspaceAction,
    };
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
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

    fn openai_tool_call_response(name: &str, arguments: &str) -> String {
        json!({
            "choices": [
                {
                    "message": {
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_test",
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": arguments
                                }
                            }
                        ]
                    }
                }
            ]
        })
        .to_string()
    }

    fn planner_json_answer() -> String {
        json!({
            "action": "answer",
            "edit": "no",
            "candidate_files": [],
            "rationale": "the mock planner can answer directly"
        })
        .to_string()
    }

    fn http_test_service(
        workspace: &Path,
        base_url: String,
        api_key: String,
        provider: crate::infrastructure::providers::ModelProvider,
        format: ApiFormat,
    ) -> MechSuitService {
        let operator_memory = Arc::new(AgentMemory::load(workspace));

        let synth_base_url = base_url.clone();
        let synth_api_key = api_key.clone();
        let synth_provider_name = provider.name().to_string();
        let synthesizer_factory: Box<crate::application::SynthesizerFactory> = Box::new(
            move |workspace: &Path, lane: &crate::application::PreparedModelLane| {
                Ok(Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    synth_provider_name.clone(),
                    lane.model_id.clone(),
                    synth_api_key.clone(),
                    synth_base_url.clone(),
                    format,
                    render_capability_for(format),
                )) as Arc<dyn SynthesizerEngine>)
            },
        );

        let planner_base_url = base_url;
        let planner_api_key = api_key;
        let planner_provider_name = provider.name().to_string();
        let planner_factory: Box<crate::application::PlannerFactory> = Box::new(
            move |workspace: &Path, lane: &crate::application::PreparedModelLane| {
                let engine = Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    planner_provider_name.clone(),
                    lane.model_id.clone(),
                    planner_api_key.clone(),
                    planner_base_url.clone(),
                    format,
                    render_capability_for(format),
                ));
                Ok(Arc::new(HttpPlannerAdapter::new(engine)) as Arc<dyn RecursivePlanner>)
            },
        );

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

    fn http_test_service_with_recorder(
        workspace: &Path,
        base_url: String,
        api_key: String,
        provider: crate::infrastructure::providers::ModelProvider,
        format: ApiFormat,
        recorder: Arc<dyn crate::domain::ports::TraceRecorder>,
    ) -> MechSuitService {
        let operator_memory = Arc::new(AgentMemory::load(workspace));

        let synth_base_url = base_url.clone();
        let synth_api_key = api_key.clone();
        let synth_provider_name = provider.name().to_string();
        let synthesizer_factory: Box<crate::application::SynthesizerFactory> = Box::new(
            move |workspace: &Path, lane: &crate::application::PreparedModelLane| {
                Ok(Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    synth_provider_name.clone(),
                    lane.model_id.clone(),
                    synth_api_key.clone(),
                    synth_base_url.clone(),
                    format,
                    render_capability_for(format),
                )) as Arc<dyn SynthesizerEngine>)
            },
        );

        let planner_base_url = base_url;
        let planner_api_key = api_key;
        let planner_provider_name = provider.name().to_string();
        let planner_factory: Box<crate::application::PlannerFactory> = Box::new(
            move |workspace: &Path, lane: &crate::application::PreparedModelLane| {
                let engine = Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    planner_provider_name.clone(),
                    lane.model_id.clone(),
                    planner_api_key.clone(),
                    planner_base_url.clone(),
                    format,
                    render_capability_for(format),
                ));
                Ok(Arc::new(HttpPlannerAdapter::new(engine)) as Arc<dyn RecursivePlanner>)
            },
        );

        let gatherer_factory: Box<crate::application::GathererFactory> =
            Box::new(|_, _, _, _| Ok::<Option<_>, anyhow::Error>(None));

        MechSuitService::with_trace_recorder(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            synthesizer_factory,
            planner_factory,
            gatherer_factory,
            recorder,
        )
    }

    async fn run_mocked_turn(
        format: ApiFormat,
        provider: crate::infrastructure::providers::ModelProvider,
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
            provider,
            format,
        );
        let runtime_lanes = RuntimeLaneConfig::new(model_id.to_string(), None)
            .with_synthesizer_provider(provider)
            .with_planner_provider(Some(provider))
            .with_planner_model_id(Some(model_id.to_string()));
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
            crate::infrastructure::providers::ModelProvider::Openai,
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
            requests[0].body["messages"][0]["content"]
                .as_str()
                .expect("planner system prompt")
                .contains("select_planner_action")
        );
        assert_eq!(
            requests[0].body["tools"][0]["type"].as_str(),
            Some("function")
        );
        assert_eq!(
            requests[0].body["tools"][0]["function"]["name"].as_str(),
            Some(super::OPENAI_PLANNER_TOOL_NAME)
        );
        assert_eq!(
            requests[0].body["tools"][0]["function"]["strict"].as_bool(),
            Some(true)
        );
        assert_eq!(
            requests[0].body["tool_choice"]["type"].as_str(),
            Some("function")
        );
        assert_eq!(
            requests[0].body["tool_choice"]["function"]["name"].as_str(),
            Some(super::OPENAI_PLANNER_TOOL_NAME)
        );
        assert_eq!(
            requests[0].body["max_completion_tokens"].as_i64(),
            Some(4096)
        );
        assert!(requests[0].body.get("max_tokens").is_none());
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
        assert_eq!(
            requests[1].body["max_completion_tokens"].as_i64(),
            Some(4096)
        );
        assert!(requests[1].body.get("max_tokens").is_none());
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
    async fn inception_provider_executes_a_full_turn_against_a_mock_server() {
        let model_id = "mercury-2";
        let (requests, events, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            crate::infrastructure::providers::ModelProvider::Inception,
            model_id,
            &structured_answer_json("Mocked final answer."),
        )
        .await;

        assert_eq!(response, "Mocked final answer.");
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[0].uri, "/v1/chat/completions");
        assert_eq!(requests[1].uri, "/v1/chat/completions");
        assert_eq!(requests[0].body["model"].as_str(), Some(model_id));
        assert_eq!(
            requests[0].body["tools"][0]["type"].as_str(),
            Some("function")
        );
        assert_eq!(
            requests[0].body["tools"][0]["function"]["name"].as_str(),
            Some(super::OPENAI_PLANNER_TOOL_NAME)
        );
        assert_eq!(
            requests[0].body["tool_choice"]["function"]["name"].as_str(),
            Some(super::OPENAI_PLANNER_TOOL_NAME)
        );
        assert_eq!(
            requests[1].body["response_format"]["type"].as_str(),
            Some("json_schema")
        );
        assert!(events.iter().any(|event| matches!(
            event,
            TurnEvent::PlannerCapability { provider, .. } if provider == model_id
        )));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_provider_records_exact_forensic_exchange_artifacts_in_trace_replay() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse the remote provider planner before answering.\n",
        )
        .expect("write AGENTS.md");

        let model_id = "gpt-4o-mini";
        let structured = structured_answer_json("Mocked final answer.");
        let raw_final_response = final_answer_response(ApiFormat::OpenAi, &structured);
        let planner_response = provider_response(ApiFormat::OpenAi, &planner_json_answer());
        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: planner_response.clone(),
            },
            MockResponse {
                status: StatusCode::OK,
                body: raw_final_response.clone(),
            },
        ])
        .await;

        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = http_test_service_with_recorder(
            workspace.path(),
            server.base_url.clone(),
            "test-key".to_string(),
            crate::infrastructure::providers::ModelProvider::Openai,
            ApiFormat::OpenAi,
            recorder.clone(),
        );
        let runtime_lanes = RuntimeLaneConfig::new(model_id.to_string(), None)
            .with_synthesizer_provider(crate::infrastructure::providers::ModelProvider::Openai);
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare runtime lanes");

        service
            .process_prompt("Sup dawg")
            .await
            .expect("process prompt");

        let task_id = recorder.task_ids().into_iter().next().expect("task id");
        let replay = recorder.replay(&task_id).expect("replay");
        let forensic = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::ModelExchangeArtifact(artifact) => Some(artifact),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Planner
                && artifact.category == TraceModelExchangeCategory::InitialAction
                && artifact.phase == TraceModelExchangePhase::AssembledContext
                && artifact
                    .artifact
                    .inline_content
                    .as_deref()
                    .is_some_and(|content: &str| content.contains("User prompt: Sup dawg"))
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Planner
                && artifact.category == TraceModelExchangeCategory::InitialAction
                && artifact.phase == TraceModelExchangePhase::ProviderRequest
                && artifact
                    .artifact
                    .inline_content
                    .as_deref()
                    .is_some_and(|content: &str| {
                        content.contains("\"authorization\": \"[redacted]\"")
                    })
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Planner
                && artifact.category == TraceModelExchangeCategory::InitialAction
                && artifact.phase == TraceModelExchangePhase::RawProviderResponse
                && artifact.artifact.inline_content.as_deref() == Some(planner_response.as_str())
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Synthesizer
                && artifact.category == TraceModelExchangeCategory::TurnResponse
                && artifact.phase == TraceModelExchangePhase::RawProviderResponse
                && artifact.artifact.inline_content.as_deref() == Some(raw_final_response.as_str())
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Synthesizer
                && artifact.category == TraceModelExchangeCategory::TurnResponse
                && artifact.phase == TraceModelExchangePhase::RenderedResponse
                && artifact.artifact.inline_content.as_deref() == Some("Mocked final answer.")
        }));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn inception_provider_records_exact_forensic_exchange_artifacts_in_trace_replay() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse the remote provider planner before answering.\n",
        )
        .expect("write AGENTS.md");

        let model_id = "mercury-2";
        let structured = structured_answer_json("Mocked final answer.");
        let raw_final_response = final_answer_response(ApiFormat::OpenAi, &structured);
        let planner_response = provider_response(ApiFormat::OpenAi, &planner_json_answer());
        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: planner_response.clone(),
            },
            MockResponse {
                status: StatusCode::OK,
                body: raw_final_response.clone(),
            },
        ])
        .await;

        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = http_test_service_with_recorder(
            workspace.path(),
            server.base_url.clone(),
            "test-key".to_string(),
            crate::infrastructure::providers::ModelProvider::Inception,
            ApiFormat::OpenAi,
            recorder.clone(),
        );
        let runtime_lanes = RuntimeLaneConfig::new(model_id.to_string(), None)
            .with_synthesizer_provider(crate::infrastructure::providers::ModelProvider::Inception)
            .with_planner_provider(Some(
                crate::infrastructure::providers::ModelProvider::Inception,
            ))
            .with_planner_model_id(Some(model_id.to_string()));
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare runtime lanes");

        service
            .process_prompt("Sup dawg")
            .await
            .expect("process prompt");

        let task_id = recorder.task_ids().into_iter().next().expect("task id");
        let replay = recorder.replay(&task_id).expect("replay");
        let forensic = replay
            .records
            .iter()
            .filter_map(|record| match &record.kind {
                TraceRecordKind::ModelExchangeArtifact(artifact) => Some(artifact),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Planner
                && artifact.category == TraceModelExchangeCategory::InitialAction
                && artifact.phase == TraceModelExchangePhase::ProviderRequest
                && artifact
                    .artifact
                    .inline_content
                    .as_deref()
                    .is_some_and(|content: &str| {
                        content.contains("\"authorization\": \"[redacted]\"")
                    })
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Synthesizer
                && artifact.category == TraceModelExchangeCategory::TurnResponse
                && artifact.phase == TraceModelExchangePhase::RawProviderResponse
                && artifact.artifact.inline_content.as_deref() == Some(raw_final_response.as_str())
        }));
        assert!(forensic.iter().any(|artifact| {
            artifact.lane == TraceModelExchangeLane::Synthesizer
                && artifact.category == TraceModelExchangeCategory::TurnResponse
                && artifact.phase == TraceModelExchangePhase::RenderedResponse
                && artifact.artifact.inline_content.as_deref() == Some("Mocked final answer.")
        }));
    }

    #[test]
    fn provider_request_redaction_hides_auth_headers_and_query_keys() {
        let redacted = super::redacted_http_request_snapshot(
            "http://localhost/v1beta/models/gemini:generateContent?key=secret-key",
            &[
                ("Authorization", "Bearer secret-token"),
                ("x-api-key", "abc123"),
            ],
            &json!({
                "model": "gpt-4o-mini",
                "api_key": "body-secret",
                "messages": [{"role": "user", "content": "hello"}]
            }),
        );

        assert!(redacted.contains("\"authorization\": \"[redacted]\""));
        assert!(redacted.contains("\"x-api-key\": \"[redacted]\""));
        assert!(redacted.contains("key=[redacted]"));
        assert!(redacted.contains("\"api_key\": \"[redacted]\""));
        assert!(redacted.contains("\"content\": \"hello\""));
    }

    #[test]
    fn parse_planner_action_infers_inspect_when_command_is_present_without_action() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_planner_action(
                r#"{"command":"nix build .#paddles -L","rationale":"Run the nix-build job to see why CI is failing","edit":"no","candidate_files":[]}"#,
            )
            .expect("planner decision");

        assert_eq!(
            decision,
            RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "nix build .#paddles -L".to_string(),
                    },
                },
                rationale: "Run the nix-build job to see why CI is failing".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }
        );
    }

    #[test]
    fn parse_planner_action_supports_apply_patch_actions() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_planner_action(
                r#"{"action":"apply_patch","patch":"*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-fn old() {}\n+fn new() {}\n*** End Patch\n","rationale":"apply the requested code change","edit":"no","candidate_files":[]}"#,
            )
            .expect("planner decision");

        assert_eq!(
            decision,
            RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::ApplyPatch {
                        patch: "*** Begin Patch\n*** Update File: src/lib.rs\n@@\n-fn old() {}\n+fn new() {}\n*** End Patch\n".to_string(),
                    },
                },
                rationale: "apply the requested code change".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }
        );
    }

    #[test]
    fn parse_planner_action_accepts_pretty_printed_multiline_json() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_planner_action(
                "{\n  \"action\": \"inspect\",\n  \"command\": \"git status --short\",\n  \"rationale\": \"check the local workspace state before deeper diagnosis\",\n  \"edit\": \"no\",\n  \"candidate_files\": []\n}",
            )
            .expect("planner decision");

        assert_eq!(
            decision,
            RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status --short".to_string(),
                    },
                },
                rationale: "check the local workspace state before deeper diagnosis".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }
        );
    }

    #[test]
    fn parse_planner_action_separates_direct_answer_from_rationale() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_planner_action(
                r#"{"action":"answer","answer":"Starter circuit\n\n[battery]---(solenoid)---(starter)","rationale":"the user asked for a direct ASCII diagram","edit":"no","candidate_files":[]}"#,
            )
            .expect("planner decision");

        assert_eq!(
            decision,
            RecursivePlannerDecision {
                action: PlannerAction::Stop {
                    reason: "model selected answer".to_string(),
                },
                rationale: "the user asked for a direct ASCII diagram".to_string(),
                answer: Some("Starter circuit\n\n[battery]---(solenoid)---(starter)".to_string()),
                edit: InitialEditInstruction::default(),
                grounding: None,
            }
        );
    }

    #[test]
    fn parse_initial_action_decision_preserves_grounding_contract() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_initial_action_decision(
                r#"{"action":"answer","answer":"I need a verified link before I answer.","rationale":"external source grounding is required","edit":"no","candidate_files":[],"grounding":{"domain":"external","reason":"need a verified docs link"}}"#,
            )
            .expect("initial action decision");

        assert_eq!(
            decision.grounding,
            Some(GroundingRequirement {
                domain: GroundingDomain::External,
                reason: Some("need a verified docs link".to_string()),
            })
        );
    }

    #[test]
    fn parse_initial_action_rejects_missing_edit_metadata() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let err = adapter
            .parse_initial_action_decision(
                r#"{"action":"search","query":".runtime-shell-host","mode":"linear","strategy":"bm25","rationale":"locate the selector"}"#,
            )
            .expect_err("missing edit metadata should be invalid");

        assert!(
            err.to_string()
                .contains("initial action reply must include top-level `edit`")
        );
    }

    #[test]
    fn parse_initial_action_preserves_structural_retriever_overrides() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "openai",
            "gpt-5.4",
            "test-key",
            "https://api.openai.com/v1/chat/completions",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let decision = adapter
            .parse_initial_action_decision(
                r#"{"action":"search","query":"runtime shell host","mode":"linear","strategy":"bm25","retrievers":["path-fuzzy","segment-fuzzy"],"edit":"yes","candidate_files":["apps/web/src/runtime-app.tsx"],"rationale":"use structural fuzzy lookup for the likely UI target"}"#,
            )
            .expect("initial action");

        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Search {
                    query: "runtime shell host".to_string(),
                    mode: crate::domain::ports::RetrievalMode::Linear,
                    strategy: crate::domain::ports::RetrievalStrategy::Lexical,
                    retrievers: vec![RetrieverOption::PathFuzzy, RetrieverOption::SegmentFuzzy,],
                    intent: None,
                },
            }
        );
        assert!(decision.edit.known_edit);
        assert_eq!(
            decision.edit.candidate_files,
            vec!["apps/web/src/runtime-app.tsx".to_string()]
        );
    }

    #[test]
    fn planner_system_prompt_demands_complete_json_action_envelopes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            "https://api.inceptionlabs.ai",
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let prompt = adapter
            .build_planner_system_prompt(&crate::domain::ports::InterpretationContext::default());

        assert!(prompt.contains("Return exactly one complete JSON object"));
        assert!(prompt.contains("The first key must be `action`"));
        assert!(prompt.contains("Do not wrap the JSON in markdown fences"));
        assert!(prompt.contains("exact-diff state space"));
        assert!(prompt.contains("replace_in_file"));
        assert!(prompt.contains("apply_patch"));
        assert!(prompt.contains("Do not emit partial, truncated, or streaming JSON"));
        assert!(prompt.contains("Paddles executes your selected action locally"));
        assert!(prompt.contains("working hypotheses until local evidence confirms them"));
        assert!(prompt.contains("safe, reasonable repository change"));
        assert!(prompt.contains("make the workspace edit in this turn"));
        assert!(prompt.contains("select_planner_action"));
        assert!(prompt.contains("Do not ask the user for logs"));
    }

    #[test]
    fn http_planner_action_prompt_includes_evidence_and_pressure_notes() {
        let request = PlannerRequest::new(
            "CI is failing. Can you debug it on this machine?",
            "/workspace",
            InterpretationContext::default(),
            PlannerBudget::default(),
        )
        .with_runtime_notes(vec![
            "Workspace retrieval readiness: bm25=available, vector=warming. Prefer bm25 if search is needed immediately.".to_string(),
        ])
        .with_loop_state(PlannerLoopState {
            steps: vec![crate::domain::ports::PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "gh run list --limit 10".to_string(),
                    },
                },
                outcome: "inspected recent CI runs".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "command: gh run list --limit 10 --json status,conclusion,name,headBranch,workflowName,url"
                    .to_string(),
                snippet: r#"[{"conclusion":"success","headBranch":"main","name":"CI","status":"completed","url":"https://github.com/spoke-sh/paddles/actions/runs/23910509164","workflowName":"CI"}]"#.to_string(),
                rationale: "recent CI status listing".to_string(),
                rank: 1,
            }],
            notes: vec![
                "Steering review [premise-challenge]\nTreat the reported failure as a hypothesis and judge the gathered sources before repeating the same probe."
                    .to_string(),
            ],
            ..PlannerLoopState::default()
        });

        let prompt = super::build_http_planner_action_prompt(&request, ApiFormat::OpenAi);

        assert!(prompt.contains("Current loop state"));
        assert!(prompt.contains("Runtime notes"));
        assert!(prompt.contains("Workspace retrieval readiness: bm25=available, vector=warming"));
        assert!(prompt.contains("Current evidence"));
        assert!(prompt.contains("Current notes"));
        assert!(prompt.contains("Steering review [premise-challenge]"));
        assert!(prompt.contains("gh run list --limit 10"));
        assert!(prompt.contains("\"conclusion\":\"success\""));
        assert!(prompt.contains("exact-diff state space"));
        assert!(prompt.contains("replace_in_file"));
        assert!(prompt.contains("apply_patch"));
        assert!(prompt.contains("safe, reasonable repository change"));
        assert!(prompt.contains("make the workspace edit in this turn"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_planner_retries_invalid_initial_action_before_succeeding() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: provider_response(ApiFormat::OpenAi, "I should inspect the workspace first."),
            },
            MockResponse {
                status: StatusCode::OK,
                body: provider_response(
                    ApiFormat::OpenAi,
                    r#"{"action":"inspect","command":"git status --short","edit":"no","candidate_files":[],"rationale":"check the local workspace state before deeper diagnosis"}"#,
                ),
            },
        ])
        .await;
        let planner = HttpPlannerAdapter::new(Arc::new(HttpProviderAdapter::new(
            workspace.path(),
            "openai",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        )));
        let request = PlannerRequest::new(
            "CI is failing can you debug it?",
            workspace.path(),
            InterpretationContext::default(),
            PlannerBudget::default(),
        );

        let decision = planner
            .select_initial_action(&request, Arc::new(RecordingTurnEventSink::default()))
            .await
            .expect("initial action");

        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "git status --short".to_string(),
                },
            }
        );
        let requests = server.recorded_requests();
        assert_eq!(requests.len(), 2);
        assert!(
            requests[1].body["messages"][1]["content"]
                .as_str()
                .expect("retry prompt")
                .contains("Your last planner reply was empty or invalid.")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_planner_uses_native_action_tool_and_parses_tool_calls() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: openai_tool_call_response(
                "select_planner_action",
                r#"{"action":"inspect","command":"git status --short","edit":"no","candidate_files":[],"rationale":"check the local workspace state before deeper diagnosis"}"#,
            ),
        }])
        .await;
        let planner = HttpPlannerAdapter::new(Arc::new(HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        )));
        let request = PlannerRequest::new(
            "CI is failing can you debug it?",
            workspace.path(),
            InterpretationContext::default(),
            PlannerBudget::default(),
        );

        let decision = planner
            .select_initial_action(&request, Arc::new(RecordingTurnEventSink::default()))
            .await
            .expect("initial action");

        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "git status --short".to_string(),
                },
            }
        );

        let requests = server.recorded_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body["tools"][0]["type"].as_str(),
            Some("function")
        );
        assert_eq!(
            requests[0].body["tools"][0]["function"]["name"].as_str(),
            Some("select_planner_action")
        );
        assert_eq!(
            requests[0].body["tool_choice"]["type"].as_str(),
            Some("function")
        );
        assert_eq!(
            requests[0].body["tool_choice"]["function"]["name"].as_str(),
            Some("select_planner_action")
        );
        assert_eq!(
            requests[0].body["max_completion_tokens"].as_i64(),
            Some(4096)
        );
        assert!(requests[0].body.get("max_tokens").is_none());
        assert!(requests[0].body.get("response_format").is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_prompt_envelope_requests_use_max_completion_tokens() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: provider_response(ApiFormat::OpenAi, "Mocked plain response."),
        }])
        .await;
        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "openai",
            "gpt-5.4",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::PromptEnvelope,
        );

        let (response, _) = adapter
            .send_async("System prompt", "User prompt", None)
            .await
            .expect("plain response");

        assert_eq!(response, "Mocked plain response.");

        let requests = server.recorded_requests();
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0].body["max_completion_tokens"].as_i64(),
            Some(4096)
        );
        assert!(requests[0].body.get("max_tokens").is_none());
    }

    #[test]
    fn planner_action_schema_avoids_conditional_keywords_for_openai_tool_compatibility() {
        let schema = super::planner_action_json_schema();
        let property_keys = schema["properties"]
            .as_object()
            .expect("planner schema properties")
            .keys()
            .cloned()
            .collect::<std::collections::BTreeSet<_>>();
        let required_keys = schema["required"]
            .as_array()
            .expect("planner schema required")
            .iter()
            .filter_map(|value| value.as_str().map(ToString::to_string))
            .collect::<std::collections::BTreeSet<_>>();

        assert!(schema["allOf"].is_null());
        assert_eq!(required_keys, property_keys);
        assert_eq!(
            schema["properties"]["command"]["type"][0].as_str(),
            Some("string")
        );
        assert_eq!(
            schema["properties"]["command"]["type"][1].as_str(),
            Some("null")
        );
        assert_eq!(
            schema["properties"]["answer"]["type"][0].as_str(),
            Some("string")
        );
        assert_eq!(
            schema["properties"]["answer"]["type"][1].as_str(),
            Some("null")
        );
        assert_eq!(
            schema["properties"]["mode"]["enum"][2],
            serde_json::Value::Null
        );
        assert_eq!(
            schema["properties"]["grounding"]["required"][0].as_str(),
            Some("domain")
        );
        assert_eq!(
            schema["properties"]["grounding"]["required"][1].as_str(),
            Some("reason")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn openai_planner_retries_invalid_next_action_before_succeeding() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: provider_response(ApiFormat::OpenAi, "Let me think out loud about that."),
            },
            MockResponse {
                status: StatusCode::OK,
                body: provider_response(
                    ApiFormat::OpenAi,
                    r#"{"action":"inspect","command":"git status --short","rationale":"check the local workspace state before deeper diagnosis","edit":"no","candidate_files":[]}"#,
                ),
            },
        ])
        .await;
        let planner = HttpPlannerAdapter::new(Arc::new(HttpProviderAdapter::new(
            workspace.path(),
            "openai",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        )));
        let request = PlannerRequest::new(
            "CI is failing can you debug it?",
            workspace.path(),
            InterpretationContext::default(),
            PlannerBudget::default(),
        );

        let decision = planner
            .select_next_action(&request, Arc::new(RecordingTurnEventSink::default()))
            .await
            .expect("planner action");

        assert_eq!(
            decision,
            RecursivePlannerDecision {
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "git status --short".to_string(),
                    },
                },
                rationale: "check the local workspace state before deeper diagnosis".to_string(),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: None,
            }
        );
        let requests = server.recorded_requests();
        assert_eq!(requests.len(), 2);
        assert!(
            requests[1].body["messages"][1]["content"]
                .as_str()
                .expect("retry prompt")
                .contains("Your last planner reply was empty or invalid.")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn http_provider_apply_patch_actions_stay_local_even_on_inception() {
        let workspace = tempfile::tempdir().expect("workspace");
        fs::write(
            workspace.path().join("sample.rs"),
            "fn greet() {\n    println!(\"hello\");\n}\n",
        )
        .expect("write sample file");

        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: provider_response(ApiFormat::OpenAi, "unused"),
        }])
        .await;

        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );

        let result = adapter
            .execute_workspace_action(&WorkspaceAction::ApplyPatch {
                patch: concat!(
                    "--- a/sample.rs\n",
                    "+++ b/sample.rs\n",
                    "@@ -1,3 +1,3 @@\n",
                    " fn greet() {\n",
                    "-    println!(\"hello\");\n",
                    "+    println!(\"hi\");\n",
                    " }\n",
                )
                .to_string(),
            })
            .expect("apply patch");

        assert_eq!(result.name, "apply_patch");
        assert!(
            result
                .summary
                .starts_with("Applied apply_patch to sample.rs (+1 -1).")
        );
        assert!(
            result
                .applied_edit
                .as_ref()
                .is_some_and(|edit| edit.diff.contains("+    println!(\"hi\");"))
        );
        assert_eq!(
            fs::read_to_string(workspace.path().join("sample.rs")).expect("read sample file"),
            "fn greet() {\n    println!(\"hi\");\n}\n"
        );

        let requests = server.recorded_requests();
        assert!(requests.is_empty(), "apply_patch should stay local");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn anthropic_provider_executes_a_full_turn_against_a_mock_server() {
        let model_id = "claude-sonnet-4-20250514";
        let (requests, events, response) = run_mocked_turn(
            ApiFormat::Anthropic,
            crate::infrastructure::providers::ModelProvider::Anthropic,
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
            crate::infrastructure::providers::ModelProvider::Google,
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
        assert_eq!(
            requests[0].body["generationConfig"]["responseMimeType"].as_str(),
            Some("application/json")
        );
        assert_eq!(
            requests[0].body["generationConfig"]["responseSchema"]["type"].as_str(),
            Some("object")
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

        let (_, _, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            crate::infrastructure::providers::ModelProvider::Openai,
            model_id,
            &structured,
        )
        .await;

        assert_eq!(
            response,
            "The next bearing is HTTP API Design For Paddles.\n\n- status: decision-ready\n- EV: 10.31"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn inception_provider_normalizes_structured_final_answers() {
        let model_id = "mercury-2";
        let structured = json!({
            "render_types": ["paragraph", "bullet_list"],
            "blocks": [
                { "type": "paragraph", "text": "The next bearing is HTTP API Design For Paddles." },
                { "type": "bullet_list", "items": ["status: decision-ready", "EV: 10.31"] }
            ]
        })
        .to_string();

        let (_, _, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            crate::infrastructure::providers::ModelProvider::Inception,
            model_id,
            &structured,
        )
        .await;

        assert_eq!(
            response,
            "The next bearing is HTTP API Design For Paddles.\n\n- status: decision-ready\n- EV: 10.31"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn inception_provider_salvages_fragmented_structured_final_answers() {
        let model_id = "mercury-2";
        let structured = r#"{
  "blocks": [
    {
      "type": "paragraph"
    },
    {
      "text": "Hey! How can I help you today?"
"#;

        let (_, _, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            crate::infrastructure::providers::ModelProvider::Inception,
            model_id,
            structured,
        )
        .await;

        assert_eq!(response, "Hey! How can I help you today?");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn inception_provider_surfaces_debuggable_excerpt_for_unrecoverable_structured_answers() {
        let model_id = "mercury-2";
        let structured = r#"{
  "blocks": [
    [
      " "
"#;

        let (_, _, response) = run_mocked_turn(
            ApiFormat::OpenAi,
            crate::infrastructure::providers::ModelProvider::Inception,
            model_id,
            structured,
        )
        .await;

        assert!(response.contains("invalid structured answer"));
        assert!(response.contains("```json"));
        assert!(response.contains("complete payload"));
        assert!(response.contains("```text"));
        assert!(response.contains("\"blocks\""));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn grounded_http_responses_fall_back_when_they_promise_future_command_work() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: final_answer_response(
                ApiFormat::OpenAi,
                &structured_answer_json(
                    "I will query the recent GitHub Actions runs that failed to retrieve details about the CI failure.",
                ),
            ),
        }])
        .await;

        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );
        let evidence = EvidenceBundle::new(
            "Recent local inspection commands already ran against the workspace and CI surface.",
            vec![
                EvidenceItem {
                    source: "command: gh run list --limit 5".to_string(),
                    snippet: "completed run listing".to_string(),
                    rationale: "recent CI runs".to_string(),
                    rank: 0,
                },
                EvidenceItem {
                    source: "command: gh run view 123 --log-failed".to_string(),
                    snippet: "log excerpt".to_string(),
                    rationale: "failing job details".to_string(),
                    rank: 1,
                },
            ],
        );

        let response = adapter
            .respond_for_turn(
                "CI is failing. Can you debug it on this machine?",
                TurnIntent::Planned,
                Some(&evidence),
                &SynthesisHandoff::default(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .expect("grounded response");

        assert!(!response.contains("I will query"));
        assert!(response.contains("I already ran local inspection commands in the harness."));
        assert!(response.contains("command: gh run list --limit 5"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn grounded_http_responses_fall_back_when_they_deny_visible_command_output() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: final_answer_response(
                ApiFormat::OpenAi,
                &structured_answer_json(
                    "I attempted to run the commands you requested, but the tool didn't produce any output. This usually means the execution environment is not available or the command failed silently. Could you please confirm that the repository is cloned and that I have access to the gh CLI and the source code on this machine?",
                ),
            ),
        }])
        .await;

        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );
        let evidence = EvidenceBundle::new(
            "Recent local inspection commands already ran against the workspace and CI surface.",
            vec![
                EvidenceItem {
                    source: "command: git status --short".to_string(),
                    snippet: "M  CONFIGURATION.md\nM  README.md".to_string(),
                    rationale: "workspace status".to_string(),
                    rank: 0,
                },
                EvidenceItem {
                    source: "command: gh run list --limit 20 --status failure".to_string(),
                    snippet: "completed\tfailure\tFix public Nix inputs for CI\tCI\tmain\tpush\t23864022492".to_string(),
                    rationale: "failed CI run".to_string(),
                    rank: 1,
                },
            ],
        );

        let response = adapter
            .respond_for_turn(
                "CI is failing. Can you debug it on this machine?",
                TurnIntent::Planned,
                Some(&evidence),
                &SynthesisHandoff::default(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .expect("grounded response");

        assert!(!response.contains("didn't produce any output"));
        assert!(!response.contains("confirm that the repository is cloned"));
        assert!(response.contains("I already ran local inspection commands in the harness."));
        assert!(response.contains("command: gh run list --limit 20 --status failure"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn grounded_http_responses_fall_back_when_they_assert_unverified_failure_claims() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: final_answer_response(
                ApiFormat::OpenAi,
                &structured_answer_json(
                    "The CI pipeline is currently failing after recent changes to the HTTP provider and rendering modules.",
                ),
            ),
        }])
        .await;

        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );
        let evidence = EvidenceBundle::new(
            "Recent harness inspection has not yet confirmed the reported CI failure.",
            vec![
                EvidenceItem {
                    source: "command: gh run list --limit 10".to_string(),
                    snippet: "completed\tsuccess\tPersist runtime lane preferences over config\tCI\tmain\tpush\t23910509164".to_string(),
                    rationale: "recent successful CI run".to_string(),
                    rank: 0,
                },
                EvidenceItem {
                    source: "command: gh run list --limit 10 --json id,status,conclusion,headBranch,workflow".to_string(),
                    snippet: "Unknown JSON field: \"id\"\nAvailable fields:\n  databaseId\n  displayTitle".to_string(),
                    rationale: "invalid gh query, not CI failure evidence".to_string(),
                    rank: 1,
                },
            ],
        );

        let response = adapter
            .respond_for_turn(
                "CI is failing. Can you debug it on this machine?",
                TurnIntent::Planned,
                Some(&evidence),
                &SynthesisHandoff::default(),
                Arc::new(RecordingTurnEventSink::default()),
            )
            .expect("grounded response");

        assert!(!response.contains("CI pipeline is currently failing"));
        assert!(
            response
                .contains("The reported failure is not yet confirmed by the gathered evidence.")
        );
        assert!(response.contains("command: gh run list --limit 10"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn external_grounding_rejects_unverified_urls_in_http_responses() {
        let workspace = tempfile::tempdir().expect("workspace");
        let server = start_mock_server(vec![MockResponse {
            status: StatusCode::OK,
            body: final_answer_response(
                ApiFormat::OpenAi,
                &structured_answer_json(
                    "You can read about it here: https://inception.ai/diffusion-llm",
                ),
            ),
        }])
        .await;

        let adapter = super::HttpProviderAdapter::new(
            workspace.path(),
            "inception",
            "mercury-2",
            "test-key",
            server.base_url.clone(),
            ApiFormat::OpenAi,
            RenderCapability::OpenAiJsonSchema,
        );
        let sink = Arc::new(RecordingTurnEventSink::default());

        let response = adapter
            .respond_for_turn(
                "Can you give me the docs link?",
                TurnIntent::DirectResponse,
                None,
                &SynthesisHandoff {
                    grounding: Some(GroundingRequirement {
                        domain: GroundingDomain::External,
                        reason: Some("need a verified web source before answering".to_string()),
                    }),
                    ..SynthesisHandoff::default()
                },
                sink.clone(),
            )
            .expect("response");

        assert!(!response.contains("https://inception.ai/diffusion-llm"));
        assert!(response.contains("can't provide a verified external link"));
        assert!(sink.recorded().iter().any(|event| matches!(
            event,
            TurnEvent::Fallback { stage, reason }
                if stage == "grounding-governor"
                    && reason.contains("verified external sources")
        )));
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
            crate::infrastructure::providers::ModelProvider::Openai,
            ApiFormat::OpenAi,
        );
        let runtime_lanes = RuntimeLaneConfig::new("kimi-k2.5".to_string(), None)
            .with_synthesizer_provider(crate::infrastructure::providers::ModelProvider::Openai);
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
