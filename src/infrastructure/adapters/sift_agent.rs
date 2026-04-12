use super::agent_memory::{AgentMemory, load_guidance_document};
use super::local_workspace_editor::LocalWorkspaceEditor;
use crate::domain::model::{
    CompactionDecision, ConversationThreadRef, ForensicArtifactCapture, NullTurnEventSink,
    StrainFactor, StrainTracker, ThreadDecision, ThreadDecisionId, ThreadDecisionKind,
    ThreadMergeMode, TraceArtifactId, TraceModelExchangeCategory, TraceModelExchangeLane,
    TraceModelExchangePhase, TurnEvent, TurnEventSink, TurnIntent,
};
use crate::domain::ports::{
    CompactionPlan, CompactionRequest, EvidenceBundle, GroundingDomain, GroundingRequirement,
    GuidanceCategory, InitialAction, InitialActionDecision, InitialEditInstruction,
    InterpretationConflict, InterpretationContext, InterpretationCoverageConfidence,
    InterpretationDecisionFramework, InterpretationDocument, InterpretationProcedure,
    InterpretationProcedureStep, InterpretationRequest, InterpretationToolHint, ModelPaths,
    OperatorMemoryDocument, PlannerAction, PlannerLoopState, PlannerRequest,
    RecursivePlannerDecision, RetrievalMode, RetrievalStrategy, RetrieverOption, SynthesisHandoff,
    ThreadDecisionRequest, WorkspaceAction, WorkspaceEditor,
};
use crate::infrastructure::execution_governance::{
    GovernedTerminalCommandResult, summarize_governance_outcome,
};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::rendering::{
    RenderCapability, ensure_citation_section, extract_http_urls, final_answer_contract_prompt,
    normalize_assistant_response,
};
use crate::infrastructure::sift_cache::{
    default_sift_cache_dir_for_workspace, ensure_sift_process_cache_dirs,
};
use crate::infrastructure::terminal::run_background_terminal_command_with_runtime_mediator;
use crate::infrastructure::transport_mediator::TransportToolMediator;
use crate::infrastructure::workspace_paths::WorkspacePathPolicy;
use anyhow::{Context, Result, anyhow, bail};
use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::generation::{LogitsProcessor, Sampling};
use candle_transformers::models::{
    qwen2::{Config as Qwen2Config, ModelForCausalLM as Qwen2Model},
    qwen3::{Config as Qwen3Config, ModelForCausalLM as Qwen3Model},
    qwen3_5::{Config as Qwen3_5Config, ModelForCausalLM as Qwen3_5Model},
};
use serde::Deserialize;
use serde_json::json;
use sift::internal::search::adapters::llm_utils::{QwenConfigPartial, get_device_for};
use sift::{
    AgentTurnInput, ContextAssemblyBudget, ContextAssemblyRequest, ContextAssemblyResponse,
    Conversation, EnvironmentFactInput, LocalContextSource, RetainedArtifact, SearchPlan, Sift,
    ToolOutputInput,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};
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
const MAX_CITATIONS: usize = 4;
const MAX_INTERPRETATION_GRAPH_DEPTH: usize = 3;
const MAX_INTERPRETATION_GRAPH_DOCS: usize = 8;
const MAX_GRAPH_DOC_CHARS: usize = 6_000;
const DEFAULT_QWEN_MAX_LENGTH: usize = 512;
const QWEN_SYSTEM_PROMPT: &str = "<|im_start|>system\nYou are Paddles, a helpful AI assistant and mech suit operator. You provide concise and accurate technical advice.<|im_end|>\n";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QwenModelFamily {
    Qwen2,
    Qwen3,
    Qwen3_5,
}

#[derive(Clone, Debug)]
struct PreparedQwenModel {
    model_id: String,
    paths: ModelPaths,
    family: QwenModelFamily,
    max_length: usize,
}

pub struct SiftAgentAdapter {
    workspace_root: PathBuf,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    transport_mediator: Arc<TransportToolMediator>,
    model_id: String,
    sift: Sift,
    conversation_factory: Box<dyn ConversationFactory>,
    base_context: Vec<LocalContextSource>,
    render_capability: RenderCapability,
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
pub(crate) struct ToolResult {
    pub(crate) name: &'static str,
    pub(crate) summary: String,
    pub(crate) applied_edit: Option<crate::domain::model::AppliedEdit>,
    pub(crate) governance_outcome: Option<crate::domain::model::ExecutionGovernanceOutcome>,
    pub(crate) retained_artifacts: Option<Vec<RetainedArtifact>>,
}

struct SiftForensicRecord {
    exchange_id: String,
    phase: TraceModelExchangePhase,
    summary: String,
    content: String,
    mime_type: String,
    parent_artifact_id: Option<TraceArtifactId>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
enum PlannerActionEnvelope {
    Search {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        retrievers: Vec<RetrieverOption>,
        #[serde(default)]
        intent: Option<String>,
        rationale: String,
    },
    ListFiles {
        #[serde(default)]
        pattern: Option<String>,
        rationale: String,
    },
    Read {
        path: String,
        rationale: String,
    },
    Inspect {
        command: String,
        rationale: String,
    },
    Shell {
        command: String,
        rationale: String,
    },
    Diff {
        #[serde(default)]
        path: Option<String>,
        rationale: String,
    },
    WriteFile {
        path: String,
        content: String,
        rationale: String,
    },
    ReplaceInFile {
        path: String,
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
        rationale: String,
    },
    ApplyPatch {
        patch: String,
        rationale: String,
    },
    Refine {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        retrievers: Vec<RetrieverOption>,
        #[serde(default)]
        rationale: Option<String>,
    },
    Branch {
        branches: Vec<String>,
        #[serde(default)]
        rationale: Option<String>,
    },
    Stop {
        reason: String,
        #[serde(default)]
        rationale: Option<String>,
        #[serde(default)]
        answer: Option<String>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct RecursiveActionEnvelope {
    #[serde(flatten)]
    action: PlannerActionEnvelope,
    #[serde(default)]
    edit: Option<String>,
    #[serde(default)]
    candidate_files: Option<Vec<String>>,
    #[serde(default)]
    grounding: Option<GroundingRequirement>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InitialActionEnvelope {
    #[serde(flatten)]
    action: InitialActionVariantEnvelope,
    #[serde(default)]
    edit: Option<String>,
    #[serde(default)]
    candidate_files: Option<Vec<String>>,
    #[serde(default)]
    grounding: Option<GroundingRequirement>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", rename_all = "snake_case")]
enum InitialActionVariantEnvelope {
    Answer {
        answer: String,
        rationale: String,
    },
    Search {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        retrievers: Vec<RetrieverOption>,
        #[serde(default)]
        intent: Option<String>,
        rationale: String,
    },
    ListFiles {
        #[serde(default)]
        pattern: Option<String>,
        rationale: String,
    },
    Read {
        path: String,
        rationale: String,
    },
    Inspect {
        command: String,
        rationale: String,
    },
    Shell {
        command: String,
        rationale: String,
    },
    Diff {
        #[serde(default)]
        path: Option<String>,
        rationale: String,
    },
    WriteFile {
        path: String,
        content: String,
        rationale: String,
    },
    ReplaceInFile {
        path: String,
        old: String,
        new: String,
        #[serde(default)]
        replace_all: bool,
        rationale: String,
    },
    ApplyPatch {
        patch: String,
        rationale: String,
    },
    Refine {
        query: String,
        mode: RetrievalMode,
        strategy: RetrievalStrategy,
        #[serde(default)]
        retrievers: Vec<RetrieverOption>,
        #[serde(default)]
        rationale: Option<String>,
    },
    Branch {
        branches: Vec<String>,
        #[serde(default)]
        rationale: Option<String>,
    },
    Stop {
        reason: String,
        #[serde(default)]
        rationale: Option<String>,
        #[serde(default)]
        answer: Option<String>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(tag = "decision", rename_all = "snake_case")]
enum ThreadDecisionEnvelope {
    ContinueCurrentThread {
        rationale: String,
    },
    OpenChildThread {
        label: String,
        rationale: String,
    },
    MergeIntoTarget {
        target_thread_id: String,
        merge_mode: String,
        #[serde(default)]
        summary: Option<String>,
        rationale: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationValidationEnvelope {
    #[serde(default)]
    gaps: Vec<InterpretationGapEnvelope>,
    #[serde(default)]
    needs_more_guidance: bool,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationGapEnvelope {
    area: String,
    rationale: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationGraphEnvelope {
    #[serde(default)]
    edges: Vec<InterpretationGraphEdge>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationContextEnvelope {
    #[serde(default)]
    summary: String,
    #[serde(default)]
    documents: Vec<InterpretationDocumentEnvelope>,
    #[serde(default)]
    tool_hints: Vec<InterpretationToolHintEnvelope>,
    #[serde(default)]
    procedures: Vec<InterpretationProcedureEnvelope>,
    #[serde(default)]
    precedence_chain: Vec<String>,
    #[serde(default)]
    conflicts: Vec<InterpretationConflictEnvelope>,
    #[serde(default)]
    coverage_confidence: InterpretationCoverageConfidence,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationDocumentEnvelope {
    source: String,
    excerpt: String,
    category: GuidanceCategory,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationConflictEnvelope {
    source_a: String,
    source_b: String,
    description: String,
    resolution: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationToolHintEnvelope {
    source: String,
    action: WorkspaceAction,
    note: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationProcedureEnvelope {
    source: String,
    label: String,
    purpose: String,
    #[serde(default)]
    steps: Vec<InterpretationProcedureStepEnvelope>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationProcedureStepEnvelope {
    index: usize,
    action: WorkspaceAction,
    note: String,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct InterpretationGraphEdge {
    source: String,
    #[serde(default)]
    targets: Vec<String>,
}

struct TurnPrompt<'a> {
    workspace_root: &'a Path,
    user_prompt: &'a str,
    recent_turns: &'a str,
    recent_thread_summary: Option<&'a str>,
    memory_prompt: &'a str,
    context: &'a ContextAssemblyResponse,
    gathered_evidence: Option<&'a EvidenceBundle>,
    prefer_tools: bool,
    follow_up_execution: bool,
    render_capability: RenderCapability,
}

struct PlannerPrompt<'a> {
    workspace_root: &'a Path,
    user_prompt: &'a str,
    interpretation: &'a InterpretationContext,
    request: &'a PlannerRequest,
}

struct ThreadPlannerPrompt<'a> {
    workspace_root: &'a Path,
    interpretation: &'a InterpretationContext,
    request: &'a ThreadDecisionRequest,
}

impl PreparedQwenModel {
    fn from_paths(model_id: &str, paths: ModelPaths) -> Result<Self> {
        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&paths.config)?)
                .with_context(|| format!("failed to parse {}", paths.config.display()))?;
        Ok(Self {
            model_id: model_id.to_string(),
            family: infer_qwen_family(&config)?,
            max_length: infer_qwen_runtime_max_length(&config),
            paths,
        })
    }
}

fn infer_qwen_family(config: &serde_json::Value) -> Result<QwenModelFamily> {
    let model_type = config
        .get("model_type")
        .and_then(|value| value.as_str())
        .or_else(|| {
            config
                .get("text_config")
                .and_then(|value| value.get("model_type"))
                .and_then(|value| value.as_str())
        });

    if let Some(model_type) = model_type {
        match model_type {
            "qwen2" => return Ok(QwenModelFamily::Qwen2),
            "qwen3" => return Ok(QwenModelFamily::Qwen3),
            "qwen3_5" | "qwen3_5_text" => return Ok(QwenModelFamily::Qwen3_5),
            _ => {}
        }
    }

    if let Some(architectures) = config
        .get("architectures")
        .and_then(|value| value.as_array())
    {
        if architectures
            .iter()
            .any(|value| value.as_str().is_some_and(|name| name.contains("Qwen3_5")))
        {
            return Ok(QwenModelFamily::Qwen3_5);
        }
        if architectures
            .iter()
            .any(|value| value.as_str().is_some_and(|name| name.contains("Qwen3")))
        {
            return Ok(QwenModelFamily::Qwen3);
        }
        if architectures
            .iter()
            .any(|value| value.as_str().is_some_and(|name| name.contains("Qwen2")))
        {
            return Ok(QwenModelFamily::Qwen2);
        }
    }

    bail!("unsupported local sift model config: expected a qwen2/qwen3/qwen3_5 bundle")
}

fn infer_qwen_runtime_max_length(config: &serde_json::Value) -> usize {
    config
        .get("max_position_embeddings")
        .and_then(|value| value.as_u64())
        .or_else(|| {
            config
                .get("text_config")
                .and_then(|value| value.get("max_position_embeddings"))
                .and_then(|value| value.as_u64())
        })
        .map(|value| usize::try_from(value).unwrap_or(DEFAULT_QWEN_MAX_LENGTH))
        .map(|value| value.min(DEFAULT_QWEN_MAX_LENGTH))
        .unwrap_or(DEFAULT_QWEN_MAX_LENGTH)
}

trait ConversationFactory: Send + Sync {
    fn start_conversation(&self) -> Result<Box<dyn Conversation>>;
}

struct ReusableQwenConversationFactory {
    runtime: Arc<Mutex<PaddlesQwenRuntime>>,
}

impl ReusableQwenConversationFactory {
    fn load(model: PreparedQwenModel) -> Result<Self> {
        Ok(Self {
            runtime: Arc::new(Mutex::new(PaddlesQwenRuntime::load(model)?)),
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
    model: PreparedQwenModel,
    session: PaddlesQwenSession,
    tokenizer: Tokenizer,
    family: QwenModelFamily,
}

impl PaddlesQwenRuntime {
    fn load(model: PreparedQwenModel) -> Result<Self> {
        let tokenizer = Tokenizer::from_file(&model.paths.tokenizer)
            .map_err(|err| anyhow!("failed to load tokenizer: {err}"))?;
        let device = get_device_for("QWEN")?;
        let session = match Self::load_session(&model, &device) {
            Ok(session) => session,
            Err(err) if should_retry_qwen_on_cpu(&device, &err) => {
                tracing::warn!(
                    "CUDA runtime for {} failed during load ({}); retrying on CPU",
                    model.model_id,
                    err
                );
                Self::load_session(&model, &Device::Cpu)?
            }
            Err(err) => return Err(err),
        };

        Ok(Self {
            family: model.family,
            model,
            session,
            tokenizer,
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.session.reset();
        Ok(())
    }

    fn send(&mut self, message: &str, max_tokens: usize) -> Result<String> {
        self.reset()?;
        let prompted = format_qwen_prompt(self.family, message);
        match self
            .session
            .generate(&prompted, max_tokens, &self.tokenizer)
        {
            Ok(response) => Ok(response),
            Err(err) if should_retry_qwen_on_cpu(&self.session.device, &err) => {
                tracing::warn!(
                    "CUDA runtime for {} failed during generation ({}); retrying on CPU",
                    self.model.model_id,
                    err
                );
                self.reload_on_cpu()?;
                self.reset()?;
                self.session
                    .generate(&prompted, max_tokens, &self.tokenizer)
            }
            Err(err) => Err(err),
        }
    }

    fn reload_on_cpu(&mut self) -> Result<()> {
        self.session = Self::load_session(&self.model, &Device::Cpu)?;
        Ok(())
    }

    fn load_session(model: &PreparedQwenModel, device: &Device) -> Result<PaddlesQwenSession> {
        let config_path = &model.paths.config;
        let generation =
            load_generation_settings(model.paths.generation_config.as_deref(), config_path)?;
        let dtype = preferred_qwen_weight_dtype(model.family, device);
        let vb = load_qwen_var_builder(&model.paths.weights, dtype, device)?;

        match model.family {
            QwenModelFamily::Qwen2 => {
                let config_partial: QwenConfigPartial =
                    serde_json::from_str(&fs::read_to_string(config_path)?)?;
                let config = config_partial.into_config()?;
                PaddlesQwenSession::new_qwen2(&config, &vb, device, model.max_length, generation)
            }
            QwenModelFamily::Qwen3 => {
                let config: Qwen3Config = serde_json::from_str(&fs::read_to_string(config_path)?)?;
                PaddlesQwenSession::new_qwen3(&config, &vb, device, model.max_length, generation)
            }
            QwenModelFamily::Qwen3_5 => {
                let config: Qwen3_5Config =
                    serde_json::from_str(&fs::read_to_string(config_path)?)?;
                PaddlesQwenSession::new_qwen3_5(&config, &vb, device, model.max_length, generation)
            }
        }
    }
}

enum QwenModelRunner {
    Qwen2 { model: Qwen2Model },
    Qwen3 { model: Qwen3Model },
    Qwen3_5 { model: Qwen3_5Model },
}

struct PaddlesQwenSession {
    runner: QwenModelRunner,
    tokens: Vec<u32>,
    device: Device,
    max_length: usize,
    generation: QwenGenerationSettings,
}

#[derive(Clone, Debug, PartialEq)]
struct QwenGenerationSettings {
    eos_token_ids: Vec<u32>,
    repetition_penalty: f32,
    repeat_last_n: usize,
    sampling: Sampling,
    seed: u64,
}

#[derive(Debug, Deserialize)]
struct QwenGenerationConfig {
    #[serde(default)]
    do_sample: bool,
    eos_token_id: serde_json::Value,
    #[serde(default)]
    repetition_penalty: Option<f32>,
    #[serde(default)]
    temperature: Option<f64>,
    #[serde(default)]
    top_p: Option<f64>,
    #[serde(default)]
    top_k: Option<usize>,
}

impl PaddlesQwenSession {
    fn new_qwen2(
        config: &Qwen2Config,
        vb: &VarBuilder<'static>,
        device: &Device,
        max_length: usize,
        generation: QwenGenerationSettings,
    ) -> Result<Self> {
        Ok(Self {
            runner: QwenModelRunner::Qwen2 {
                model: Qwen2Model::new(config, vb.clone())?,
            },
            tokens: Vec::new(),
            device: device.clone(),
            max_length,
            generation,
        })
    }

    fn new_qwen3(
        config: &Qwen3Config,
        vb: &VarBuilder<'static>,
        device: &Device,
        max_length: usize,
        generation: QwenGenerationSettings,
    ) -> Result<Self> {
        Ok(Self {
            runner: QwenModelRunner::Qwen3 {
                model: Qwen3Model::new(config, vb.clone())?,
            },
            tokens: Vec::new(),
            device: device.clone(),
            max_length,
            generation,
        })
    }

    fn new_qwen3_5(
        config: &Qwen3_5Config,
        vb: &VarBuilder<'static>,
        device: &Device,
        max_length: usize,
        generation: QwenGenerationSettings,
    ) -> Result<Self> {
        Ok(Self {
            runner: QwenModelRunner::Qwen3_5 {
                model: Qwen3_5Model::new(config, vb.clone())?,
            },
            tokens: Vec::new(),
            device: device.clone(),
            max_length,
            generation,
        })
    }

    fn reset(&mut self) {
        self.tokens.clear();
        match &mut self.runner {
            QwenModelRunner::Qwen2 { model, .. } => model.clear_kv_cache(),
            QwenModelRunner::Qwen3 { model } => model.clear_kv_cache(),
            QwenModelRunner::Qwen3_5 { model } => model.clear_kv_cache(),
        }
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
        let mut logits_processor = self.generation.logits_processor();
        let first_token_ban_ids = self.first_token_ban_ids(tokenizer);
        let mut pending_next_token = None;

        if !new_tokens.is_empty() {
            let pos = self.tokens.len();
            let logits = self.next_token_logits(new_tokens, pos)?;
            self.tokens.extend_from_slice(new_tokens);

            if max_tokens > 0 {
                pending_next_token = Some(self.select_next_token(
                    &logits,
                    &mut logits_processor,
                    &first_token_ban_ids,
                )?);
            }
        } else if max_tokens > 0 && self.tokens.is_empty() {
            return Ok(String::new());
        }

        for _ in 0..max_tokens {
            let next_token = match pending_next_token.take() {
                Some(token) => token,
                None => {
                    let last_token = *self.tokens.last().unwrap();
                    let logits = self.next_token_logits(&[last_token], self.tokens.len() - 1)?;
                    let banned_ids = if generated_tokens.is_empty() {
                        first_token_ban_ids.as_slice()
                    } else {
                        &[][..]
                    };
                    self.select_next_token(&logits, &mut logits_processor, banned_ids)?
                }
            };

            if self.generation.is_eos_token(next_token) || self.tokens.len() >= self.max_length {
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

    fn select_next_token(
        &self,
        logits: &Tensor,
        logits_processor: &mut LogitsProcessor,
        banned_ids: &[u32],
    ) -> Result<u32> {
        let logits = if self.generation.repetition_penalty <= 1.0 {
            logits.to_dtype(DType::F32)?
        } else {
            let start_at = self
                .tokens
                .len()
                .saturating_sub(self.generation.repeat_last_n);
            candle_transformers::utils::apply_repeat_penalty(
                logits,
                self.generation.repetition_penalty,
                &self.tokens[start_at..],
            )?
        };

        let sampled = logits_processor.sample(&logits);
        let sampled = match sampled {
            Ok(token_id) if !banned_ids.contains(&token_id) => return Ok(token_id),
            Ok(_) | Err(_) => self.select_argmax_token(&logits, banned_ids)?,
        };

        Ok(sampled)
    }

    fn select_argmax_token(&self, logits: &Tensor, banned_ids: &[u32]) -> Result<u32> {
        let mut logits = logits.to_dtype(DType::F32)?.to_vec1::<f32>()?;

        for token_id in banned_ids {
            if let Some(logit) = logits.get_mut(*token_id as usize) {
                *logit = f32::NEG_INFINITY;
            }
        }

        logits
            .iter()
            .enumerate()
            .filter(|(_, value)| value.is_finite())
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(index, _)| index as u32)
            .ok_or_else(|| anyhow!("no valid token remained after filtering logits"))
    }

    fn first_token_ban_ids(&self, tokenizer: &Tokenizer) -> Vec<u32> {
        let mut banned_ids = self.generation.eos_token_ids.clone();
        if let Some(token_id) = tokenizer.token_to_id("<|im_start|>") {
            banned_ids.push(token_id);
        }
        banned_ids.sort_unstable();
        banned_ids.dedup();
        banned_ids
    }

    fn next_token_logits(&mut self, tokens: &[u32], offset: usize) -> Result<Tensor> {
        let tokens_tensor = Tensor::new(tokens, &self.device)?.unsqueeze(0)?;
        match &mut self.runner {
            QwenModelRunner::Qwen2 { model } => {
                let logits = model.forward(&tokens_tensor, offset)?;
                logits.get(0)?.get(0).map_err(Into::into)
            }
            QwenModelRunner::Qwen3 { model } => {
                let logits = model.forward(&tokens_tensor, offset)?;
                logits.get(0)?.get(0).map_err(Into::into)
            }
            QwenModelRunner::Qwen3_5 { model } => {
                let logits = model.forward(&tokens_tensor, offset)?;
                logits.get(0)?.get(0).map_err(Into::into)
            }
        }
    }
}

fn preferred_qwen_weight_dtype(family: QwenModelFamily, device: &Device) -> DType {
    if !device.is_cuda() {
        return DType::F32;
    }

    match family {
        QwenModelFamily::Qwen2 | QwenModelFamily::Qwen3 | QwenModelFamily::Qwen3_5 => DType::BF16,
    }
}

impl QwenGenerationSettings {
    fn logits_processor(&self) -> LogitsProcessor {
        LogitsProcessor::from_sampling(self.seed, self.sampling.clone())
    }

    fn is_eos_token(&self, token_id: u32) -> bool {
        self.eos_token_ids.contains(&token_id)
    }
}

fn load_generation_settings(
    generation_config_path: Option<&Path>,
    config_path: &Path,
) -> Result<QwenGenerationSettings> {
    let generation_config = if let Some(path) = generation_config_path {
        serde_json::from_str::<QwenGenerationConfig>(&fs::read_to_string(path)?)
            .with_context(|| format!("failed to parse {}", path.display()))?
    } else {
        let config: serde_json::Value = serde_json::from_str(&fs::read_to_string(config_path)?)
            .with_context(|| format!("failed to parse {}", config_path.display()))?;
        let eos_token_id = config
            .get("eos_token_id")
            .cloned()
            .or_else(|| {
                config
                    .get("text_config")
                    .and_then(|value| value.get("eos_token_id"))
                    .cloned()
            })
            .ok_or_else(|| anyhow!("config does not define eos_token_id"))?;
        QwenGenerationConfig {
            do_sample: false,
            eos_token_id,
            repetition_penalty: None,
            temperature: None,
            top_p: None,
            top_k: None,
        }
    };

    Ok(QwenGenerationSettings {
        eos_token_ids: parse_eos_token_ids(&generation_config.eos_token_id)?,
        repetition_penalty: generation_config.repetition_penalty.unwrap_or(1.0),
        repeat_last_n: 64,
        sampling: generation_sampling(&generation_config),
        seed: 299_792_458,
    })
}

fn parse_eos_token_ids(value: &serde_json::Value) -> Result<Vec<u32>> {
    let eos_ids = match value {
        serde_json::Value::Number(number) => vec![
            number
                .as_u64()
                .ok_or_else(|| anyhow!("invalid eos_token_id: {number}"))? as u32,
        ],
        serde_json::Value::Array(values) => values
            .iter()
            .map(|value| {
                value
                    .as_u64()
                    .map(|value| value as u32)
                    .ok_or_else(|| anyhow!("invalid eos_token_id entry: {value}"))
            })
            .collect::<Result<Vec<_>>>()?,
        other => bail!("unsupported eos_token_id format: {other}"),
    };

    if eos_ids.is_empty() {
        bail!("generation config does not define any eos token ids");
    }

    Ok(eos_ids)
}

fn generation_sampling(config: &QwenGenerationConfig) -> Sampling {
    if !config.do_sample {
        return Sampling::ArgMax;
    }

    let temperature = config.temperature.unwrap_or(0.7);
    let top_p = config.top_p.unwrap_or(0.8);

    match config.top_k {
        Some(top_k) if top_k > 0 => Sampling::TopKThenTopP {
            k: top_k,
            p: top_p,
            temperature,
        },
        _ => Sampling::TopP {
            p: top_p,
            temperature,
        },
    }
}

fn should_retry_qwen_on_cpu(device: &Device, err: &anyhow::Error) -> bool {
    should_retry_qwen_on_cpu_message(device.is_cuda(), &err.to_string())
}

fn should_retry_qwen_on_cpu_message(device_is_cuda: bool, error_message: &str) -> bool {
    if !device_is_cuda {
        return false;
    }

    let error_message = error_message.to_ascii_lowercase();
    error_message.contains("out of memory") || error_message.contains("unexpected dtype")
}

fn format_qwen_prompt(family: QwenModelFamily, message: &str) -> String {
    let assistant_prefix = match family {
        QwenModelFamily::Qwen2 => "<|im_start|>assistant\n",
        QwenModelFamily::Qwen3 | QwenModelFamily::Qwen3_5 => {
            "<|im_start|>assistant\n<think>\n\n</think>\n\n"
        }
    };

    format!("{QWEN_SYSTEM_PROMPT}<|im_start|>user\n{message}<|im_end|>\n{assistant_prefix}")
}

fn load_qwen_var_builder(
    weights_paths: &[PathBuf],
    dtype: DType,
    device: &Device,
) -> Result<VarBuilder<'static>> {
    unsafe { VarBuilder::from_mmaped_safetensors(weights_paths, dtype, device) }.map_err(Into::into)
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
    pub fn new(
        workspace_root: impl Into<PathBuf>,
        model_id: &str,
        model_paths: ModelPaths,
        render_capability: RenderCapability,
    ) -> Result<Self> {
        Self::new_with_execution_hand_registry(
            workspace_root,
            Arc::new(ExecutionHandRegistry::with_default_local_governance()),
            model_id,
            model_paths,
            render_capability,
        )
    }

    pub fn new_with_execution_hand_registry(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        model_id: &str,
        model_paths: ModelPaths,
        render_capability: RenderCapability,
    ) -> Result<Self> {
        let transport_mediator = Arc::new(TransportToolMediator::with_execution_hand_registry(
            Arc::clone(&execution_hand_registry),
        ));
        Self::new_with_runtime_mediator(
            workspace_root,
            execution_hand_registry,
            transport_mediator,
            model_id,
            model_paths,
            render_capability,
        )
    }

    pub fn new_with_runtime_mediator(
        workspace_root: impl Into<PathBuf>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        transport_mediator: Arc<TransportToolMediator>,
        model_id: &str,
        model_paths: ModelPaths,
        render_capability: RenderCapability,
    ) -> Result<Self> {
        let workspace_root = workspace_root.into();
        let model = ReusableQwenConversationFactory::load(PreparedQwenModel::from_paths(
            model_id,
            model_paths,
        )?)?;
        Ok(Self::from_factory(
            workspace_root,
            execution_hand_registry,
            transport_mediator,
            model_id,
            Box::new(model),
            render_capability,
        ))
    }

    fn from_factory(
        workspace_root: PathBuf,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        transport_mediator: Arc<TransportToolMediator>,
        model_id: &str,
        conversation_factory: Box<dyn ConversationFactory>,
        render_capability: RenderCapability,
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
        ensure_sift_process_cache_dirs();

        Self {
            workspace_root: workspace_root.clone(),
            execution_hand_registry,
            transport_mediator,
            model_id: model_id.to_string(),
            sift: Sift::builder()
                .with_cache_dir(default_sift_cache_dir_for_workspace(&workspace_root))
                .build(),
            conversation_factory,
            base_context,
            render_capability,
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
    pub(crate) fn new_for_test(
        workspace_root: impl Into<PathBuf>,
        model_id: &str,
        conversation: Box<dyn Conversation>,
    ) -> Self {
        let execution_hand_registry =
            Arc::new(ExecutionHandRegistry::with_default_local_governance());
        Self::from_factory(
            workspace_root.into(),
            Arc::clone(&execution_hand_registry),
            Arc::new(TransportToolMediator::with_execution_hand_registry(
                execution_hand_registry,
            )),
            model_id,
            Box::new(StaticConversationFactory::new(vec![conversation])),
            RenderCapability::PromptEnvelope,
        )
    }

    #[cfg(test)]
    pub(crate) fn new_for_test_with_conversations(
        workspace_root: impl Into<PathBuf>,
        model_id: &str,
        conversations: Vec<Box<dyn Conversation>>,
    ) -> Self {
        let execution_hand_registry =
            Arc::new(ExecutionHandRegistry::with_default_local_governance());
        Self::from_factory(
            workspace_root.into(),
            Arc::clone(&execution_hand_registry),
            Arc::new(TransportToolMediator::with_execution_hand_registry(
                execution_hand_registry,
            )),
            model_id,
            Box::new(StaticConversationFactory::new(conversations)),
            RenderCapability::PromptEnvelope,
        )
    }

    pub fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    pub fn respond(&self, prompt: &str) -> Result<String> {
        self.respond_internal(
            prompt,
            TurnIntent::DirectResponse,
            None,
            &SynthesisHandoff::default(),
            &NullTurnEventSink,
        )
    }

    pub fn respond_with_evidence(
        &self,
        prompt: &str,
        gathered_evidence: Option<&EvidenceBundle>,
    ) -> Result<String> {
        let intent = if gathered_evidence.is_some_and(|bundle| !bundle.items.is_empty()) {
            TurnIntent::Planned
        } else {
            TurnIntent::DirectResponse
        };
        self.respond_internal(
            prompt,
            intent,
            gathered_evidence,
            &SynthesisHandoff::default(),
            &NullTurnEventSink,
        )
    }

    pub fn respond_for_turn(
        &self,
        prompt: &str,
        turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        handoff: &SynthesisHandoff,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.respond_internal(
            prompt,
            turn_intent,
            gathered_evidence,
            handoff,
            event_sink.as_ref(),
        )
    }

    pub fn select_initial_action(
        &self,
        request: &PlannerRequest,
        _event_sink: &dyn TurnEventSink,
    ) -> Result<InitialActionDecision> {
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut reply = self.send_to_model(
            conversation.as_mut(),
            &build_initial_action_prompt(&PlannerPrompt {
                workspace_root: &request.workspace_root,
                user_prompt: &request.user_prompt,
                interpretation: &request.interpretation,
                request,
            }),
        )?;
        let mut parsed = parse_initial_action(&reply);

        if is_blank_model_reply(&reply)
            || parsed.as_ref().map_or(true, |decision| decision.is_none())
        {
            self.log_retry_reason(
                "initial-action-retry",
                &reply,
                "missing or invalid initial action response",
            );
            reply = self.send_to_model(
                conversation.as_mut(),
                &build_initial_action_retry_prompt(request),
            )?;
            parsed = parse_initial_action(&reply);
        }

        if is_blank_model_reply(&reply)
            || parsed.as_ref().map_or(true, |decision| decision.is_none())
        {
            self.log_retry_reason(
                "initial-action-redecision",
                &reply,
                "asking the planner for one final constrained initial action before failing closed",
            );
            reply = self.send_to_model(
                conversation.as_mut(),
                &build_initial_action_redecision_prompt(request, &reply),
            )?;
            parsed = parse_initial_action(&reply);
        }

        if let Ok(Some(decision)) = parsed {
            return Ok(decision);
        }

        self.log_retry_reason(
            "initial-action-fallback",
            &reply,
            "failing closed after invalid initial action replies",
        );
        Ok(fail_closed_initial_action(request))
    }

    pub fn select_planner_action(
        &self,
        request: &PlannerRequest,
        _event_sink: &dyn TurnEventSink,
    ) -> Result<RecursivePlannerDecision> {
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut reply = self.send_to_model(
            conversation.as_mut(),
            &build_planner_action_prompt(&PlannerPrompt {
                workspace_root: &request.workspace_root,
                user_prompt: &request.user_prompt,
                interpretation: &request.interpretation,
                request,
            }),
        )?;

        if is_blank_model_reply(&reply) || parse_planner_action(&reply)?.is_none() {
            self.log_retry_reason(
                "planner-retry",
                &reply,
                "missing or invalid planner action response",
            );
            reply =
                self.send_to_model(conversation.as_mut(), &build_planner_retry_prompt(request))?;
        }

        if parse_planner_action(&reply)?.is_none() {
            self.log_retry_reason(
                "planner-redecision",
                &reply,
                "asking the planner for one final constrained next action before failing closed",
            );
            reply = self.send_to_model(
                conversation.as_mut(),
                &build_planner_redecision_prompt(request, &reply),
            )?;
        }

        if let Some(decision) = parse_planner_action(&reply)? {
            return Ok(decision);
        }

        self.log_retry_reason(
            "planner-fallback",
            &reply,
            "failing closed after invalid planner replies",
        );
        Ok(fail_closed_planner_action())
    }

    pub fn select_thread_decision(
        &self,
        request: &ThreadDecisionRequest,
        _event_sink: &dyn TurnEventSink,
    ) -> Result<ThreadDecision> {
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut reply = self.send_to_model(
            conversation.as_mut(),
            &build_thread_decision_prompt(&ThreadPlannerPrompt {
                workspace_root: &request.workspace_root,
                interpretation: &request.interpretation,
                request,
            }),
        )?;

        if is_blank_model_reply(&reply) || parse_thread_decision(&reply, request)?.is_none() {
            self.log_retry_reason(
                "thread-decision-retry",
                &reply,
                "missing or invalid thread decision response",
            );
            reply = self.send_to_model(
                conversation.as_mut(),
                &build_thread_decision_retry_prompt(request),
            )?;
        }

        if let Some(decision) = parse_thread_decision(&reply, request)? {
            return Ok(decision);
        }

        self.log_retry_reason(
            "thread-decision-fallback",
            &reply,
            "falling back to bounded continue-current-thread behavior",
        );
        Ok(fallback_thread_decision(request))
    }

    pub fn derive_interpretation_context(
        &self,
        request: &InterpretationRequest,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<InterpretationContext> {
        if request.operator_memory.is_empty() {
            return Ok(InterpretationContext::default());
        }

        // --- Iteration 1: Initial Assembly ---
        let mut documents =
            self.expand_interpretation_guidance_graph(request, None, event_sink.clone())?;
        let mut context = self.assemble_interpretation_pass(request, &documents)?;

        // --- Validation Pass ---
        let mut conversation = self.conversation_factory.start_conversation()?;
        let validation_reply = self.send_to_model(
            conversation.as_mut(),
            &build_interpretation_validation_prompt(request, &context),
        )?;

        if let Some(validation) = parse_interpretation_validation(&validation_reply)?
            .filter(|v| v.needs_more_guidance && !v.gaps.is_empty())
        {
            // --- Iteration 2: Refined Assembly (Bounded) ---
            let new_documents = self.expand_interpretation_guidance_graph(
                request,
                Some(&validation.gaps),
                event_sink.clone(),
            )?;

            // Only proceed if we actually found new documents or different ones
            if !new_documents.is_empty() {
                let initial_doc_count = documents.len();
                documents.extend(new_documents);
                // Deduplicate by path
                let mut seen = std::collections::HashSet::new();
                documents.retain(|d| seen.insert(canonical_document_path(&d.path)));

                if documents.len() > initial_doc_count {
                    context = self.assemble_interpretation_pass(request, &documents)?;
                }
            }
        }

        Ok(context)
    }

    /// Evaluate context artifacts for relevance and produce a compaction plan.
    pub fn assess_context_relevance(&self, request: &CompactionRequest) -> Result<CompactionPlan> {
        // For now, we use a heuristic-driven self-assessment.
        // We preserve the most recent signals and discard deep history to maintain budget.
        let mut decisions = std::collections::HashMap::new();

        for (i, artifact_id) in request.target_scope.iter().enumerate() {
            let decision = if i == 0 {
                // Heuristic: Keep the most recent/relevant artifact (usually the prompt or latest interpretation)
                CompactionDecision::Keep { priority: 1 }
            } else if i < 3 {
                // Heuristic: Compact intermediate artifacts
                CompactionDecision::Compact {
                    summary: format!("Summary of artifact {}", artifact_id.as_str()),
                }
            } else {
                // Heuristic: Discard old history
                CompactionDecision::Discard {
                    reason: "Archived due to context strain".to_string(),
                }
            };
            decisions.insert(artifact_id.clone(), decision);
        }

        Ok(CompactionPlan { decisions })
    }

    fn assemble_interpretation_pass(
        &self,
        request: &InterpretationRequest,
        documents: &[OperatorMemoryDocument],
    ) -> Result<InterpretationContext> {
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut reply = self.send_to_model(
            conversation.as_mut(),
            &build_interpretation_context_prompt(request, documents),
        )?;

        if is_blank_model_reply(&reply) || parse_interpretation_context(&reply)?.is_none() {
            self.log_retry_reason(
                "interpretation-context-retry",
                &reply,
                "missing or invalid interpretation context response",
            );
            reply = self.send_to_model(
                conversation.as_mut(),
                &build_interpretation_context_retry_prompt(request, documents),
            )?;
        }

        if let Some(envelope) = parse_interpretation_context(&reply)? {
            return Ok(interpretation_context_from_envelope(
                envelope,
                &request.workspace_root,
                documents,
            ));
        }

        self.log_retry_reason(
            "interpretation-context-fallback",
            &reply,
            "falling back to AGENTS-rooted interpretation context only",
        );
        Ok(InterpretationContext::default())
    }

    pub fn recent_turn_summaries(&self) -> Result<Vec<String>> {
        let state = self
            .state
            .lock()
            .map_err(|_| anyhow!("Sift session state lock poisoned"))?;

        Ok(state
            .local_context
            .iter()
            .rev()
            .filter_map(|item| match item {
                LocalContextSource::AgentTurn(turn) => Some(format!(
                    "{}: {}",
                    turn.role,
                    trim_for_context(&turn.content, 180)
                )),
                _ => None,
            })
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect())
    }

    fn respond_internal(
        &self,
        prompt: &str,
        turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        handoff: &SynthesisHandoff,
        event_sink: &dyn TurnEventSink,
    ) -> Result<String> {
        let memory = AgentMemory::load(&self.workspace_root);
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            for warning in memory.warnings() {
                println!("[WARN] {warning}");
            }
        }
        let memory_prompt = memory.render_for_prompt();

        // Track context strain from memory truncation.
        let mut strain_tracker = StrainTracker::new();
        let mem_truncation_count = memory.truncation_count();
        if mem_truncation_count > 0 {
            strain_tracker.record_many(StrainFactor::MemoryTruncated, mem_truncation_count);
        }
        let strain = strain_tracker.finish();
        if !strain.is_nominal() {
            event_sink.emit(TurnEvent::ContextStrain { strain });
        }

        let mut state = self
            .state
            .lock()
            .map_err(|_| anyhow!("Sift session state lock poisoned"))?;

        state.turn_counter += 1;
        let turn_id = format!("turn-{}", state.turn_counter);
        let assistant_turn_id = format!("{turn_id}-assistant");
        let recent_turns = format_synthesis_recent_turns(handoff, &state.local_context);
        let recent_thread_summary = handoff
            .recent_thread_summary
            .as_deref()
            .filter(|summary| !summary.trim().is_empty());
        let instruction_handoff = format_instruction_handoff(handoff);

        let mut working_retained = state.retained_artifacts.clone();
        let mut working_local_context = state.local_context.clone();
        push_local_context(
            &mut working_local_context,
            LocalContextSource::AgentTurn(
                AgentTurnInput::new(&turn_id, "user", prompt).with_session_id(&state.session_id),
            ),
        );
        let direct_response_turn =
            matches!(turn_intent, TurnIntent::DirectResponse | TurnIntent::Casual);
        let require_grounding = gathered_evidence.is_some_and(|bundle| !bundle.items.is_empty());
        let prefer_tools = turn_intent.prefers_tools();
        let follow_up_execution = false;
        let mut conversation = self.conversation_factory.start_conversation()?;
        let mut rendered_parent_artifact_id = None;
        let mut rendered_exchange_id = None;
        let mut reply = if direct_response_turn {
            let (reply, exchange_id, raw_response_artifact_id) = self.send_to_model_for_turn(
                conversation.as_mut(),
                &build_direct_turn_prompt(
                    prompt,
                    &recent_turns,
                    recent_thread_summary,
                    &instruction_handoff,
                    &memory_prompt,
                    handoff,
                    self.render_capability,
                ),
                event_sink,
            )?;
            rendered_exchange_id = Some(exchange_id);
            rendered_parent_artifact_id = raw_response_artifact_id;
            reply
        } else if require_grounding {
            match gathered_evidence.filter(|bundle| !bundle.items.is_empty()) {
                Some(evidence) => {
                    let (reply, exchange_id, raw_response_artifact_id) = self
                        .send_to_model_for_turn(
                            conversation.as_mut(),
                            &build_grounded_turn_prompt(
                                prompt,
                                &recent_turns,
                                recent_thread_summary,
                                &instruction_handoff,
                                &memory_prompt,
                                evidence,
                                self.render_capability,
                            ),
                            event_sink,
                        )?;
                    rendered_exchange_id = Some(exchange_id);
                    rendered_parent_artifact_id = raw_response_artifact_id;
                    reply
                }
                None => {
                    event_sink.emit(TurnEvent::Fallback {
                        stage: "grounded-synthesis".to_string(),
                        reason: "no explicit evidence bundle was available for this planned turn"
                            .to_string(),
                    });
                    String::new()
                }
            }
        } else if prefer_tools {
            let initial_local_context = self.combined_local_context(&working_local_context);
            let initial_context =
                self.assemble_context(prompt, None, &initial_local_context, &working_retained)?;
            working_retained = initial_context.retained_artifacts.clone();
            self.log_context_assembly("initial", &initial_context, event_sink);

            let (reply, exchange_id, raw_response_artifact_id) = self.send_to_model_for_turn(
                conversation.as_mut(),
                &build_turn_prompt(&TurnPrompt {
                    workspace_root: &self.workspace_root,
                    user_prompt: prompt,
                    recent_turns: &recent_turns,
                    recent_thread_summary,
                    memory_prompt: &memory_prompt,
                    context: &initial_context,
                    gathered_evidence,
                    prefer_tools,
                    follow_up_execution,
                    render_capability: self.render_capability,
                }),
                event_sink,
            )?;
            rendered_exchange_id = Some(exchange_id);
            rendered_parent_artifact_id = raw_response_artifact_id;
            reply
        } else {
            let (reply, exchange_id, raw_response_artifact_id) = self.send_to_model_for_turn(
                conversation.as_mut(),
                &build_planned_direct_prompt(
                    prompt,
                    &recent_turns,
                    recent_thread_summary,
                    &instruction_handoff,
                    &memory_prompt,
                    gathered_evidence,
                    self.render_capability,
                ),
                event_sink,
            )?;
            rendered_exchange_id = Some(exchange_id);
            rendered_parent_artifact_id = raw_response_artifact_id;
            reply
        };

        if direct_response_turn {
            if is_blank_model_reply(&reply) || response_looks_like_tool_protocol(&reply)? {
                self.log_retry_reason(
                    "direct-response-retry",
                    &reply,
                    "empty or tool-like direct response",
                );
                let (next_reply, exchange_id, raw_response_artifact_id) = self
                    .send_to_model_for_turn(
                        conversation.as_mut(),
                        &build_direct_retry_prompt(
                            prompt,
                            &recent_turns,
                            recent_thread_summary,
                            &instruction_handoff,
                            &memory_prompt,
                            handoff,
                            self.render_capability,
                        ),
                        event_sink,
                    )?;
                rendered_exchange_id = Some(exchange_id);
                rendered_parent_artifact_id = raw_response_artifact_id;
                reply = next_reply;
            }
            if is_blank_model_reply(&reply) || response_looks_like_tool_protocol(&reply)? {
                self.log_retry_reason(
                    "direct-response-fallback",
                    &reply,
                    "repeated empty or tool-like direct response",
                );
                reply =
                    "I couldn't produce a usable response. Ask again or request a concrete workspace action.".to_string();
            }
        } else {
            if require_grounding {
                if is_blank_model_reply(&reply) || response_looks_like_tool_protocol(&reply)? {
                    self.log_retry_reason(
                        "grounded-retry",
                        &reply,
                        "empty or tool-like grounded response",
                    );
                    if let Some(evidence) =
                        gathered_evidence.filter(|bundle| !bundle.items.is_empty())
                    {
                        let (next_reply, exchange_id, raw_response_artifact_id) = self
                            .send_to_model_for_turn(
                                conversation.as_mut(),
                                &build_grounded_retry_prompt(
                                    prompt,
                                    &recent_turns,
                                    recent_thread_summary,
                                    &memory_prompt,
                                    evidence,
                                    handoff,
                                    self.render_capability,
                                ),
                                event_sink,
                            )?;
                        rendered_exchange_id = Some(exchange_id);
                        rendered_parent_artifact_id = raw_response_artifact_id;
                        reply = next_reply;
                    }
                }
            } else if prefer_tools
                && (is_blank_model_reply(&reply) || parse_tool_call(&reply)?.is_none())
            {
                self.log_retry_reason("tool-retry", &reply, "missing or empty tool call response");
                let (next_reply, exchange_id, raw_response_artifact_id) = self
                    .send_to_model_for_turn(
                        conversation.as_mut(),
                        &build_tool_retry_prompt(
                            prompt,
                            &recent_turns,
                            recent_thread_summary,
                            &memory_prompt,
                        ),
                        event_sink,
                    )?;
                rendered_exchange_id = Some(exchange_id);
                rendered_parent_artifact_id = raw_response_artifact_id;
                reply = next_reply;
            } else if is_blank_model_reply(&reply)
                || response_looks_like_malformed_tool_protocol(&reply)?
            {
                self.log_retry_reason("direct-retry", &reply, "empty or tool-like direct response");
                let (next_reply, exchange_id, raw_response_artifact_id) = self
                    .send_to_model_for_turn(
                        conversation.as_mut(),
                        &build_direct_retry_prompt(
                            prompt,
                            &recent_turns,
                            recent_thread_summary,
                            &instruction_handoff,
                            &memory_prompt,
                            handoff,
                            self.render_capability,
                        ),
                        event_sink,
                    )?;
                rendered_exchange_id = Some(exchange_id);
                rendered_parent_artifact_id = raw_response_artifact_id;
                reply = next_reply;
            }

            for _ in 0..MAX_TOOL_STEPS {
                let Some(tool_call) = parse_tool_call(&reply)? else {
                    break;
                };

                state.tool_counter += 1;
                let call_id = format!("tool-{}", state.tool_counter);
                let combined_context = self.combined_local_context(&working_local_context);
                event_sink.emit(TurnEvent::ToolCalled {
                    call_id: call_id.clone(),
                    tool_name: tool_call.name().to_string(),
                    invocation: describe_tool_call(&tool_call),
                });
                let result = match self.execute_tool(
                    &tool_call,
                    &call_id,
                    &combined_context,
                    &working_retained,
                    event_sink,
                ) {
                    Ok(result) => result,
                    Err(err) => ToolResult {
                        name: tool_call.name(),
                        summary: format!("Tool `{}` failed: {err:#}", tool_call.name()),
                        applied_edit: None,
                        governance_outcome: None,
                        retained_artifacts: None,
                    },
                };

                if let Some(retained) = result.retained_artifacts {
                    working_retained = retained;
                }
                if let Some(edit) = result.applied_edit.clone() {
                    event_sink.emit(TurnEvent::WorkspaceEditApplied {
                        call_id: call_id.clone(),
                        tool_name: result.name.to_string(),
                        edit,
                    });
                } else {
                    event_sink.emit(TurnEvent::ToolFinished {
                        call_id: call_id.clone(),
                        tool_name: result.name.to_string(),
                        summary: result.summary.clone(),
                    });
                }

                push_local_context(
                    &mut working_local_context,
                    LocalContextSource::ToolOutput(ToolOutputInput::new(
                        result.name,
                        &call_id,
                        result.summary.clone(),
                    )),
                );

                let (next_reply, exchange_id, raw_response_artifact_id) = self
                    .send_to_model_for_turn(
                        conversation.as_mut(),
                        &build_tool_follow_up_prompt(
                            prompt,
                            &call_id,
                            result.name,
                            &result.summary,
                            &memory_prompt,
                            self.render_capability,
                        ),
                        event_sink,
                    )?;
                rendered_exchange_id = Some(exchange_id);
                rendered_parent_artifact_id = raw_response_artifact_id;
                reply = next_reply;
            }

            if parse_tool_call(&reply)?.is_some() {
                bail!("tool step limit exceeded after {MAX_TOOL_STEPS} tool call(s)");
            }

            if is_blank_model_reply(&reply) {
                self.log_retry_reason(
                    "blank-fallback",
                    &reply,
                    "repeated empty response after retries",
                );
                reply =
                    "I couldn't produce a usable response. Ask again or request a concrete workspace action.".to_string();
            } else if !prefer_tools && response_looks_like_malformed_tool_protocol(&reply)? {
                self.log_retry_reason(
                    "tool-like-fallback",
                    &reply,
                    "repeated tool-like response after retries",
                );
                reply =
                    "I couldn't produce a usable response. Ask again or request a concrete workspace action.".to_string();
            }
        }

        reply = finalize_turn_reply(
            &self.workspace_root,
            prompt,
            reply,
            &turn_intent,
            gathered_evidence,
            handoff,
            event_sink,
        );
        self.record_rendered_turn_response(
            event_sink,
            rendered_exchange_id
                .as_deref()
                .unwrap_or("exchange:untracked"),
            &reply,
            rendered_parent_artifact_id,
        );

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
            if is_blank_model_reply(&response) {
                println!("[DEBUG] Model response: <empty>");
            } else {
                println!("[DEBUG] Model response: {}", response);
            }
        }

        Ok(response)
    }

    fn record_forensic_artifact(
        &self,
        event_sink: &dyn TurnEventSink,
        record: SiftForensicRecord,
    ) -> Option<TraceArtifactId> {
        event_sink.forensic_trace_sink().and_then(|sink| {
            sink.record_forensic_artifact(ForensicArtifactCapture {
                exchange_id: record.exchange_id,
                lane: TraceModelExchangeLane::Synthesizer,
                category: TraceModelExchangeCategory::TurnResponse,
                phase: record.phase,
                provider: "sift".to_string(),
                model: self.model_id.clone(),
                parent_artifact_id: record.parent_artifact_id,
                summary: record.summary,
                content: record.content,
                mime_type: record.mime_type,
                labels: std::collections::BTreeMap::new(),
            })
        })
    }

    fn send_to_model_for_turn(
        &self,
        conversation: &mut dyn Conversation,
        prompt: &str,
        event_sink: &dyn TurnEventSink,
    ) -> Result<(String, String, Option<TraceArtifactId>)> {
        let exchange_id = event_sink
            .forensic_trace_sink()
            .map(|sink| {
                sink.allocate_model_exchange_id(
                    TraceModelExchangeLane::Synthesizer,
                    TraceModelExchangeCategory::TurnResponse,
                )
            })
            .unwrap_or_else(|| "exchange:untracked".to_string());
        let assembled_context_id = self.record_forensic_artifact(
            event_sink,
            SiftForensicRecord {
                exchange_id: exchange_id.clone(),
                phase: TraceModelExchangePhase::AssembledContext,
                summary: "sift turn assembled context".to_string(),
                content: prompt.to_string(),
                mime_type: "text/plain".to_string(),
                parent_artifact_id: None,
            },
        );
        let request_envelope_id = self.record_forensic_artifact(
            event_sink,
            SiftForensicRecord {
                exchange_id: exchange_id.clone(),
                phase: TraceModelExchangePhase::ProviderRequest,
                summary: "sift local request envelope".to_string(),
                content: json!({
                    "runtime": "sift-native",
                    "model": self.model_id.clone(),
                    "max_tokens": MAX_MODEL_TOKENS,
                    "prompt": prompt,
                })
                .to_string(),
                mime_type: "application/json".to_string(),
                parent_artifact_id: assembled_context_id,
            },
        );
        let response = self.send_to_model(conversation, prompt)?;
        let raw_response_artifact_id = self.record_forensic_artifact(
            event_sink,
            SiftForensicRecord {
                exchange_id: exchange_id.clone(),
                phase: TraceModelExchangePhase::RawProviderResponse,
                summary: "sift raw model response".to_string(),
                content: response.clone(),
                mime_type: "text/plain".to_string(),
                parent_artifact_id: request_envelope_id,
            },
        );
        Ok((response, exchange_id, raw_response_artifact_id))
    }

    fn record_rendered_turn_response(
        &self,
        event_sink: &dyn TurnEventSink,
        exchange_id: &str,
        response: &str,
        parent_artifact_id: Option<TraceArtifactId>,
    ) -> Option<TraceArtifactId> {
        self.record_forensic_artifact(
            event_sink,
            SiftForensicRecord {
                exchange_id: exchange_id.to_string(),
                phase: TraceModelExchangePhase::RenderedResponse,
                summary: "sift rendered response".to_string(),
                content: response.to_string(),
                mime_type: "text/plain".to_string(),
                parent_artifact_id,
            },
        )
    }

    fn log_retry_reason(&self, stage: &str, response: &str, reason: &str) {
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            let observed = if is_blank_model_reply(response) {
                "<empty>"
            } else {
                "non-empty"
            };
            println!("[INFO] Response recovery ({stage}): {reason} (observed={observed}).");
        }
    }

    fn log_context_assembly(
        &self,
        label: &str,
        response: &ContextAssemblyResponse,
        event_sink: &dyn TurnEventSink,
    ) {
        if self.verbose.load(Ordering::Relaxed) >= 1 {
            println!(
                "[INFO] Context assembly ({label}) produced {} hit(s), retained {} artifact(s), pruned {}.",
                response.response.hits.len(),
                response.retained_artifacts.len(),
                response.pruned_artifacts
            );
        }
        event_sink.emit(TurnEvent::ContextAssembly {
            label: label.to_string(),
            hits: response.response.hits.len(),
            retained_artifacts: response.retained_artifacts.len(),
            pruned_artifacts: response.pruned_artifacts,
        });
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
        event_sink: &dyn TurnEventSink,
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
                self.log_context_assembly("search", &assembly, event_sink);
                Ok(ToolResult {
                    name: "search",
                    summary: format_search_summary(query, &assembly),
                    applied_edit: None,
                    governance_outcome: None,
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
                    applied_edit: None,
                    governance_outcome: None,
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
                    applied_edit: None,
                    governance_outcome: None,
                    retained_artifacts: None,
                })
            }
            ToolCall::WriteFile { path, content } => {
                let result = LocalWorkspaceEditor::with_runtime_mediator(
                    self.workspace_root.clone(),
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .write_file(path, content)?;
                Ok(ToolResult {
                    name: "write_file",
                    summary: result.summary,
                    applied_edit: result.applied_edit,
                    governance_outcome: result.governance_outcome,
                    retained_artifacts: None,
                })
            }
            ToolCall::ReplaceInFile {
                path,
                old,
                new,
                replace_all,
            } => {
                let result = LocalWorkspaceEditor::with_runtime_mediator(
                    self.workspace_root.clone(),
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .replace_in_file(path, old, new, *replace_all)?;
                Ok(ToolResult {
                    name: "replace_in_file",
                    summary: result.summary,
                    applied_edit: result.applied_edit,
                    governance_outcome: result.governance_outcome,
                    retained_artifacts: None,
                })
            }
            ToolCall::Shell { command } => {
                let output = run_background_terminal_command_with_runtime_mediator(
                    &self.workspace_root,
                    command,
                    "shell",
                    call_id,
                    event_sink,
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .with_context(|| format!("failed to execute shell command `{command}`"))?;
                let (summary, governance_outcome) = match output {
                    GovernedTerminalCommandResult::Executed(output) => {
                        let summary = format_command_summary("Shell command", command, &output);
                        if !output.status.success() {
                            bail!("{summary}");
                        }
                        (summary, None)
                    }
                    GovernedTerminalCommandResult::Blocked(outcome) => {
                        (summarize_governance_outcome(&outcome), Some(outcome))
                    }
                };
                Ok(ToolResult {
                    name: "shell",
                    summary,
                    applied_edit: None,
                    governance_outcome,
                    retained_artifacts: None,
                })
            }
            ToolCall::Diff { path } => {
                let result = LocalWorkspaceEditor::with_runtime_mediator(
                    self.workspace_root.clone(),
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .diff(path.as_deref())?;
                Ok(ToolResult {
                    name: "diff",
                    summary: result.summary,
                    applied_edit: result.applied_edit,
                    governance_outcome: result.governance_outcome,
                    retained_artifacts: None,
                })
            }
            ToolCall::ApplyPatch { patch } => {
                let result = LocalWorkspaceEditor::with_runtime_mediator(
                    self.workspace_root.clone(),
                    Arc::clone(&self.execution_hand_registry),
                    Arc::clone(&self.transport_mediator),
                )
                .apply_patch(patch)?;
                Ok(ToolResult {
                    name: "apply_patch",
                    summary: result.summary,
                    applied_edit: result.applied_edit,
                    governance_outcome: result.governance_outcome,
                    retained_artifacts: None,
                })
            }
        }
    }

    pub(crate) fn run_workspace_action(&self, action: &WorkspaceAction) -> Result<ToolResult> {
        let tool_call = tool_call_from_workspace_action(action).ok_or_else(|| {
            anyhow!(
                "workspace action `{}` is not executable via the tool adapter",
                action.label()
            )
        })?;
        self.execute_tool(
            &tool_call,
            "planner-workspace-action",
            &[],
            &[],
            &NullTurnEventSink,
        )
    }

    fn expand_interpretation_guidance_graph(
        &self,
        request: &InterpretationRequest,
        gaps: Option<&[InterpretationGapEnvelope]>,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<Vec<OperatorMemoryDocument>> {
        let mut seen = request
            .operator_memory
            .iter()
            .map(|document| canonical_document_path(&document.path))
            .collect::<std::collections::HashSet<_>>();
        let mut all_documents = request.operator_memory.clone();
        let mut frontier = request.operator_memory.clone();

        let depth_limit = if gaps.is_some() {
            1
        } else {
            MAX_INTERPRETATION_GRAPH_DEPTH
        };

        for depth in 0..depth_limit {
            if frontier.is_empty() || all_documents.len() >= MAX_INTERPRETATION_GRAPH_DOCS {
                break;
            }

            let mut conversation = self.conversation_factory.start_conversation()?;
            let prompt = if let Some(gaps) = gaps {
                build_interpretation_graph_refinement_prompt(
                    request,
                    &frontier,
                    &all_documents,
                    gaps,
                )
            } else {
                build_interpretation_graph_prompt(request, &frontier, &all_documents)
            };

            let mut reply = self.send_to_model(conversation.as_mut(), &prompt)?;

            if is_blank_model_reply(&reply) || parse_interpretation_graph(&reply)?.is_none() {
                self.log_retry_reason(
                    "interpretation-graph-retry",
                    &reply,
                    "missing or invalid guidance graph response",
                );
                let retry_prompt = if let Some(gaps) = gaps {
                    build_interpretation_graph_refinement_prompt(
                        request,
                        &frontier,
                        &all_documents,
                        gaps,
                    )
                } else {
                    build_interpretation_graph_retry_prompt(request, &frontier, &all_documents)
                };
                reply = self.send_to_model(conversation.as_mut(), &retry_prompt)?;
            }

            let Some(graph) = parse_interpretation_graph(&reply)? else {
                self.log_retry_reason(
                    "interpretation-graph-fallback",
                    &reply,
                    "falling back to AGENTS-rooted interpretation only",
                );
                break;
            };

            let initial_doc_count = all_documents.len();
            let source_index = frontier
                .iter()
                .chain(all_documents.iter())
                .map(|document| (document.source.clone(), document.path.clone()))
                .collect::<std::collections::HashMap<_, _>>();

            let mut next_frontier = Vec::new();
            for edge in graph.edges {
                let Some(source_path) = source_index.get(&edge.source) else {
                    continue;
                };
                for target in edge.targets {
                    if all_documents.len() >= MAX_INTERPRETATION_GRAPH_DOCS {
                        break;
                    }
                    let Some(document) =
                        load_guidance_document(source_path, &target, &request.workspace_root)?
                    else {
                        continue;
                    };
                    let canonical = canonical_document_path(&document.path);
                    if !seen.insert(canonical) {
                        continue;
                    }
                    next_frontier.push(document.clone());
                    all_documents.push(document);
                }
            }

            if all_documents.len() > initial_doc_count {
                event_sink.emit(TurnEvent::GuidanceGraphExpanded {
                    depth: depth + 1,
                    document_count: all_documents.len(),
                    sources: all_documents.iter().map(|d| d.source.clone()).collect(),
                });
            }

            frontier = next_frontier;
        }

        Ok(all_documents)
    }
}

impl crate::domain::ports::SynthesizerEngine for SiftAgentAdapter {
    fn set_verbose(&self, level: u8) {
        self.verbose.store(level, Ordering::Relaxed);
    }

    fn respond_for_turn(
        &self,
        prompt: &str,
        turn_intent: TurnIntent,
        gathered_evidence: Option<&EvidenceBundle>,
        handoff: &SynthesisHandoff,
        event_sink: Arc<dyn TurnEventSink>,
    ) -> Result<String> {
        self.respond_internal(
            prompt,
            turn_intent,
            gathered_evidence,
            handoff,
            event_sink.as_ref(),
        )
    }

    fn recent_turn_summaries(&self) -> Result<Vec<String>> {
        SiftAgentAdapter::recent_turn_summaries(self)
    }

    fn execute_workspace_action(
        &self,
        action: &WorkspaceAction,
    ) -> Result<crate::domain::ports::WorkspaceActionResult> {
        let result = self.run_workspace_action(action)?;
        Ok(crate::domain::ports::WorkspaceActionResult {
            name: result.name.to_string(),
            summary: result.summary,
            applied_edit: result.applied_edit,
            governance_outcome: result.governance_outcome,
        })
    }
}

fn build_turn_prompt(turn: &TurnPrompt<'_>) -> String {
    let routing_guidance = if turn.follow_up_execution {
        "This request refers to a previous turn. Resolve words like `it` or `that` using the recent conversation, then perform the implied action with a tool when possible."
    } else if turn.prefer_tools {
        "This request is action-oriented. If a workspace tool can answer it, call the tool instead of explaining a command the user could run."
    } else {
        "Use tools when they materially improve accuracy; otherwise answer directly."
    };

    format!(
        "You are Paddles, a local-first coding assistant operating inside the workspace `{}`.\n\
Use the provided workspace evidence first. If you have enough information, answer directly.\n\
Final answers should stay concise and focus on the requested result.\n\
When the user asks for a safe, reasonable repository change, your job is to make the workspace edit with a tool instead of stopping at diagnosis or advice once local evidence is sufficient.\n\
Routing guidance: {}\n\
Persistent operator memory:\n\
{}\n\
\n\
If you need a tool, respond with ONLY a single JSON tool call and no prose outside it.\n\
If you do not need a tool, respond with ONLY a single JSON final answer object using this rendering contract:\n\
{}\n\
\n\
When the user asks you to inspect repository state, run a command, read a file, search the workspace, or apply a change, prefer a tool call over describing how they could do it themselves.\n\
For `search.query`, return only the retrieval target terms. Do not prefix the query with tool verbs or instructions like `search`, `find`, `look for`, or `search for` unless those words are part of the literal text you need to match.\n\
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
Active thread summary:\n\
{}\n\
\n\
Gathered retrieval evidence:\n\
{}\n\
\n\
Current workspace evidence:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        turn.workspace_root.display(),
        routing_guidance,
        turn.memory_prompt,
        final_answer_contract_prompt(turn.render_capability, turn.gathered_evidence.is_some()),
        turn.recent_turns,
        turn.recent_thread_summary
            .unwrap_or("No active thread summary."),
        format_gathered_evidence_digest(turn.gathered_evidence),
        format_context_digest(turn.context),
        turn.user_prompt
    )
}

fn build_grounded_turn_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    instruction_handoff: &str,
    memory_prompt: &str,
    evidence: &EvidenceBundle,
    render_capability: RenderCapability,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "You are Paddles, a local-first coding assistant operating inside a repository.\n\
The planner lane gathered repository evidence for this workspace question.\n\
Answer ONLY from the gathered repository evidence below.\n\
Do not refer the user to external documentation.\n\
If the evidence is insufficient, say that explicitly.\n\
Include source/file citations in the final answer.\n\
Use this final answer rendering contract:\n\
{}\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Recent conversation:\n\
{recent_turns}\n\
\n\
Active thread summary:\n\
{thread_summary}\n\
\n\
Instruction manifold:\n\
{instruction_handoff}\n\
\n\
Gathered repository evidence:\n\
{}\n\
\n\
Current user request:\n\
{user_prompt}\n",
        final_answer_contract_prompt(render_capability, true),
        format_gathered_evidence_digest(Some(evidence)),
    )
}

fn build_direct_turn_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    instruction_handoff: &str,
    memory_prompt: &str,
    handoff: &SynthesisHandoff,
    render_capability: RenderCapability,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "You are Paddles, a local-first coding assistant.\n\
The user is making a conversational request that does not require workspace tools.\n\
Use this final answer rendering contract:\n\
{}\n\
Do not modify files or suggest workspace actions unless the user explicitly asks for them.\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Recent conversation:\n\
{recent_turns}\n\
\n\
Active thread summary:\n\
{thread_summary}\n\
\n\
Instruction manifold:\n\
{instruction_handoff}\n\
\n\
{}\
Current user request:\n\
{user_prompt}\n",
        final_answer_contract_prompt(render_capability, false),
        format_grounding_contract_section(handoff),
    )
}

fn build_planned_direct_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    instruction_handoff: &str,
    memory_prompt: &str,
    gathered_evidence: Option<&EvidenceBundle>,
    render_capability: RenderCapability,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "You are Paddles, a local-first coding assistant.\n\
This turn has already passed through the planner lane.\n\
Use this final answer rendering contract:\n\
{}\n\
If planner evidence is attached, use it and stay grounded.\n\
If no planner evidence is attached, do not invent repository-specific claims.\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Recent conversation:\n\
{recent_turns}\n\
\n\
Active thread summary:\n\
{thread_summary}\n\
\n\
Instruction manifold:\n\
{instruction_handoff}\n\
\n\
Planner evidence handoff:\n\
{}\n\
\n\
Current user request:\n\
{user_prompt}\n",
        final_answer_contract_prompt(render_capability, gathered_evidence.is_some()),
        format_gathered_evidence_digest(gathered_evidence),
    )
}

fn build_direct_retry_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    instruction_handoff: &str,
    memory_prompt: &str,
    handoff: &SynthesisHandoff,
    render_capability: RenderCapability,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "Your last reply tried to call a workspace tool for a conversational message.\n\
Use this final answer rendering contract:\n\
{}\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Recent conversation:\n\
{recent_turns}\n\
\n\
Active thread summary:\n\
{thread_summary}\n\
\n\
Instruction manifold:\n\
{instruction_handoff}\n\
\n\
{}\
Current user request:\n\
        {user_prompt}\n",
        final_answer_contract_prompt(render_capability, false),
        format_grounding_contract_section(handoff),
    )
}

fn format_instruction_handoff(handoff: &SynthesisHandoff) -> String {
    let base = match handoff.instruction_frame.as_ref() {
        Some(frame) if frame.requires_applied_edit() && frame.requires_applied_commit() => {
            if let Some(candidates) = frame.candidate_summary() {
                format!(
                    "Open obligation: this turn is not complete until Paddles applies the repository change and records the requested git commit.\nCandidate files: {candidates}"
                )
            } else {
                "Open obligation: this turn is not complete until Paddles applies the repository change and records the requested git commit."
                    .to_string()
            }
        }
        Some(frame) if frame.requires_applied_edit() => {
            if let Some(candidates) = frame.candidate_summary() {
                format!(
                    "Open obligation: this turn is not complete until Paddles applies a repository edit.\nCandidate files: {candidates}"
                )
            } else {
                "Open obligation: this turn is not complete until Paddles applies a repository edit."
                    .to_string()
            }
        }
        Some(frame) if frame.requires_applied_commit() => {
            "Open obligation: this turn is not complete until Paddles records the requested git commit."
                .to_string()
        }
        Some(_) => "Instruction obligations are currently satisfied.".to_string(),
        None => "No explicit instruction obligations are active.".to_string(),
    };

    let grounding = format_grounding_contract_section(handoff);
    if grounding.is_empty() {
        base
    } else {
        format!("{base}\n\n{}", grounding.trim_end())
    }
}

fn format_grounding_contract_section(handoff: &SynthesisHandoff) -> String {
    handoff
        .grounding
        .as_ref()
        .map(|grounding| {
            format!(
                "Grounding contract:\n{}\n\n",
                format_grounding_contract(grounding)
            )
        })
        .unwrap_or_default()
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

fn planner_grounding_rules() -> &'static str {
    "- Add optional top-level `grounding` when the final answer requires verified evidence before it can be trusted.\n\
- `grounding` must be {\"domain\":\"repository|external|mixed\",\"reason\":\"...\"}.\n\
- Use `grounding.domain = \"external\"` for websites, docs links, package pages, or any request to read about something on the web.\n\
- Use `grounding.domain = \"repository\"` for repository claims that need local evidence.\n\
- Use `grounding.domain = \"mixed\"` when both repository and external evidence are required.\n\
"
}

fn build_interpretation_validation_prompt(
    request: &InterpretationRequest,
    context: &InterpretationContext,
) -> String {
    format!(
        "You are the interpretation validator for Paddles.\n\
Review the current interpretation context against the user prompt and identify any missing guidance areas.\n\
Identify if there are rules, procedures, or conventions mentioned in the prompt that are not represented in the interpretation.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
\n\
Return this schema:\n\
{{\n\
  \"gaps\":[{{\"area\":\"...\",\"rationale\":\"...\"}}],\n\
  \"needs_more_guidance\":true|false\n\
}}\n\
\n\
User prompt:\n\
{}\n\
\n\
Current interpretation summary:\n\
{}\n\
\n\
Current interpretation documents:\n\
{}\n",
        request.user_prompt,
        context.summary,
        context
            .documents
            .iter()
            .map(|doc| format!("- {} ({:?})", doc.source, doc.category))
            .collect::<Vec<_>>()
            .join("\n")
    )
}

fn build_interpretation_graph_refinement_prompt(
    request: &InterpretationRequest,
    _frontier: &[OperatorMemoryDocument],
    loaded: &[OperatorMemoryDocument],
    gaps: &[InterpretationGapEnvelope],
) -> String {
    format!(
        "You are the operator-memory graph selector for Paddles (refinement phase).\n\
Your goal is to fill specific guidance gaps identified in the initial interpretation.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
\n\
Return this schema:\n\
{{\n\
  \"edges\":[{{\"source\":\"loaded/source/path\",\"targets\":[\"new/target/path\"]}}]\n\
}}\n\
\n\
Identified gaps:\n\
{}\n\
\n\
User prompt:\n\
{}\n\
\n\
Currently loaded sources:\n\
{}\n",
        gaps.iter()
            .map(|gap| format!("- {}: {}", gap.area, gap.rationale))
            .collect::<Vec<_>>()
            .join("\n"),
        request.user_prompt,
        format_operator_memory_documents(loaded),
    )
}

fn build_interpretation_graph_prompt(
    request: &InterpretationRequest,
    frontier: &[OperatorMemoryDocument],
    loaded: &[OperatorMemoryDocument],
) -> String {
    format!(
        "You are the operator-memory graph selector for Paddles.\n\
Read the frontier documents and select the next local files that should be loaded to interpret the current turn.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
\n\
Return this schema:\n\
{{\"edges\":[{{\"source\":\"frontier/source/path\",\"targets\":[\"relative/or/absolute/path\", \"...\"]}}]}}\n\
\n\
Rules:\n\
- The only roots are the loaded AGENTS.md memory files.\n\
- Use ONLY targets that are explicitly referenced in the source document text.\n\
- Keep the target path exactly as written in the source document.\n\
- Select only local files that help interpret the current user request.\n\
- Prefer a small relevant subgraph over a broad crawl.\n\
- Never invent file names.\n\
- If no more files should be loaded, return {{\"edges\":[]}}.\n\
\n\
Workspace root:\n\
{}\n\
\n\
Current user request:\n\
{}\n\
\n\
Already loaded operator-memory graph:\n\
{}\n\
\n\
Frontier documents to expand:\n\
{}\n",
        request.workspace_root.display(),
        request.user_prompt,
        format_operator_memory_documents(loaded),
        format_operator_memory_documents(frontier),
    )
}

fn build_interpretation_graph_retry_prompt(
    request: &InterpretationRequest,
    frontier: &[OperatorMemoryDocument],
    loaded: &[OperatorMemoryDocument],
) -> String {
    format!(
        "Your last operator-memory graph reply was empty or invalid.\n\
Return ONLY one valid JSON object with this shape:\n\
{{\"edges\":[{{\"source\":\"frontier/source/path\",\"targets\":[\"relative/or/absolute/path\"]}}]}}\n\
\n\
If no more files should be loaded, return {{\"edges\":[]}}.\n\
\n\
Current user request:\n\
{}\n\
\n\
Already loaded operator-memory graph:\n\
{}\n\
\n\
Frontier documents to expand:\n\
{}\n",
        request.user_prompt,
        format_operator_memory_documents(loaded),
        format_operator_memory_documents(frontier),
    )
}

fn build_interpretation_context_prompt(
    request: &InterpretationRequest,
    documents: &[OperatorMemoryDocument],
) -> String {
    format!(
        "You are the interpretation assembler for Paddles.\n\
Build the turn-time interpretation context from the loaded AGENTS-rooted guidance graph.\n\
Identify guidance categories, extract the precedence chain, and detect any conflicts.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
\n\
Return this schema:\n\
{{\n\
  \"summary\":\"...\",\n\
  \"documents\":[{{\"source\":\"loaded/source/path\",\"category\":\"rule|convention|constraint|preference\",\"excerpt\":\"...\"}}],\n\
  \"tool_hints\":[{{\"source\":\"loaded/source/path\",\"action\":{{...workspace action...}},\"note\":\"...\"}}],\n\
  \"procedures\":[{{\"source\":\"loaded/source/path\",\"label\":\"...\",\"purpose\":\"...\",\"steps\":[{{\"index\":0,\"action\":{{...workspace action...}},\"note\":\"...\"}}]}}],\n\
  \"precedence_chain\":[\"source1\", \"source2\"],\n\
  \"conflicts\":[{{\"source_a\":\"...\",\"source_b\":\"...\",\"description\":\"...\",\"resolution\":\"...\"}}],\n\
  \"coverage_confidence\":\"low|medium|high\"\n\
}}\n\
\n\
Categories:\n\
- rule: mandatory requirement or policy\n\
- convention: preferred style or approach\n\
- constraint: hard limitation (technical or procedural)\n\
- preference: operator-specific preference\n\
\n\
Precedence Rules:\n\
- Use the provided document hierarchy to establish the precedence chain.\n\
- Identify conflicts between guidance documents and state how you resolved them.\n\
- Assess coverage confidence: \"high\" if all aspects of the user prompt are covered by rules/procedures, \"low\" if major gaps exist.\n\
\n\
Workspace action schema:\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\"}}\n\
\n\
Rules:\n\
- Use ONLY the loaded document sources shown below.\n\
- Select only the documents, hints, and procedures that matter for the current turn.\n\
- Prefer a small relevant context over a broad dump.\n\
- Extract excerpts from the loaded guidance; do not invent file contents.\n\
- Tool hints and procedure steps should come from the guidance, not from controller guesses.\n\
- Use inspect only for read-only commands.\n\
- If no tool hints or procedures are justified, return empty arrays.\n\
\n\
Workspace root:\n\
{}\n\
\n\
Current user request:\n\
{}\n\
\n\
Loaded AGENTS-rooted guidance graph:\n\
{}\n",
        request.workspace_root.display(),
        request.user_prompt,
        format_operator_memory_documents(documents),
    )
}

fn build_interpretation_context_retry_prompt(
    request: &InterpretationRequest,
    documents: &[OperatorMemoryDocument],
) -> String {
    format!(
        "Your last interpretation-context reply was empty or invalid.\n\
Return ONLY one valid JSON object with this shape:\n\
{{\n\
  \"summary\":\"...\",\n\
  \"documents\":[{{\"source\":\"loaded/source/path\",\"category\":\"rule|convention|constraint|preference\",\"excerpt\":\"...\"}}],\n\
  \"tool_hints\":[{{\"source\":\"loaded/source/path\",\"action\":{{...workspace action...}},\"note\":\"...\"}}],\n\
  \"procedures\":[{{\"source\":\"loaded/source/path\",\"label\":\"...\",\"purpose\":\"...\",\"steps\":[{{\"index\":0,\"action\":{{...workspace action...}},\"note\":\"...\"}}]}}],\n\
  \"precedence_chain\":[\"source1\", \"source2\"],\n\
  \"conflicts\":[{{\"source_a\":\"...\",\"source_b\":\"...\",\"description\":\"...\",\"resolution\":\"...\"}}],\n\
  \"coverage_confidence\":\"low|medium|high\"\n\
}}\n\
\n\
Use ONLY the loaded document sources and only actions justified by the guidance.\n\
\n\
Current user request:\n\
{}\n\
\n\
Loaded AGENTS-rooted guidance graph:\n\
{}\n",
        request.user_prompt,
        format_operator_memory_documents(documents),
    )
}

fn format_operator_memory_documents(documents: &[OperatorMemoryDocument]) -> String {
    if documents.is_empty() {
        return "No operator-memory documents are loaded.".to_string();
    }

    let mut sections = Vec::new();
    for document in documents {
        sections.push(format!(
            "--- {} ---\n{}",
            document.source,
            trim_for_context(&document.contents, MAX_GRAPH_DOC_CHARS)
        ));
    }
    sections.join("\n\n")
}

fn build_initial_action_prompt(prompt: &PlannerPrompt<'_>) -> String {
    format!(
        "You are the top-level routing planner for Paddles.\n\
Choose the NEXT bounded action for this turn after reading the interpretation context.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
Every reply MUST include top-level `edit` and `candidate_files` fields.\n\
Core mission: when the user asks for a safe, reasonable repository change, make the workspace edit in this turn rather than stop at diagnosis or advice once local evidence is sufficient.\n\
\n\
Allowed actions:\n\
- {{\"action\":\"answer\",\"answer\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
\n\
Rules:\n\
- Read the interpretation context before choosing.\n\
- Answer or stop as soon as you have sufficient evidence. Do not use remaining budget for redundant or confirmatory searches.\n\
- For `answer`, put the user-facing reply in `answer` and keep `rationale` as the planner-only reason for selecting it.\n\
- Choose the most specific next workspace action when the turn requires repository work.\n\
- `edit` must be `yes` when the user is clearly asking for a code or file edit; otherwise return `no`.\n\
- `candidate_files` must list up to 3 plausible relative file paths to inspect or edit first. Use `[]` only when `edit` is `no`.\n\
	- Choose retrieval mode and strategy explicitly whenever you select search or refine.\n\
	- Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would outperform plain lexical or vector search.\n\
	- Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	- Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	- Use only fast retrieval strategies: `bm25` for keyword-heavy lookup or `vector` for semantic retrieval. Never request `hybrid`.\n\
- When the user requests a specific code or UI change, you are in execution mode. Use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
- Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
- If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
- For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
- Prefer a relevant interpretation tool hint over a generic search when the hint clearly matches the current request.\n\
- Use inspect for read-only shell commands and shell for broader workspace command execution.\n\
- When the user requests a code change, you MUST use write_file, replace_in_file, or apply_patch to make the edit — never describe the edit for the user to apply manually.\n\
- Search, list_files, read, inspect, shell, diff, refine, or branch when more workspace evidence or action is needed.\n\
- Stop when the turn should not recurse further before synthesis.\n\
- Never answer the user directly here.\n\
- Inspect commands must stay read-only.\n\
{}\n\
\n\
Workspace root:\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Interpretation tool hints:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        planner_grounding_rules(),
        prompt.workspace_root.display(),
        format_interpretation_context_digest(prompt.interpretation),
        format_interpretation_tool_hints(prompt.interpretation),
        format_decision_framework(prompt.interpretation),
        format_recent_turn_list(&prompt.request.recent_turns),
        prompt
            .request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&prompt.request.runtime_notes),
        prompt.user_prompt,
    )
}

fn build_initial_action_retry_prompt(request: &PlannerRequest) -> String {
    format!(
        "Your last top-level routing reply was empty or invalid.\n\
Return ONLY one valid JSON initial action.\n\
Every reply MUST include top-level `edit` and `candidate_files` fields.\n\
Core mission: when the user asks for a safe, reasonable repository change, make the workspace edit in this turn rather than stop at diagnosis or advice once local evidence is sufficient.\n\
\n\
Allowed actions:\n\
- {{\"action\":\"answer\",\"answer\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
\n\
Do not answer the user directly.\n\
For `answer`, put the user-facing reply in `answer` and keep `rationale` as the planner-only reason for selecting it.\n\
`edit` must be `yes` when the user is clearly asking for a code or file edit; otherwise return `no`.\n\
`candidate_files` must list up to 3 plausible relative file paths to inspect or edit first. Use `[]` only when `edit` is `no`.\n\
	Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would help.\n\
	Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	Use only fast retrieval strategies: `bm25` or `vector`. Never request `hybrid`.\n\
When the user requests a specific code or UI change, use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Interpretation tool hints:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        planner_grounding_rules(),
        format_interpretation_context_digest(&request.interpretation),
        format_interpretation_tool_hints(&request.interpretation),
        format_decision_framework(&request.interpretation),
        format_recent_turn_list(&request.recent_turns),
        request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&request.runtime_notes),
        request.user_prompt,
    )
}

fn build_initial_action_redecision_prompt(request: &PlannerRequest, invalid_reply: &str) -> String {
    format!(
        "Your previous initial-action replies were invalid.\n\
Make one final constrained routing decision.\n\
If no workspace action is clearly justified by the interpretation context, return stop.\n\
Return ONLY one valid JSON object.\n\
Every reply MUST include top-level `edit` and `candidate_files` fields.\n\
Core mission: when the user asks for a safe, reasonable repository change, make the workspace edit in this turn rather than stop at diagnosis or advice once local evidence is sufficient.\n\
\n\
Allowed actions:\n\
- {{\"action\":\"answer\",\"answer\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"edit\":\"yes\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"edit\":\"yes|no\",\"candidate_files\":[\"path1\",\"path2\",\"path3\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"edit\":\"no\",\"candidate_files\":[],\"rationale\":\"...\"}}\n\
\n\
Invalid reply to correct:\n\
{}\n\
\n\
`edit` must be `yes` when the user is clearly asking for a code or file edit; otherwise return `no`.\n\
For `answer`, put the user-facing reply in `answer` and keep `rationale` as the planner-only reason for selecting it.\n\
`candidate_files` must list up to 3 plausible relative file paths to inspect or edit first. Use `[]` only when `edit` is `no`.\n\
	Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would help.\n\
	Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	Use only fast retrieval strategies: `bm25` or `vector`. Never request `hybrid`.\n\
When the user requests a specific code or UI change, use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
If `edit` is `yes` and one likely target file is already known, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Interpretation tool hints:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        trim_for_context(invalid_reply, 800),
        planner_grounding_rules(),
        format_interpretation_context_digest(&request.interpretation),
        format_interpretation_tool_hints(&request.interpretation),
        format_decision_framework(&request.interpretation),
        format_recent_turn_list(&request.recent_turns),
        request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&request.runtime_notes),
        request.user_prompt,
    )
}

fn build_planner_action_prompt(prompt: &PlannerPrompt<'_>) -> String {
    format!(
        "You are the recursive planner lane for Paddles.\n\
Choose the NEXT bounded workspace resource action for this turn.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
Core mission: when the user asks for a safe, reasonable repository change, make the workspace edit in this turn rather than stop at diagnosis or advice once local evidence is sufficient.\n\
\n\
Allowed actions:\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"answer\":\"optional direct reply when ending immediately\",\"rationale\":\"...\"}}\n\
\n\
Rules:\n\
- Search when you need workspace retrieval.\n\
	- Choose retrieval mode and strategy explicitly when you search or refine.\n\
	- Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would outperform plain lexical or vector search.\n\
	- Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	- Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	- Use only fast retrieval strategies: `bm25` for keyword-heavy lookup or `vector` for semantic retrieval. Never request `hybrid`.\n\
- When the user requests a specific code or UI change, you are in execution mode. Use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
- Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
- If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
- For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
- List files when you need a bounded inventory of candidate files.\n\
- Read when a specific file or artifact should be opened.\n\
- Inspect when a read-only workspace command would clarify state.\n\
- Prefer a relevant interpretation tool hint over a generic search when the hint clearly matches the current request and has not been used yet.\n\
- Use shell, diff, or edit actions when the requested next step is a concrete workspace action that should stay inside the planner loop.\n\
- Refine when an earlier search needs a sharper query.\n\
- Branch when the investigation should split into multiple subqueries.\n\
- Stop as soon as you have enough evidence to answer. Do not use remaining budget for redundant or confirmatory searches — efficiency matters more than thoroughness.\n\
- If you are stopping because you already have the final user-facing answer, put that reply in `answer` and keep `rationale` for planner-only control reasoning.\n\
- When the user requests a code change, use write_file, replace_in_file, or apply_patch to make the edit directly — never describe the edit for the user to apply manually.\n\
- If the current loop state notes contain a `Steering review`, judge the proposed move against the gathered sources and return the action that should actually execute next.\n\
- Never answer the user directly here.\n\
- Inspect commands must stay read-only.\n\
{}\n\
\n\
Workspace root:\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Interpretation tool hints:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current loop state:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        planner_grounding_rules(),
        prompt.workspace_root.display(),
        format_interpretation_context_digest(prompt.interpretation),
        format_interpretation_tool_hints(prompt.interpretation),
        format_decision_framework(prompt.interpretation),
        format_recent_turn_list(&prompt.request.recent_turns),
        prompt
            .request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&prompt.request.runtime_notes),
        format_planner_loop_state_digest(prompt.request),
        prompt.user_prompt,
    )
}

fn build_planner_retry_prompt(request: &PlannerRequest) -> String {
    format!(
        "Your last planner reply was empty or invalid.\n\
Return ONLY one valid JSON planner action.\n\
\n\
Allowed actions:\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"answer\":\"optional direct reply when ending immediately\",\"rationale\":\"...\"}}\n\
\n\
Do not answer the user directly.\n\
If you are stopping because you already have the final user-facing answer, put that reply in `answer` and keep `rationale` for planner-only control reasoning.\n\
	Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would help.\n\
	Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	Use only fast retrieval strategies: `bm25` or `vector`. Never request `hybrid`.\n\
When the user requests a specific code or UI change, use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
If the current loop state notes contain a `Steering review`, judge the proposed move against the gathered sources and return the action that should actually execute next.\n\
For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Interpretation tool hints:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current loop state:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        planner_grounding_rules(),
        format_interpretation_context_digest(&request.interpretation),
        format_interpretation_tool_hints(&request.interpretation),
        format_decision_framework(&request.interpretation),
        format_recent_turn_list(&request.recent_turns),
        request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&request.runtime_notes),
        format_planner_loop_state_digest(request),
        request.user_prompt,
    )
}

fn build_planner_redecision_prompt(request: &PlannerRequest, invalid_reply: &str) -> String {
    format!(
        "Your previous planner replies were invalid.\n\
Make one final constrained next-action decision.\n\
If the loop state already contains enough evidence, return stop.\n\
Return ONLY one valid JSON planner action.\n\
\n\
Allowed actions:\n\
	- {{\"action\":\"search\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"intent\":\"optional\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"list_files\",\"pattern\":\"optional substring\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"read\",\"path\":\"relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"inspect\",\"command\":\"read-only shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"shell\",\"command\":\"workspace shell command\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"diff\",\"path\":\"optional relative/path\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"write_file\",\"path\":\"relative/path\",\"content\":\"full file contents\",\"rationale\":\"...\"}}\n\
- {{\"action\":\"replace_in_file\",\"path\":\"relative/path\",\"old\":\"exact old text\",\"new\":\"replacement text\",\"replace_all\":false,\"rationale\":\"...\"}}\n\
- {{\"action\":\"apply_patch\",\"patch\":\"unified diff text\",\"rationale\":\"...\"}}\n\
	- {{\"action\":\"refine\",\"query\":\"...\",\"mode\":\"linear|graph\",\"strategy\":\"bm25|vector\",\"retrievers\":[\"path-fuzzy\",\"segment-fuzzy\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"branch\",\"branches\":[\"...\",\"...\"],\"rationale\":\"...\"}}\n\
- {{\"action\":\"stop\",\"reason\":\"...\",\"answer\":\"optional direct reply when ending immediately\",\"rationale\":\"...\"}}\n\
\n\
Invalid reply to correct:\n\
{}\n\
\n\
	Optional `retrievers` may include `path-fuzzy` and `segment-fuzzy` when structural fuzzy lookup would help.\n\
	Use `retrievers:[\"path-fuzzy\"]` when the query names a likely file, path, selector, or symbol.\n\
	Use `retrievers:[\"path-fuzzy\",\"segment-fuzzy\"]` when you need fuzzy definition lookup for a structural code shape or snippet.\n\
	Use only fast retrieval strategies: `bm25` or `vector`. Never request `hybrid`.\n\
When the user requests a specific code or UI change, use at most one bounded search only if needed to identify the file, then move to list_files/read/apply_patch instead of continuing research.\n\
Action produces information. Once you have a plausible target file, prefer reading or editing it over another broad search.\n\
If one likely target file is already known or already read, move into exact-diff state space. For local, mechanical changes like padding, copy, a selector, one condition, or a small UI tweak, prefer `replace_in_file` or `apply_patch` over rereading the same file.\n\
If the current loop state notes contain a `Steering review`, judge the proposed move against the gathered sources and return the action that should actually execute next.\n\
If you are stopping because you already have the final user-facing answer, put that reply in `answer` and keep `rationale` for planner-only control reasoning.\n\
For `search.query` and `refine.query`, return concise retrieval terms, not an instruction sentence. Omit prefixes like `search`, `find`, `look for`, or `search for` unless they are part of the literal text to match.\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Derived decision framework:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread summary:\n\
{}\n\
\n\
Runtime notes:\n\
{}\n\
\n\
Current loop state:\n\
{}\n\
\n\
Current user request:\n\
{}\n",
        trim_for_context(invalid_reply, 800),
        planner_grounding_rules(),
        format_interpretation_context_digest(&request.interpretation),
        format_decision_framework(&request.interpretation),
        format_recent_turn_list(&request.recent_turns),
        request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        format_runtime_notes(&request.runtime_notes),
        format_planner_loop_state_digest(request),
        request.user_prompt,
    )
}

fn build_thread_decision_prompt(prompt: &ThreadPlannerPrompt<'_>) -> String {
    format!(
        "You are the steering-thread planner for Paddles.\n\
Choose how a steering prompt captured during an active turn should flow.\n\
Reply with ONLY one JSON object and no prose or markdown.\n\
\n\
Allowed decisions:\n\
- {{\"decision\":\"continue_current_thread\",\"rationale\":\"...\"}}\n\
- {{\"decision\":\"open_child_thread\",\"label\":\"...\",\"rationale\":\"...\"}}\n\
- {{\"decision\":\"merge_into_target\",\"target_thread_id\":\"mainline-or-thread-id\",\"merge_mode\":\"summary|backlink|merge\",\"summary\":\"optional\",\"rationale\":\"...\"}}\n\
\n\
Rules:\n\
- Continue when the steering prompt belongs on the active thread.\n\
- Open a child thread when the work should branch without mutating the mainline.\n\
- Merge into target when a child thread should reconcile back into a known thread or mainline.\n\
- Use ONLY target thread ids that exist in the known thread list.\n\
- Never answer the user directly here.\n\
\n\
Workspace root:\n\
{}\n\
\n\
Interpretation context:\n\
{}\n\
\n\
Recent turns:\n\
{}\n\
\n\
Active thread:\n\
- id={}\n\
- label={}\n\
\n\
Known threads:\n\
{}\n\
\n\
Recent thread summary:\n\
{}\n\
\n\
Steering candidate:\n\
- id={}\n\
- active_thread={}\n\
- prompt={}\n",
        prompt.workspace_root.display(),
        format_interpretation_context_digest(prompt.interpretation),
        format_recent_turn_list(&prompt.request.recent_turns),
        prompt.request.active_thread.thread_ref.stable_id(),
        prompt.request.active_thread.label,
        format_known_threads(&prompt.request.known_threads),
        prompt
            .request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        prompt.request.candidate.candidate_id.as_str(),
        prompt.request.candidate.active_thread.stable_id(),
        prompt.request.candidate.prompt,
    )
}

fn build_thread_decision_retry_prompt(request: &ThreadDecisionRequest) -> String {
    format!(
        "Your last steering-thread decision reply was empty or invalid.\n\
Return ONLY one valid JSON thread decision.\n\
\n\
Allowed decisions:\n\
- {{\"decision\":\"continue_current_thread\",\"rationale\":\"...\"}}\n\
- {{\"decision\":\"open_child_thread\",\"label\":\"...\",\"rationale\":\"...\"}}\n\
- {{\"decision\":\"merge_into_target\",\"target_thread_id\":\"mainline-or-thread-id\",\"merge_mode\":\"summary|backlink|merge\",\"summary\":\"optional\",\"rationale\":\"...\"}}\n\
\n\
Known threads:\n\
{}\n\
\n\
Recent thread summary:\n\
{}\n\
\n\
Steering candidate:\n\
- id={}\n\
- prompt={}\n",
        format_known_threads(&request.known_threads),
        request
            .recent_thread_summary
            .as_deref()
            .unwrap_or("No recent thread-local summary."),
        request.candidate.candidate_id.as_str(),
        request.candidate.prompt,
    )
}

fn build_grounded_retry_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    memory_prompt: &str,
    evidence: &EvidenceBundle,
    handoff: &SynthesisHandoff,
    render_capability: RenderCapability,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "Your last reply was empty or tried to call a tool for a repository question.\n\
Answer using ONLY the gathered repository evidence.\n\
Include source/file citations in the final answer.\n\
If the evidence is insufficient, say so explicitly.\n\
Use this final answer rendering contract:\n\
{}\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Recent conversation:\n\
{recent_turns}\n\
\n\
Active thread summary:\n\
{thread_summary}\n\
\n\
Gathered repository evidence:\n\
{}\n\
\n\
{}\
Current user request:\n\
{user_prompt}\n",
        final_answer_contract_prompt(render_capability, true),
        format_gathered_evidence_digest(Some(evidence)),
        format_grounding_contract_section(handoff),
    )
}

fn build_tool_retry_prompt(
    user_prompt: &str,
    recent_turns: &str,
    recent_thread_summary: Option<&str>,
    memory_prompt: &str,
) -> String {
    let thread_summary = recent_thread_summary.unwrap_or("No active thread summary.");
    format!(
        "The user asked for a workspace action and your last reply used prose instead of a tool.\n\
Reply with ONLY one JSON tool call and no prose.\n\
If the request refers to `it` or `that`, resolve it from the recent conversation first.\n\
\n\
Persistent operator memory:\n\
{memory_prompt}\n\
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
Active thread summary:\n\
{thread_summary}\n\
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
    memory_prompt: &str,
    render_capability: RenderCapability,
) -> String {
    format!(
        "Persistent operator memory:\n\
{memory_prompt}\n\
\n\
Tool call {call_id} ({tool_name}) completed.\n\
Tool result:\n\
{summary}\n\
\n\
Original user request:\n\
{user_prompt}\n\
\n\
If you need another tool, respond with ONLY one JSON tool call.\n\
Otherwise respond with ONLY one JSON final answer object using this rendering contract:\n\
{}",
        final_answer_contract_prompt(render_capability, false),
    )
}

fn finalize_turn_reply(
    workspace_root: &Path,
    prompt: &str,
    reply: String,
    turn_intent: &TurnIntent,
    gathered_evidence: Option<&EvidenceBundle>,
    handoff: &SynthesisHandoff,
    event_sink: &dyn TurnEventSink,
) -> String {
    let reply = normalize_assistant_response(&reply);
    let verified_external_urls = gathered_evidence
        .map(verified_external_urls_from_evidence)
        .unwrap_or_default();
    if external_grounding_required_without_verified_sources(handoff, &verified_external_urls) {
        event_sink.emit(TurnEvent::Fallback {
            stage: "grounding-governor".to_string(),
            reason: "planner declared external grounding, but no verified external sources were attached"
                .to_string(),
        });
        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: false,
            citations: Vec::new(),
            insufficient_evidence: true,
        });
        return external_grounding_unavailable_fallback(prompt);
    }
    if let Some(unverified_url) = first_unverified_external_url(&reply, &verified_external_urls) {
        event_sink.emit(TurnEvent::Fallback {
            stage: "grounding-governor".to_string(),
            reason: format!(
                "drafted answer referenced an unverified external URL without verified external sources: {unverified_url}"
            ),
        });
        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: false,
            citations: Vec::new(),
            insufficient_evidence: false,
        });
        return unverified_external_url_fallback(prompt);
    }
    let Some(evidence) = gathered_evidence else {
        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: false,
            citations: Vec::new(),
            insufficient_evidence: false,
        });
        return reply;
    };

    if evidence.items.is_empty() {
        let insufficient = turn_intent.uses_planner();
        event_sink.emit(TurnEvent::SynthesisReady {
            grounded: false,
            citations: Vec::new(),
            insufficient_evidence: insufficient,
        });
        return if insufficient {
            insufficient_evidence_reply(prompt)
        } else {
            reply
        };
    }

    let citations = citation_sources(workspace_root, evidence);
    let reply = if is_blank_model_reply(&reply)
        || response_looks_ungrounded(&reply)
        || !reply_contains_citation(&reply, &citations)
    {
        grounded_answer_fallback(workspace_root, evidence)
    } else {
        reply
    };
    let reply = ensure_citation_section(&reply, &citations);
    event_sink.emit(TurnEvent::SynthesisReady {
        grounded: true,
        citations: citations.clone(),
        insufficient_evidence: false,
    });
    reply
}

fn insufficient_evidence_reply(prompt: &str) -> String {
    format!(
        "I couldn't gather enough repository evidence to answer `{}` confidently.\n\nSources: none",
        prompt.trim()
    )
}

fn citation_sources(workspace_root: &Path, evidence: &EvidenceBundle) -> Vec<String> {
    let mut sources = Vec::new();
    for item in &evidence.items {
        let source = normalize_citation_source(workspace_root, &item.source);
        if !sources.contains(&source) {
            sources.push(source);
        }
    }
    if let Some(planner) = evidence.planner.as_ref() {
        for artifact in &planner.retained_artifacts {
            let source = normalize_citation_source(workspace_root, &artifact.source);
            if !sources.contains(&source) {
                sources.push(source);
            }
        }
        if let Some(graph) = planner.graph_episode.as_ref() {
            for branch in &graph.branches {
                for artifact in &branch.retained_artifacts {
                    let source = normalize_citation_source(workspace_root, &artifact.source);
                    if !sources.contains(&source) {
                        sources.push(source);
                    }
                }
            }
        }
    }
    if sources.len() > MAX_CITATIONS {
        sources.truncate(MAX_CITATIONS);
    }
    sources
}

fn reply_contains_citation(reply: &str, citations: &[String]) -> bool {
    citations.iter().any(|citation| {
        reply.contains(citation)
            || std::path::Path::new(citation)
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|basename| reply.contains(basename))
    })
}

fn response_looks_ungrounded(reply: &str) -> bool {
    let normalized = reply.to_ascii_lowercase();
    normalized.starts_with("i'm sorry")
        || normalized.contains("i didn't understand")
        || normalized.contains("could you please rephrase")
        || normalized.contains("couldn't produce a usable response")
        || normalized.contains("official documentation")
        || normalized.contains("refer to")
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

fn grounded_answer_fallback(workspace_root: &Path, evidence: &EvidenceBundle) -> String {
    let mut lines = vec![
        "I couldn't verify a clean grounded synthesis from the model reply, so here is the gathered repository evidence directly:"
            .to_string(),
        evidence.summary.clone(),
    ];

    for item in evidence.items.iter().take(3) {
        lines.push(format!(
            "- {}: {}",
            normalize_citation_source(workspace_root, &item.source),
            trim_for_context(&item.snippet, 180)
        ));
    }

    lines.join("\n")
}

fn normalize_citation_source(workspace_root: &Path, source: &str) -> String {
    let source_path = Path::new(source);
    if source_path.is_absolute() {
        return relative_path(workspace_root, source_path);
    }

    source.to_string()
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

fn format_gathered_evidence_digest(evidence: Option<&EvidenceBundle>) -> String {
    let Some(evidence) = evidence else {
        return "No pre-gathered retrieval evidence was provided.".to_string();
    };

    let mut lines = vec![evidence.summary.clone()];
    if let Some(planner) = evidence.planner.as_ref() {
        lines.push(format!(
            "Planner: strategy={}, mode={}, turns={}, steps={}, stop={}",
            format_planner_strategy(&planner.strategy),
            planner.mode.label(),
            planner.turn_count,
            planner.steps.len(),
            planner.stop_reason.as_deref().unwrap_or("none"),
        ));
        if let Some(graph) = planner.graph_episode.as_ref() {
            lines.push(format!(
                "Graph: active_branch={}, branches={}, frontier={}, completed={}",
                graph.active_branch_id.as_deref().unwrap_or("none"),
                graph.branches.len(),
                graph.frontier.len(),
                graph.completed
            ));
        }
        for step in planner.steps.iter().take(3) {
            let actions = step
                .decisions
                .iter()
                .map(|decision| {
                    decision
                        .query
                        .as_ref()
                        .map(|query| format!("{}({query})", decision.action))
                        .unwrap_or_else(|| decision.action.clone())
                })
                .collect::<Vec<_>>();
            lines.push(format!(
                "- planner step {}#{}: {}",
                step.step_id,
                step.sequence,
                actions.join(" -> "),
            ));
        }
        if !planner.retained_artifacts.is_empty() {
            lines.push(format!(
                "Retained artifacts: {}",
                planner
                    .retained_artifacts
                    .iter()
                    .take(3)
                    .map(|artifact| artifact.source.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }
    }
    if evidence.items.is_empty() {
        lines.push("No ranked evidence items were attached.".to_string());
    } else {
        for item in evidence.items.iter().take(MAX_CONTEXT_HITS) {
            lines.push(format!(
                "- [{}] {}: {}",
                item.rank,
                item.source,
                trim_for_context(&item.snippet, 240),
            ));
        }
    }

    if !evidence.warnings.is_empty() {
        lines.push(format!("Warnings: {}", evidence.warnings.join(" | ")));
    }

    lines.join("\n")
}

fn format_interpretation_context_digest(context: &InterpretationContext) -> String {
    context.render()
}

fn format_interpretation_tool_hints(context: &InterpretationContext) -> String {
    if context.tool_hints.is_empty() {
        return "No interpretation tool hints were available.".to_string();
    }

    context
        .tool_hints
        .iter()
        .map(|hint| {
            format!(
                "- {} ({}) — {}",
                hint.action.summary(),
                hint.source,
                trim_for_context(&hint.note, 160)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_decision_framework(context: &InterpretationContext) -> String {
    if context.decision_framework.procedures.is_empty() {
        return "No decision procedures were derived.".to_string();
    }

    context
        .decision_framework
        .procedures
        .iter()
        .map(|procedure| {
            let steps = procedure
                .steps
                .iter()
                .map(|step| step.action.summary())
                .collect::<Vec<_>>()
                .join(" -> ");
            format!(
                "- {} ({}) — {}\n  steps: {}",
                procedure.label,
                procedure.source,
                trim_for_context(&procedure.purpose, 160),
                trim_for_context(&steps, 200)
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_recent_turn_list(turns: &[String]) -> String {
    if turns.is_empty() {
        return "No recent turns were attached.".to_string();
    }

    turns
        .iter()
        .map(|turn| format!("- {}", trim_for_context(turn, 240)))
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_known_threads(threads: &[crate::domain::model::ConversationThread]) -> String {
    if threads.is_empty() {
        return "No known threads.".to_string();
    }

    threads
        .iter()
        .map(|thread| {
            let parent = thread
                .parent
                .as_ref()
                .map(|parent| parent.stable_id())
                .unwrap_or_else(|| "none".to_string());
            format!(
                "- id={} label={} status={} parent={}",
                thread.thread_ref.stable_id(),
                thread.label,
                thread.status.label(),
                parent
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn format_planner_loop_state_digest(request: &PlannerRequest) -> String {
    let remaining_steps = request
        .budget
        .max_steps
        .saturating_sub(request.loop_state.steps.len());
    let mut lines = vec![format!(
        "Budget remaining: steps={}, evidence_limit={}, pending_branches={}",
        remaining_steps,
        request.budget.max_evidence_items,
        request.loop_state.pending_branches.len()
    )];

    if request.loop_state.steps.is_empty() {
        lines.push("No planner steps have executed yet.".to_string());
    } else {
        for step in request.loop_state.steps.iter().rev().take(4).rev() {
            lines.push(format!(
                "- step {}: {} -> {}",
                step.sequence,
                step.action.summary(),
                trim_for_context(&step.outcome, 180)
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
                trim_for_context(&item.snippet, 180)
            ));
        }
    }

    let likely_targets = planner_likely_target_files(&request.loop_state, &request.workspace_root);
    if !likely_targets.is_empty() {
        lines.push("Likely target files:".to_string());
        for path in likely_targets {
            lines.push(format!("- {}", path));
        }
    }

    if !request.loop_state.notes.is_empty() {
        lines.push("Current notes:".to_string());
        for note in request.loop_state.notes.iter().take(3) {
            lines.push(format!("- {}", trim_for_context(note, 180)));
        }
    }

    if !request.loop_state.pending_branches.is_empty() {
        lines.push(format!(
            "Pending branches: {}",
            request
                .loop_state
                .pending_branches
                .iter()
                .map(|branch| branch.summary())
                .collect::<Vec<_>>()
                .join(" | ")
        ));
    }

    lines.join("\n")
}

fn format_runtime_notes(runtime_notes: &[String]) -> String {
    if runtime_notes.is_empty() {
        "No runtime notes.".to_string()
    } else {
        runtime_notes.join("\n")
    }
}

fn planner_likely_target_files(
    loop_state: &PlannerLoopState,
    workspace_root: &Path,
) -> Vec<String> {
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let mut ranked = loop_state
        .evidence_items
        .iter()
        .filter_map(|item| {
            planner_candidate_path(&item.source, &path_policy).map(|path| (path, item.rank))
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|(path_a, rank_a), (path_b, rank_b)| {
        planner_candidate_score(path_b, *rank_b)
            .cmp(&planner_candidate_score(path_a, *rank_a))
            .then_with(|| path_a.cmp(path_b))
    });
    ranked.dedup_by(|(path_a, _), (path_b, _)| path_a == path_b);
    ranked.into_iter().take(3).map(|(path, _)| path).collect()
}

fn planner_candidate_path(source: &str, path_policy: &WorkspacePathPolicy) -> Option<String> {
    if source.trim().is_empty() || source.starts_with("command: ") {
        return None;
    }
    let path = source.replace('\\', "/");
    if path_policy.allows_relative_file(&path) {
        Some(path)
    } else {
        None
    }
}

fn planner_candidate_score(path: &str, rank: usize) -> i32 {
    let mut score = if path.starts_with("src/") { 100 } else { 40 };
    score += match Path::new(path).extension().and_then(|ext| ext.to_str()) {
        Some("rs" | "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte") => 30,
        Some("html" | "css" | "json" | "toml") => 15,
        Some("md") => -20,
        Some(_) => 0,
        None => -40,
    };
    score - rank as i32
}

fn format_planner_strategy(strategy: &crate::domain::ports::PlannerStrategyKind) -> &'static str {
    match strategy {
        crate::domain::ports::PlannerStrategyKind::Heuristic => "heuristic",
        crate::domain::ports::PlannerStrategyKind::ModelDriven => "model-driven",
    }
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

fn format_synthesis_recent_turns(
    handoff: &SynthesisHandoff,
    local_context: &[LocalContextSource],
) -> String {
    let mut turns = handoff
        .recent_turns
        .iter()
        .map(|turn| format!("- {}", trim_for_context(turn, 240)))
        .collect::<Vec<_>>();

    for turn in format_recent_turns(local_context).lines() {
        let trimmed = turn.trim();
        if trimmed.is_empty() || trimmed == "No prior conversation in this session." {
            continue;
        }
        if turns.iter().any(|existing| existing == trimmed) {
            continue;
        }
        turns.push(trimmed.to_string());
    }

    if turns.is_empty() {
        return "No prior conversation in this session.".to_string();
    }

    turns.into_iter().take(6).collect::<Vec<_>>().join("\n")
}

fn describe_tool_call(tool_call: &ToolCall) -> String {
    match tool_call {
        ToolCall::Search { query, intent } => match intent {
            Some(intent) => format!("search workspace for `{query}` ({intent})"),
            None => format!("search workspace for `{query}`"),
        },
        ToolCall::ListFiles { pattern } => match pattern {
            Some(pattern) => format!("list files matching `{pattern}`"),
            None => "list workspace files".to_string(),
        },
        ToolCall::ReadFile { path } => format!("read `{path}`"),
        ToolCall::WriteFile { path, .. } => format!("write `{path}`"),
        ToolCall::ReplaceInFile { path, .. } => format!("replace text in `{path}`"),
        ToolCall::Shell { command } => command.clone(),
        ToolCall::Diff { path } => match path {
            Some(path) => format!("git diff --no-ext-diff -- {path}"),
            None => "git diff --no-ext-diff".to_string(),
        },
        ToolCall::ApplyPatch { .. } => "git apply --whitespace=nowarn -".to_string(),
    }
}

fn parse_tool_call(response: &str) -> Result<Option<ToolCall>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

fn parse_interpretation_graph(response: &str) -> Result<Option<InterpretationGraphEnvelope>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

fn parse_interpretation_validation(
    response: &str,
) -> Result<Option<InterpretationValidationEnvelope>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

fn parse_interpretation_context(response: &str) -> Result<Option<InterpretationContextEnvelope>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    Ok(serde_json::from_str(json).ok())
}

fn parse_initial_action(response: &str) -> Result<Option<InitialActionDecision>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    let Ok(action) = serde_json::from_str::<InitialActionEnvelope>(json) else {
        return Ok(None);
    };

    Ok(Some(initial_action_from_envelope(action)?))
}

fn parse_planner_action(response: &str) -> Result<Option<RecursivePlannerDecision>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    let Ok(action) = serde_json::from_str::<RecursiveActionEnvelope>(json) else {
        return Ok(None);
    };

    Ok(Some(planner_action_from_envelope(action)?))
}

fn parse_thread_decision(
    response: &str,
    request: &ThreadDecisionRequest,
) -> Result<Option<ThreadDecision>> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(None);
    };
    let Ok(decision) = serde_json::from_str::<ThreadDecisionEnvelope>(json) else {
        return Ok(None);
    };

    Ok(Some(thread_decision_from_envelope(decision, request)?))
}

fn interpretation_context_from_envelope(
    envelope: InterpretationContextEnvelope,
    workspace_root: &Path,
    loaded_documents: &[OperatorMemoryDocument],
) -> InterpretationContext {
    let allowed_sources = loaded_documents
        .iter()
        .map(|document| document.source.as_str())
        .collect::<std::collections::HashSet<_>>();
    let fallback_summary = format!(
        "Operator interpretation context assembled from {} AGENTS-rooted guidance document(s). Use it before choosing recursive workspace actions.",
        loaded_documents.len()
    );

    let documents = envelope
        .documents
        .into_iter()
        .filter(|document| allowed_sources.contains(document.source.as_str()))
        .filter_map(|document| {
            let excerpt = trim_for_context(&document.excerpt, 1_200);
            (!excerpt.trim().is_empty()).then(|| InterpretationDocument {
                source: normalize_citation_source(workspace_root, &document.source),
                excerpt,
                category: document.category,
            })
        })
        .take(5)
        .collect::<Vec<_>>();

    let tool_hints = envelope
        .tool_hints
        .into_iter()
        .filter(|hint| allowed_sources.contains(hint.source.as_str()))
        .filter_map(|hint| {
            let note = trim_for_context(&hint.note, 240);
            (!note.trim().is_empty()).then(|| InterpretationToolHint {
                source: normalize_citation_source(workspace_root, &hint.source),
                action: hint.action,
                note,
            })
        })
        .take(6)
        .collect::<Vec<_>>();

    let procedures = envelope
        .procedures
        .into_iter()
        .filter(|procedure| allowed_sources.contains(procedure.source.as_str()))
        .filter_map(|procedure| {
            let label = procedure.label.trim().to_string();
            let purpose = trim_for_context(&procedure.purpose, 240);
            let steps = procedure
                .steps
                .into_iter()
                .enumerate()
                .filter_map(|(index, step)| {
                    let note = trim_for_context(&step.note, 240);
                    (!note.trim().is_empty()).then_some(InterpretationProcedureStep {
                        index,
                        action: step.action,
                        note,
                    })
                })
                .collect::<Vec<_>>();
            (!label.is_empty() && !purpose.trim().is_empty() && !steps.is_empty()).then(|| {
                InterpretationProcedure {
                    source: normalize_citation_source(workspace_root, &procedure.source),
                    label,
                    purpose,
                    steps,
                }
            })
        })
        .take(4)
        .collect::<Vec<_>>();

    let summary = envelope.summary.trim();
    InterpretationContext {
        summary: if summary.is_empty() {
            fallback_summary
        } else {
            trim_for_context(summary, 320)
        },
        documents,
        tool_hints,
        decision_framework: InterpretationDecisionFramework { procedures },
        precedence_chain: envelope.precedence_chain,
        conflicts: envelope
            .conflicts
            .into_iter()
            .map(|c| InterpretationConflict {
                source_a: normalize_citation_source(workspace_root, &c.source_a),
                source_b: normalize_citation_source(workspace_root, &c.source_b),
                description: c.description,
                resolution: c.resolution,
            })
            .collect(),
        coverage_confidence: envelope.coverage_confidence,
    }
}

fn initial_action_from_envelope(envelope: InitialActionEnvelope) -> Result<InitialActionDecision> {
    let edit = initial_edit_instruction_from_envelope(&envelope)?;
    let grounding = envelope.grounding.clone();
    let decision = match envelope.action {
        InitialActionVariantEnvelope::Answer { answer, rationale } => InitialActionDecision {
            action: InitialAction::Answer,
            rationale: required_planner_field("rationale", rationale)?,
            answer: Some(required_planner_field("answer", answer)?),
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Search {
            query,
            mode,
            strategy,
            retrievers,
            intent,
            rationale,
        } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Search {
                    query: required_planner_field("query", query)?,
                    mode,
                    strategy,
                    retrievers,
                    intent: intent.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::ListFiles { pattern, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::ListFiles {
                    pattern: pattern.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Read { path, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Read {
                    path: required_planner_field("path", path)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Inspect { command, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: required_planner_field("command", command)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Shell { command, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Shell {
                    command: required_planner_field("command", command)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Diff { path, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::Diff {
                    path: path.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::WriteFile {
            path,
            content,
            rationale,
        } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::WriteFile {
                    path: required_planner_field("path", path)?,
                    content,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::ReplaceInFile {
            path,
            old,
            new,
            replace_all,
            rationale,
        } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::ReplaceInFile {
                    path: required_planner_field("path", path)?,
                    old: required_planner_field("old", old)?,
                    new: required_planner_field("new", new)?,
                    replace_all,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::ApplyPatch { patch, rationale } => InitialActionDecision {
            action: InitialAction::Workspace {
                action: WorkspaceAction::ApplyPatch {
                    patch: required_planner_field("patch", patch)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit,
            grounding: grounding.clone(),
        },
        InitialActionVariantEnvelope::Refine {
            query,
            mode,
            strategy,
            retrievers,
            rationale,
        } => {
            let rationale_text = rationale
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            InitialActionDecision {
                action: InitialAction::Refine {
                    query: required_planner_field("query", query)?,
                    mode,
                    strategy,
                    retrievers,
                    rationale: rationale_text.clone(),
                },
                rationale: rationale_text.unwrap_or_else(|| "refine the investigation".to_string()),
                answer: None,
                edit,
                grounding: grounding.clone(),
            }
        }
        InitialActionVariantEnvelope::Branch {
            branches,
            rationale,
        } => {
            let branches = branches
                .into_iter()
                .map(|branch| branch.trim().to_string())
                .filter(|branch| !branch.is_empty())
                .collect::<Vec<_>>();
            if branches.is_empty() {
                bail!("initial action branch must include at least one branch");
            }
            InitialActionDecision {
                action: InitialAction::Branch {
                    branches,
                    rationale: rationale.clone(),
                },
                rationale: rationale.unwrap_or_else(|| "branch the investigation".to_string()),
                answer: None,
                edit,
                grounding: grounding.clone(),
            }
        }
        InitialActionVariantEnvelope::Stop {
            reason,
            rationale,
            answer,
        } => InitialActionDecision {
            action: InitialAction::Stop {
                reason: required_planner_field("reason", reason)?,
            },
            rationale: rationale.unwrap_or_else(|| "stop routing".to_string()),
            answer: answer
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            edit,
            grounding,
        },
    };

    Ok(decision)
}

fn edit_instruction_from_fields(
    edit: Option<&str>,
    candidate_files: Option<&Vec<String>>,
) -> Result<InitialEditInstruction> {
    let edit_value = edit
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("planner reply must include top-level `edit`"))?;
    let known_edit = match edit_value {
        "yes" | "true" => true,
        "no" | "false" => false,
        other => bail!("edit must be `yes` or `no`, got `{other}`"),
    };
    let candidate_files = candidate_files
        .ok_or_else(|| anyhow!("planner reply must include top-level `candidate_files`"))?
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

fn initial_edit_instruction_from_envelope(
    envelope: &InitialActionEnvelope,
) -> Result<InitialEditInstruction> {
    edit_instruction_from_fields(envelope.edit.as_deref(), envelope.candidate_files.as_ref())
}

fn recursive_edit_instruction_from_envelope(
    envelope: &RecursiveActionEnvelope,
) -> Result<InitialEditInstruction> {
    edit_instruction_from_fields(envelope.edit.as_deref(), envelope.candidate_files.as_ref())
}

fn planner_action_from_envelope(
    envelope: RecursiveActionEnvelope,
) -> Result<RecursivePlannerDecision> {
    let edit = recursive_edit_instruction_from_envelope(&envelope)?;
    let grounding = envelope.grounding.clone();
    let decision = match envelope.action {
        PlannerActionEnvelope::Search {
            query,
            mode,
            strategy,
            retrievers,
            intent,
            rationale,
        } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Search {
                    query: required_planner_field("query", query)?,
                    mode,
                    strategy,
                    retrievers,
                    intent: intent.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::ListFiles { pattern, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::ListFiles {
                    pattern: pattern.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::Read { path, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Read {
                    path: required_planner_field("path", path)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::Inspect { command, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: required_planner_field("command", command)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::Shell { command, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Shell {
                    command: required_planner_field("command", command)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::Diff { path, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::Diff {
                    path: path.and_then(|value| {
                        let trimmed = value.trim();
                        (!trimmed.is_empty()).then(|| trimmed.to_string())
                    }),
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::WriteFile {
            path,
            content,
            rationale,
        } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::WriteFile {
                    path: required_planner_field("path", path)?,
                    content,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::ReplaceInFile {
            path,
            old,
            new,
            replace_all,
            rationale,
        } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::ReplaceInFile {
                    path: required_planner_field("path", path)?,
                    old: required_planner_field("old", old)?,
                    new: required_planner_field("new", new)?,
                    replace_all,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::ApplyPatch { patch, rationale } => RecursivePlannerDecision {
            action: PlannerAction::Workspace {
                action: WorkspaceAction::ApplyPatch {
                    patch: required_planner_field("patch", patch)?,
                },
            },
            rationale: required_planner_field("rationale", rationale)?,
            answer: None,
            edit: InitialEditInstruction::default(),
            grounding: grounding.clone(),
        },
        PlannerActionEnvelope::Refine {
            query,
            mode,
            strategy,
            retrievers,
            rationale,
        } => {
            let rationale_text = rationale
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            RecursivePlannerDecision {
                action: PlannerAction::Refine {
                    query: required_planner_field("query", query)?,
                    mode,
                    strategy,
                    retrievers,
                    rationale: rationale_text.clone(),
                },
                rationale: rationale_text.unwrap_or_else(|| "refine the investigation".to_string()),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: grounding.clone(),
            }
        }
        PlannerActionEnvelope::Branch {
            branches,
            rationale,
        } => {
            let branches = branches
                .into_iter()
                .map(|branch| branch.trim().to_string())
                .filter(|branch| !branch.is_empty())
                .collect::<Vec<_>>();
            if branches.is_empty() {
                bail!("planner branch action must include at least one branch");
            }
            RecursivePlannerDecision {
                action: PlannerAction::Branch {
                    branches,
                    rationale: rationale.clone(),
                },
                rationale: rationale.unwrap_or_else(|| "branch the investigation".to_string()),
                answer: None,
                edit: InitialEditInstruction::default(),
                grounding: grounding.clone(),
            }
        }
        PlannerActionEnvelope::Stop {
            reason,
            rationale,
            answer,
        } => RecursivePlannerDecision {
            action: PlannerAction::Stop {
                reason: required_planner_field("reason", reason)?,
            },
            rationale: rationale.unwrap_or_else(|| "stop planning".to_string()),
            answer: answer
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            edit: InitialEditInstruction::default(),
            grounding,
        },
    };
    Ok(RecursivePlannerDecision { edit, ..decision })
}

fn thread_decision_from_envelope(
    envelope: ThreadDecisionEnvelope,
    request: &ThreadDecisionRequest,
) -> Result<ThreadDecision> {
    let decision_id = ThreadDecisionId::new(format!(
        "{}.decision",
        request.candidate.candidate_id.as_str()
    ))?;

    let decision = match envelope {
        ThreadDecisionEnvelope::ContinueCurrentThread { rationale } => ThreadDecision {
            decision_id,
            candidate_id: request.candidate.candidate_id.clone(),
            kind: ThreadDecisionKind::ContinueCurrent,
            rationale: required_planner_field("rationale", rationale)?,
            target_thread: request.active_thread.thread_ref.clone(),
            new_thread_label: None,
            merge_mode: None,
            merge_summary: None,
        },
        ThreadDecisionEnvelope::OpenChildThread { label, rationale } => ThreadDecision {
            decision_id,
            candidate_id: request.candidate.candidate_id.clone(),
            kind: ThreadDecisionKind::OpenChildThread,
            rationale: required_planner_field("rationale", rationale)?,
            target_thread: request.active_thread.thread_ref.clone(),
            new_thread_label: Some(required_planner_field("label", label)?),
            merge_mode: None,
            merge_summary: None,
        },
        ThreadDecisionEnvelope::MergeIntoTarget {
            target_thread_id,
            merge_mode,
            summary,
            rationale,
        } => ThreadDecision {
            decision_id,
            candidate_id: request.candidate.candidate_id.clone(),
            kind: ThreadDecisionKind::MergeIntoTarget,
            rationale: required_planner_field("rationale", rationale)?,
            target_thread: required_thread_target(request, &target_thread_id)?,
            new_thread_label: None,
            merge_mode: Some(parse_merge_mode(&merge_mode)?),
            merge_summary: summary
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        },
    };

    Ok(decision)
}

fn required_planner_field(name: &str, value: String) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("planner field `{name}` must not be empty");
    }
    Ok(trimmed.to_string())
}

fn required_thread_target(
    request: &ThreadDecisionRequest,
    target_thread_id: &str,
) -> Result<ConversationThreadRef> {
    let normalized = target_thread_id.trim();
    if normalized.eq_ignore_ascii_case("mainline") {
        return Ok(ConversationThreadRef::Mainline);
    }

    request
        .known_threads
        .iter()
        .find_map(|thread| match &thread.thread_ref {
            ConversationThreadRef::Mainline => None,
            ConversationThreadRef::Branch(branch_id) if branch_id.as_str() == normalized => {
                Some(ConversationThreadRef::Branch(branch_id.clone()))
            }
            _ => None,
        })
        .ok_or_else(|| anyhow!("unknown target thread `{normalized}`"))
}

fn parse_merge_mode(value: &str) -> Result<ThreadMergeMode> {
    match value.trim() {
        "backlink" => Ok(ThreadMergeMode::Backlink),
        "summary" => Ok(ThreadMergeMode::Summary),
        "merge" => Ok(ThreadMergeMode::Merge),
        other => bail!("unknown merge mode `{other}`"),
    }
}

fn fail_closed_initial_action(request: &PlannerRequest) -> InitialActionDecision {
    InitialActionDecision {
        action: InitialAction::Stop {
            reason: format!(
                "initial-action-unavailable after invalid planner replies for `{}`",
                trim_for_context(&request.user_prompt, 120)
            ),
        },
        rationale: "controller failed closed after repeated invalid initial-action replies"
            .to_string(),
        answer: None,
        edit: InitialEditInstruction::default(),
        grounding: None,
    }
}

fn fail_closed_planner_action() -> RecursivePlannerDecision {
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

fn fallback_thread_decision(request: &ThreadDecisionRequest) -> ThreadDecision {
    ThreadDecision {
        decision_id: ThreadDecisionId::new(format!(
            "{}.decision-fallback",
            request.candidate.candidate_id.as_str()
        ))
        .expect("fallback decision id"),
        candidate_id: request.candidate.candidate_id.clone(),
        kind: ThreadDecisionKind::ContinueCurrent,
        rationale: "keep the steering prompt on the active thread when the thread decision reply is invalid"
            .to_string(),
        target_thread: request.active_thread.thread_ref.clone(),
        new_thread_label: None,
        merge_mode: None,
        merge_summary: None,
    }
}

fn tool_call_from_workspace_action(action: &WorkspaceAction) -> Option<ToolCall> {
    match action {
        WorkspaceAction::Search { query, intent, .. } => Some(ToolCall::Search {
            query: query.clone(),
            intent: intent.clone(),
        }),
        WorkspaceAction::ListFiles { pattern } => Some(ToolCall::ListFiles {
            pattern: pattern.clone(),
        }),
        WorkspaceAction::Read { path } => Some(ToolCall::ReadFile { path: path.clone() }),
        WorkspaceAction::Inspect { .. } => None,
        WorkspaceAction::Shell { command } => Some(ToolCall::Shell {
            command: command.clone(),
        }),
        WorkspaceAction::Diff { path } => Some(ToolCall::Diff { path: path.clone() }),
        WorkspaceAction::WriteFile { path, content } => Some(ToolCall::WriteFile {
            path: path.clone(),
            content: content.clone(),
        }),
        WorkspaceAction::ReplaceInFile {
            path,
            old,
            new,
            replace_all,
        } => Some(ToolCall::ReplaceInFile {
            path: path.clone(),
            old: old.clone(),
            new: new.clone(),
            replace_all: *replace_all,
        }),
        WorkspaceAction::ApplyPatch { patch } => Some(ToolCall::ApplyPatch {
            patch: patch.clone(),
        }),
    }
}

fn response_looks_like_tool_protocol(response: &str) -> Result<bool> {
    Ok(parse_tool_call(response)?.is_some()
        || response_looks_like_malformed_tool_protocol(response)?)
}

fn response_looks_like_malformed_tool_protocol(response: &str) -> Result<bool> {
    let trimmed = response.trim();
    let Some(json) = extract_json_payload(trimmed) else {
        return Ok(false);
    };
    if parse_tool_call(response)?.is_some() {
        return Ok(false);
    }

    Ok(match serde_json::from_str::<serde_json::Value>(json) {
        Ok(value) => value
            .get("tool")
            .and_then(serde_json::Value::as_str)
            .is_some(),
        Err(_) => json.contains("\"tool\""),
    })
}

fn is_blank_model_reply(response: &str) -> bool {
    response.trim().is_empty()
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

fn canonical_document_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
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
    let path_policy = WorkspacePathPolicy::new(workspace_root);
    let mut files = Vec::new();
    visit_files(
        workspace_root,
        workspace_root,
        &path_policy,
        pattern,
        &mut files,
    )?;
    files.sort();
    if files.len() > MAX_LISTED_FILES {
        files.truncate(MAX_LISTED_FILES);
    }
    Ok(files)
}

fn visit_files(
    dir: &Path,
    workspace_root: &Path,
    path_policy: &WorkspacePathPolicy,
    pattern: Option<&str>,
    files: &mut Vec<String>,
) -> Result<()> {
    if files.len() >= MAX_LISTED_FILES {
        return Ok(());
    }

    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)
            .with_context(|| format!("failed to stat {}", path.display()))?;

        if metadata.file_type().is_symlink() {
            continue;
        }

        if metadata.is_dir() {
            let Some(relative_dir) = path
                .strip_prefix(workspace_root)
                .ok()
                .map(|relative| relative.to_string_lossy().replace('\\', "/"))
            else {
                continue;
            };
            if !path_policy.allows_relative_directory(&relative_dir) {
                continue;
            }
            visit_files(&path, workspace_root, path_policy, pattern, files)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let rel = relative_path(workspace_root, &path);
        if path_policy.allows_relative_file(&rel)
            && pattern.is_none_or(|needle| rel.contains(needle))
        {
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
        LocalContextSource, MAX_TOOL_STEPS, QwenGenerationConfig, SiftAgentAdapter, ToolCall,
        extract_json_payload, format_qwen_prompt, generation_sampling, grounded_answer_fallback,
        normalize_relative_path, preferred_qwen_weight_dtype, should_retry_qwen_on_cpu_message,
        trim_for_context,
    };
    use crate::domain::model::{
        ForensicArtifactCapture, ForensicTraceSink, NullTurnEventSink, TraceArtifactId,
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase, TurnEvent,
        TurnEventSink, TurnIntent,
    };
    use crate::domain::ports::{
        EvidenceBundle, EvidenceItem, GroundingDomain, GroundingRequirement, InitialAction,
        InterpretationContext, InterpretationDecisionFramework, InterpretationProcedure,
        InterpretationProcedureStep, InterpretationRequest, InterpretationToolHint,
        OperatorMemoryDocument, PlannerAction, PlannerDecision, PlannerLoopState, PlannerRequest,
        PlannerStepRecord, PlannerStrategyKind, PlannerTraceMetadata, PlannerTraceStep,
        RefinementPolicy, RetainedEvidence, RetrievalMode, RetrievalStrategy, RetrieverOption,
        SynthesisHandoff, WorkspaceAction,
    };
    use crate::infrastructure::adapters::sift_agent::{
        DEFAULT_QWEN_MAX_LENGTH, QwenModelFamily, infer_qwen_family, infer_qwen_runtime_max_length,
    };
    use anyhow::{Result, anyhow};
    use candle_core::{DType, Device};
    use candle_transformers::generation::Sampling;
    use serde_json::json;
    use sift::Conversation;
    use std::collections::VecDeque;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs as unix_fs;
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct RecordingForensicSink {
        events: Mutex<Vec<TurnEvent>>,
        captures: Mutex<Vec<ForensicArtifactCapture>>,
    }

    impl RecordingForensicSink {
        fn captures(&self) -> Vec<ForensicArtifactCapture> {
            self.captures.lock().expect("captures lock").clone()
        }
    }

    impl TurnEventSink for RecordingForensicSink {
        fn emit(&self, event: TurnEvent) {
            self.events.lock().expect("events lock").push(event);
        }

        fn forensic_trace_sink(&self) -> Option<&dyn ForensicTraceSink> {
            Some(self)
        }
    }

    impl ForensicTraceSink for RecordingForensicSink {
        fn allocate_model_exchange_id(
            &self,
            _lane: TraceModelExchangeLane,
            _category: TraceModelExchangeCategory,
        ) -> String {
            let next = self.captures.lock().expect("captures lock").len() + 1;
            format!("exchange-{next:04}")
        }

        fn record_forensic_artifact(
            &self,
            capture: ForensicArtifactCapture,
        ) -> Option<TraceArtifactId> {
            let mut guard = self.captures.lock().expect("captures lock");
            let artifact_id =
                TraceArtifactId::new(format!("capture-{:04}", guard.len() + 1)).expect("id");
            guard.push(capture);
            Some(artifact_id)
        }
    }

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

    struct RecordingConversation {
        response: String,
        history: Arc<Mutex<Vec<String>>>,
    }

    impl RecordingConversation {
        fn new(response: impl Into<String>, history: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                response: response.into(),
                history,
            }
        }
    }

    impl Conversation for RecordingConversation {
        fn send(&mut self, message: &str, _max_tokens: usize) -> Result<String> {
            self.history
                .lock()
                .expect("history lock")
                .push(message.to_string());
            Ok(self.response.clone())
        }

        fn history(&self) -> &[String] {
            &[]
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
    fn qwen3_family_prompts_disable_thinking_explicitly() {
        let prompt = format_qwen_prompt(QwenModelFamily::Qwen3_5, "Hello");
        assert!(prompt.contains("<|im_start|>assistant\n<think>\n\n</think>\n\n"));
    }

    #[test]
    fn infers_qwen_family_from_prepared_config_shapes() {
        assert_eq!(
            infer_qwen_family(&json!({
                "model_type": "qwen2",
                "max_position_embeddings": 32768
            }))
            .expect("qwen2 family"),
            QwenModelFamily::Qwen2
        );

        assert_eq!(
            infer_qwen_family(&json!({
                "model_type": "qwen3",
                "architectures": ["Qwen3ForCausalLM"]
            }))
            .expect("qwen3 family"),
            QwenModelFamily::Qwen3
        );

        assert_eq!(
            infer_qwen_family(&json!({
                "model_type": "qwen3_5",
                "text_config": {
                    "model_type": "qwen3_5_text",
                    "max_position_embeddings": 262144
                },
                "architectures": ["Qwen3_5ForConditionalGeneration"]
            }))
            .expect("qwen3.5 family"),
            QwenModelFamily::Qwen3_5
        );
    }

    #[test]
    fn caps_prepared_qwen_runtime_length_to_existing_budget() {
        assert_eq!(
            infer_qwen_runtime_max_length(&json!({
                "max_position_embeddings": 32768
            })),
            DEFAULT_QWEN_MAX_LENGTH
        );
        assert_eq!(
            infer_qwen_runtime_max_length(&json!({
                "text_config": {
                    "max_position_embeddings": 128
                }
            })),
            128
        );
    }

    #[test]
    fn cpu_runtime_keeps_qwen_weights_in_f32() {
        assert_eq!(
            preferred_qwen_weight_dtype(QwenModelFamily::Qwen2, &Device::Cpu),
            DType::F32
        );
        assert_eq!(
            preferred_qwen_weight_dtype(QwenModelFamily::Qwen3_5, &Device::Cpu),
            DType::F32
        );
    }

    #[cfg(feature = "cuda")]
    #[test]
    fn cuda_runtime_uses_family_specific_weight_dtypes() {
        let device = Device::new_cuda(0).expect("cuda device");
        assert_eq!(
            preferred_qwen_weight_dtype(QwenModelFamily::Qwen2, &device),
            DType::BF16
        );
        assert_eq!(
            preferred_qwen_weight_dtype(QwenModelFamily::Qwen3_5, &device),
            DType::BF16
        );
    }

    #[test]
    fn qwen_generation_sampling_uses_model_defaults() {
        let config = QwenGenerationConfig {
            do_sample: true,
            eos_token_id: json!([151645, 151643]),
            repetition_penalty: Some(1.1),
            temperature: Some(0.7),
            top_p: Some(0.8),
            top_k: Some(20),
        };

        assert_eq!(
            generation_sampling(&config),
            Sampling::TopKThenTopP {
                k: 20,
                p: 0.8,
                temperature: 0.7
            }
        );
    }

    #[test]
    fn retries_qwen_cuda_runtime_on_oom_or_dtype_errors() {
        assert!(should_retry_qwen_on_cpu_message(
            true,
            "DriverError(CUDA_ERROR_OUT_OF_MEMORY, \"out of memory\")"
        ));
        assert!(should_retry_qwen_on_cpu_message(
            true,
            "unexpected dtype, expected: F32, got: BF16"
        ));
        assert!(!should_retry_qwen_on_cpu_message(
            false,
            "DriverError(CUDA_ERROR_OUT_OF_MEMORY, \"out of memory\")"
        ));
        assert!(!should_retry_qwen_on_cpu_message(
            true,
            "failed to load tokenizer: broken tokenizer"
        ));
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
            .respond_for_turn(
                "Where is the entrypoint?",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
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
    fn respond_with_evidence_includes_gathered_summary_in_prompt() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                "Here is the answer.",
                Arc::clone(&recorded_messages),
            )),
        );

        let evidence = EvidenceBundle::new(
            "Gathered repository evidence for runtime lanes.",
            vec![EvidenceItem {
                source: "src/application/mod.rs".to_string(),
                snippet: "PreparedRuntimeLanes keeps synthesizer and gatherer lanes.".to_string(),
                rationale: "Relevant runtime lane wiring.".to_string(),
                rank: 1,
            }],
        )
        .with_planner(PlannerTraceMetadata {
            mode: crate::domain::ports::RetrievalMode::Linear,
            strategy: PlannerStrategyKind::Heuristic,
            profile: None,
            session_id: Some("session-1".to_string()),
            completed: true,
            stop_reason: Some("goal-satisfied".to_string()),
            turn_count: 2,
            steps: vec![PlannerTraceStep {
                step_id: "step-1".to_string(),
                sequence: 1,
                parent_step_id: None,
                decisions: vec![PlannerDecision {
                    action: "search".to_string(),
                    query: Some("runtime lane architecture".to_string()),
                    rationale: Some("start with the lane wiring".to_string()),
                    next_step_id: None,
                    turn_id: Some("turn-1".to_string()),
                    branch_id: None,
                    node_id: None,
                    target_branch_id: None,
                    target_node_id: None,
                    edge_id: None,
                    edge_kind: None,
                    frontier_id: None,
                    stop_reason: None,
                }],
            }],
            retained_artifacts: vec![RetainedEvidence {
                source: "src/application/mod.rs".to_string(),
                snippet: Some(
                    "PreparedRuntimeLanes keeps synthesizer and gatherer lanes.".to_string(),
                ),
                rationale: Some("keep the runtime contract handy".to_string()),
                locator: None,
            }],
            graph_episode: None,
            trace_artifact_ref: None,
        });

        let reply = adapter
            .respond_with_evidence(
                "Summarize how runtime lanes work across the repo.",
                Some(&evidence),
            )
            .expect("response");
        assert!(reply.contains("Gathered repository evidence for runtime lanes."));
        assert!(reply.contains("PreparedRuntimeLanes keeps synthesizer and gatherer lanes."));
        assert!(reply.contains("Sources: src/application/mod.rs"));

        let prompt = recorded_messages
            .lock()
            .expect("history lock")
            .first()
            .cloned()
            .expect("recorded prompt");
        assert!(prompt.contains("Gathered repository evidence for runtime lanes."));
        assert!(prompt.contains("Planner: strategy=heuristic"));
        assert!(prompt.contains("planner step step-1#1"));
        assert!(prompt.contains("src/application/mod.rs"));
    }

    #[test]
    fn repository_turns_fall_back_to_grounded_evidence_when_model_is_confused() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "I'm sorry, I didn't understand the request.".to_string(),
                "I'm sorry, I didn't understand the request.".to_string(),
            ])),
        );

        let evidence = EvidenceBundle::new(
            "Gathered repository evidence for AGENTS.md memory loading.",
            vec![EvidenceItem {
                source: "src/infrastructure/adapters/agent_memory.rs".to_string(),
                snippet:
                    "AgentMemory::load reads /etc/paddles/AGENTS.md, ~/.config/paddles/AGENTS.md, then ancestor AGENTS.md files."
                        .to_string(),
                rationale: "Relevant to memory loading.".to_string(),
                rank: 1,
            }],
        );

        let reply = adapter
            .respond_for_turn(
                "How does memory work in paddles?",
                TurnIntent::Planned,
                Some(&evidence),
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert!(reply.contains("agent_memory.rs"));
        assert!(reply.contains("Sources: src/infrastructure/adapters/agent_memory.rs"));
        assert!(!reply.contains("didn't understand"));
    }

    #[test]
    fn repository_turns_without_evidence_report_insufficient_evidence() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec!["I do not know.".to_string()])),
        );
        let evidence = EvidenceBundle::new(
            "Planner reached synthesis without retained evidence.",
            Vec::new(),
        );

        let reply = adapter
            .respond_for_turn(
                "How does memory work in paddles?",
                TurnIntent::Planned,
                Some(&evidence),
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert!(reply.contains("couldn't gather enough repository evidence"));
        assert!(reply.contains("Sources: none"));
    }

    #[test]
    fn external_grounding_rejects_unverified_urls_in_sift_responses() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "You can read about it here: https://inception.ai/diffusion-llm".to_string(),
            ])),
        );
        let sink = Arc::new(RecordingForensicSink::default());

        let reply = adapter
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

        assert!(!reply.contains("https://inception.ai/diffusion-llm"));
        assert!(reply.contains("can't provide a verified external link"));
        assert!(
            sink.events
                .lock()
                .expect("events lock")
                .iter()
                .any(|event| matches!(
                    event,
                    TurnEvent::Fallback { stage, reason }
                        if stage == "grounding-governor"
                            && reason.contains("verified external sources")
                ))
        );
    }

    #[test]
    fn direct_prompts_include_loaded_agents_memory() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "Reply tersely.\nMention the active workspace when relevant.\n",
        )
        .expect("write AGENTS");
        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                "Hello.",
                Arc::clone(&recorded_messages),
            )),
        );

        let reply = adapter.respond("Hello").expect("response");
        assert_eq!(reply, "Hello.");

        let prompt = recorded_messages
            .lock()
            .expect("history lock")
            .first()
            .cloned()
            .expect("recorded prompt");
        assert!(prompt.contains("Persistent operator memory"));
        assert!(prompt.contains("Reply tersely."));
        assert!(prompt.contains("AGENTS.md"));
        assert!(prompt.contains("Final answer rendering contract"));
        assert!(prompt.contains("render_types"));
    }

    #[test]
    fn planned_prompts_include_loaded_agents_memory() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "Prefer concrete repository answers over generic advice.\n",
        )
        .expect("write AGENTS");
        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                "The repository layout is straightforward.\n\nSources: README.md",
                Arc::clone(&recorded_messages),
            )),
        );

        let reply = adapter
            .respond_for_turn(
                "Summarize the repository layout",
                TurnIntent::Planned,
                Some(&EvidenceBundle::new(
                    "Repository evidence for the runtime layout.",
                    vec![EvidenceItem {
                        source: "README.md".to_string(),
                        snippet: "The README explains the runtime layout.".to_string(),
                        rationale: "Ground the answer in repository evidence.".to_string(),
                        rank: 1,
                    }],
                )),
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");
        assert_eq!(
            reply,
            "The repository layout is straightforward.\n\nSources: README.md"
        );

        let prompt = recorded_messages
            .lock()
            .expect("history lock")
            .first()
            .cloned()
            .expect("recorded prompt");
        assert!(prompt.contains("Persistent operator memory"));
        assert!(prompt.contains("Prefer concrete repository answers over generic advice."));
        assert!(prompt.contains("Gathered repository evidence"));
        assert!(prompt.contains("Final answer rendering contract"));
        assert!(prompt.contains("citations"));
    }

    #[test]
    fn structured_final_answer_envelopes_are_normalized_before_return() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![json!({
                "render_types": ["paragraph", "bullet_list", "citations"],
                "blocks": [
                    { "type": "paragraph", "text": "The repository layout is straightforward." },
                    { "type": "bullet_list", "items": ["entrypoint: src/main.rs", "docs: README.md"] },
                    { "type": "citations", "sources": ["README.md"] }
                ]
            })
            .to_string()])),
        );

        let reply = adapter
            .respond_for_turn(
                "Summarize the repository layout",
                TurnIntent::Planned,
                Some(&EvidenceBundle::new(
                    "Repository evidence for the runtime layout.",
                    vec![EvidenceItem {
                        source: "README.md".to_string(),
                        snippet: "The README explains the runtime layout.".to_string(),
                        rationale: "Ground the answer in repository evidence.".to_string(),
                        rank: 1,
                    }],
                )),
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert_eq!(
            reply,
            "The repository layout is straightforward.\n\n- entrypoint: src/main.rs\n- docs: README.md\n\nSources: README.md"
        );
    }

    #[test]
    fn respond_for_turn_records_forensic_exchange_artifacts_on_trace_capable_sink() {
        let workspace = tempfile::tempdir().expect("workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                json!({
                    "render_types": ["paragraph"],
                    "blocks": [{ "type": "paragraph", "text": "Hello." }]
                })
                .to_string(),
            ])),
        );
        let sink = Arc::new(RecordingForensicSink::default());

        let reply = adapter
            .respond_for_turn(
                "Hello",
                TurnIntent::DirectResponse,
                None,
                &SynthesisHandoff::default(),
                sink.clone(),
            )
            .expect("reply");

        assert_eq!(reply, "Hello.");
        let captures = sink.captures();
        assert!(captures.iter().any(|capture| {
            capture.lane == TraceModelExchangeLane::Synthesizer
                && capture.category == TraceModelExchangeCategory::TurnResponse
                && capture.phase == TraceModelExchangePhase::AssembledContext
                && capture.content.contains("Current user request:")
        }));
        assert!(captures.iter().any(|capture| {
            capture.lane == TraceModelExchangeLane::Synthesizer
                && capture.category == TraceModelExchangeCategory::TurnResponse
                && capture.phase == TraceModelExchangePhase::ProviderRequest
                && capture.content.contains("\"runtime\":\"sift-native\"")
        }));
        assert!(captures.iter().any(|capture| {
            capture.lane == TraceModelExchangeLane::Synthesizer
                && capture.category == TraceModelExchangeCategory::TurnResponse
                && capture.phase == TraceModelExchangePhase::RawProviderResponse
                && capture.content.contains("\"render_types\"")
                && capture.content.contains("\"Hello.\"")
        }));
        assert!(captures.iter().any(|capture| {
            capture.lane == TraceModelExchangeLane::Synthesizer
                && capture.category == TraceModelExchangeCategory::TurnResponse
                && capture.phase == TraceModelExchangePhase::RenderedResponse
                && capture.content == "Hello."
        }));
    }

    #[test]
    fn initial_action_prompts_include_interpretation_context() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                r#"{"action":"answer","answer":"No workspace resources needed.","edit":"no","candidate_files":[],"rationale":"no workspace resources needed"}"#,
                Arc::clone(&recorded_messages),
            )),
        );

        let interpretation = InterpretationContext {
            summary: "Operator interpretation context assembled from AGENTS and linked docs."
                .to_string(),
            documents: vec![
                crate::domain::ports::InterpretationDocument {
                    source: "AGENTS.md".to_string(),
                    excerpt: "Use AGENTS guidance before choosing the next action.".to_string(),
                    category: crate::domain::ports::GuidanceCategory::Rule,
                },
                crate::domain::ports::InterpretationDocument {
                    source: "POLICY.md".to_string(),
                    excerpt: "Controller validates actions after the model selects them."
                        .to_string(),
                    category: crate::domain::ports::GuidanceCategory::Rule,
                },
            ],
            tool_hints: vec![InterpretationToolHint {
                source: "INSTRUCTIONS.md".to_string(),
                action: WorkspaceAction::Inspect {
                    command: "keel mission next".to_string(),
                },
                note: "Inspect current demand on the board.".to_string(),
            }],
            decision_framework: InterpretationDecisionFramework {
                procedures: vec![InterpretationProcedure {
                    source: "INSTRUCTIONS.md".to_string(),
                    label: "Inspect".to_string(),
                    purpose: "Read current demand on the board.".to_string(),
                    steps: vec![InterpretationProcedureStep {
                        index: 0,
                        action: WorkspaceAction::Inspect {
                            command: "keel mission next".to_string(),
                        },
                        note: "Read current demand on the board.".to_string(),
                    }],
                }],
            },
            ..Default::default()
        };
        let request = PlannerRequest::new(
            "What's next on the board?",
            workspace.path(),
            interpretation,
            crate::domain::ports::PlannerBudget::default(),
        )
        .with_runtime_notes(vec![
            "Workspace retrieval readiness: bm25=warming, vector=warming. Avoid search or refine until warmup completes.".to_string(),
        ])
        .with_recent_turns(vec!["user: previous turn".to_string()]);

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action");
        assert_eq!(decision.action, InitialAction::Answer);

        let prompt = recorded_messages
            .lock()
            .expect("history lock")
            .first()
            .cloned()
            .expect("recorded prompt");
        assert!(prompt.contains("Interpretation context"));
        assert!(prompt.contains("AGENTS.md"));
        assert!(prompt.contains("POLICY.md"));
        assert!(prompt.contains("Use AGENTS guidance before choosing the next action."));
        assert!(prompt.contains("Interpretation tool hints"));
        assert!(prompt.contains("keel mission next"));
        assert!(prompt.contains("Derived decision framework"));
        assert!(prompt.contains("Inspect (INSTRUCTIONS.md)"));
        assert!(prompt.contains("Recent turns"));
        assert!(prompt.contains("user: previous turn"));
        assert!(prompt.contains("Workspace retrieval readiness: bm25=warming, vector=warming"));
        assert!(prompt.contains("\"edit\":\"yes|no\""));
        assert!(prompt.contains("\"candidate_files\":[\"path1\",\"path2\",\"path3\"]"));
        assert!(prompt.contains("exact-diff state space"));
        assert!(prompt.contains("replace_in_file"));
        assert!(prompt.contains("apply_patch"));
        assert!(prompt.contains("safe, reasonable repository change"));
        assert!(prompt.contains("make the workspace edit in this turn"));
    }

    #[test]
    fn planner_action_prompts_include_exact_diff_guidance() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                r#"{"action":"stop","reason":"done","rationale":"enough context","answer":"patched the css","edit":"yes","candidate_files":["apps/web/src/runtime-shell.css"]}"#,
                Arc::clone(&recorded_messages),
            )),
        );

        let request = PlannerRequest::new(
            "The .runtime-shell-host class needs some padding. Something around 8px",
            workspace.path(),
            InterpretationContext::default(),
            crate::domain::ports::PlannerBudget::default(),
        )
        .with_loop_state(PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "planner-step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Read {
                        path: "apps/web/src/runtime-shell.css".to_string(),
                    },
                },
                outcome: "read the likely target file".to_string(),
            }],
            notes: vec!["Steering review [action-bias]\nIf one likely target file is already known or already read, move into exact-diff state space.".to_string()],
            ..PlannerLoopState::default()
        });

        adapter
            .select_planner_action(&request, &NullTurnEventSink)
            .expect("planner action");

        let prompt = recorded_messages
            .lock()
            .expect("history lock")
            .first()
            .cloned()
            .expect("recorded prompt");
        assert!(prompt.contains("exact-diff state space"));
        assert!(prompt.contains("replace_in_file"));
        assert!(prompt.contains("apply_patch"));
        assert!(prompt.contains("Steering review [action-bias]"));
        assert!(prompt.contains("safe, reasonable repository change"));
        assert!(prompt.contains("make the workspace edit in this turn"));
    }

    #[test]
    fn initial_action_answer_payload_is_preserved_separately_from_rationale() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                r#"{"action":"answer","answer":"Starter circuit\n\n[battery]---(solenoid)---(starter)","edit":"no","candidate_files":[],"rationale":"the user asked for a direct ASCII diagram"}"#,
                Arc::new(Mutex::new(Vec::new())),
            )),
        );

        let request = PlannerRequest::new(
            "Can you generate an ASCII diagram of the start circuit?",
            workspace.path(),
            InterpretationContext::default(),
            crate::domain::ports::PlannerBudget::default(),
        );

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action");

        assert_eq!(decision.action, InitialAction::Answer);
        assert_eq!(
            decision.rationale,
            "the user asked for a direct ASCII diagram"
        );
        assert_eq!(
            decision.answer.as_deref(),
            Some("Starter circuit\n\n[battery]---(solenoid)---(starter)")
        );
    }

    #[test]
    fn interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::write(
            workspace.path().join("AGENTS.md"),
            "Follow `INSTRUCTIONS.md` for the canonical turn loop.",
        )
        .expect("write AGENTS");
        fs::write(
            workspace.path().join("INSTRUCTIONS.md"),
            "Inspect with `keel mission next` and `keel pulse`.",
        )
        .expect("write INSTRUCTIONS");

        let recorded_messages = Arc::new(Mutex::new(Vec::new()));
        let adapter = SiftAgentAdapter::new_for_test_with_conversations(
            workspace.path(),
            "qwen-1.5b",
            vec![
                Box::new(RecordingConversation::new(
                    r#"{"edges":[{"source":"AGENTS.md","targets":["INSTRUCTIONS.md"]}]}"#,
                    Arc::clone(&recorded_messages),
                )),
                Box::new(RecordingConversation::new(
                    r#"{"edges":[]}"#,
                    Arc::clone(&recorded_messages),
                )),
                Box::new(RecordingConversation::new(
                    r#"{"summary":"Operator interpretation context assembled from AGENTS-rooted guidance document(s). Use it before choosing recursive workspace actions.","documents":[{"source":"AGENTS.md","excerpt":"Follow `INSTRUCTIONS.md` for the canonical turn loop.","category":"rule"},{"source":"INSTRUCTIONS.md","excerpt":"Inspect with `keel mission next` and `keel pulse`.","category":"rule"}],"tool_hints":[{"source":"INSTRUCTIONS.md","action":{"action":"inspect","command":"keel mission next"},"note":"Inspect current board demand before acting."}],"procedures":[{"source":"INSTRUCTIONS.md","label":"Inspect","purpose":"Read current demand on the board.","steps":[{"index":0,"action":{"action":"inspect","command":"keel mission next"},"note":"Inspect current board demand."}]}]}"#,
                    Arc::clone(&recorded_messages),
                )),
                Box::new(RecordingConversation::new(
                    r#"{"gaps":[],"needs_more_guidance":false}"#,
                    Arc::clone(&recorded_messages),
                )),
            ],
        );

        let interpretation = adapter
            .derive_interpretation_context(
                &InterpretationRequest::new(
                    "What's next on the keel board?",
                    workspace.path(),
                    vec![OperatorMemoryDocument {
                        path: workspace.path().join("AGENTS.md"),
                        source: "AGENTS.md".to_string(),
                        contents: "Follow `INSTRUCTIONS.md` for the canonical turn loop."
                            .to_string(),
                    }],
                ),
                Arc::new(NullTurnEventSink),
            )
            .expect("interpretation");

        assert!(interpretation.sources().contains(&"AGENTS.md".to_string()));
        assert!(
            interpretation
                .sources()
                .contains(&"INSTRUCTIONS.md".to_string())
        );
        assert!(
            interpretation
                .tool_hints
                .iter()
                .any(|hint| hint.action.summary().contains("keel mission next"))
        );

        let history = recorded_messages.lock().expect("history lock");
        assert!(
            history
                .first()
                .is_some_and(|prompt| prompt.contains("AGENTS.md"))
        );
        assert!(
            history
                .iter()
                .any(|prompt| prompt.contains("INSTRUCTIONS.md"))
        );
    }

    #[test]
    fn invalid_initial_action_replies_use_constrained_redecision_before_succeeding() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "not json".to_string(),
                "still not json".to_string(),
                r#"{"action":"inspect","command":"git status","edit":"no","candidate_files":[],"rationale":"confirm the repository state after invalid replies"}"#.to_string(),
            ])),
        );

        let request = PlannerRequest::new(
            "show me the git status",
            workspace.path(),
            InterpretationContext::default(),
            crate::domain::ports::PlannerBudget::default(),
        );

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action redecision");
        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "git status".to_string(),
                }
            }
        );
    }

    #[test]
    fn initial_action_retries_when_edit_metadata_is_missing() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"action":"search","query":".runtime-shell-host","mode":"linear","strategy":"bm25","rationale":"locate the selector"}"#.to_string(),
                r#"{"action":"search","query":".runtime-shell-host","mode":"linear","strategy":"bm25","edit":"yes","candidate_files":["apps/web/src/runtime-shell.css"],"rationale":"locate the selector"}"#.to_string(),
            ])),
        );

        let request = PlannerRequest::new(
            "The .runtime-shell-host class needs some padding. Something around 8px",
            workspace.path(),
            InterpretationContext::default(),
            crate::domain::ports::PlannerBudget::default(),
        );

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action");

        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Search {
                    query: ".runtime-shell-host".to_string(),
                    mode: RetrievalMode::Linear,
                    strategy: RetrievalStrategy::Lexical,
                    retrievers: Vec::new(),
                    intent: None,
                }
            }
        );
        assert!(decision.edit.known_edit);
        assert_eq!(
            decision.edit.candidate_files,
            vec!["apps/web/src/runtime-shell.css".to_string()]
        );
    }

    #[test]
    fn initial_action_can_request_structural_fuzzy_retrievers() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"action":"search","query":"runtime shell host","mode":"linear","strategy":"bm25","retrievers":["path-fuzzy","segment-fuzzy"],"edit":"yes","candidate_files":["apps/web/src/runtime-app.tsx"],"rationale":"use structural fuzzy lookup for the likely UI target"}"#.to_string(),
            ])),
        );

        let request = PlannerRequest::new(
            "Find the runtime shell host implementation",
            workspace.path(),
            InterpretationContext::default(),
            crate::domain::ports::PlannerBudget::default(),
        );

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action");

        assert_eq!(
            decision.action,
            InitialAction::Workspace {
                action: WorkspaceAction::Search {
                    query: "runtime shell host".to_string(),
                    mode: RetrievalMode::Linear,
                    strategy: RetrievalStrategy::Lexical,
                    retrievers: vec![RetrieverOption::PathFuzzy, RetrieverOption::SegmentFuzzy,],
                    intent: None,
                }
            }
        );
    }

    #[test]
    fn invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                "not json",
                Arc::new(Mutex::new(Vec::new())),
            )),
        );

        let request = PlannerRequest::new(
            "What's the next step on the keel board?",
            workspace.path(),
            InterpretationContext {
                summary: "Operator docs include relevant keel inspection commands.".to_string(),
                documents: vec![],
                tool_hints: vec![InterpretationToolHint {
                    source: "INSTRUCTIONS.md".to_string(),
                    action: WorkspaceAction::Inspect {
                        command: "keel mission next".to_string(),
                    },
                    note: "Inspect current demand on the board.".to_string(),
                }],
                decision_framework: InterpretationDecisionFramework {
                    procedures: vec![InterpretationProcedure {
                        source: "INSTRUCTIONS.md".to_string(),
                        label: "Inspect".to_string(),
                        purpose: "Read current demand on the board.".to_string(),
                        steps: vec![InterpretationProcedureStep {
                            index: 0,
                            action: WorkspaceAction::Inspect {
                                command: "keel mission next".to_string(),
                            },
                            note: "Read current demand on the board.".to_string(),
                        }],
                    }],
                },
                ..Default::default()
            },
            crate::domain::ports::PlannerBudget::default(),
        );

        let decision = adapter
            .select_initial_action(&request, &NullTurnEventSink)
            .expect("initial action fallback");
        assert_eq!(
            decision.action,
            InitialAction::Stop {
                reason:
                    "initial-action-unavailable after invalid planner replies for `What's the next step on the keel board?`"
                        .to_string()
            }
        );
    }

    #[test]
    fn invalid_planner_replies_use_constrained_redecision_before_succeeding() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                "not json".to_string(),
                "still not json".to_string(),
                r#"{"action":"inspect","command":"keel mission next","edit":"no","candidate_files":[],"rationale":"read the current board demand after invalid replies"}"#.to_string(),
            ])),
        );

        let request = PlannerRequest::new(
            "What's the next step on the keel board?",
            workspace.path(),
            InterpretationContext {
                summary: "Operator docs include relevant keel inspection commands.".to_string(),
                documents: vec![],
                tool_hints: vec![InterpretationToolHint {
                    source: "INSTRUCTIONS.md".to_string(),
                    action: WorkspaceAction::Inspect {
                        command: "keel mission next".to_string(),
                    },
                    note: "Inspect current demand on the board.".to_string(),
                }],
                decision_framework: InterpretationDecisionFramework {
                    procedures: vec![InterpretationProcedure {
                        source: "INSTRUCTIONS.md".to_string(),
                        label: "Inspect".to_string(),
                        purpose: "Read current demand on the board.".to_string(),
                        steps: vec![InterpretationProcedureStep {
                            index: 0,
                            action: WorkspaceAction::Inspect {
                                command: "keel mission next".to_string(),
                            },
                            note: "Read current demand on the board.".to_string(),
                        }],
                    }],
                },
                ..Default::default()
            },
            crate::domain::ports::PlannerBudget::default(),
        )
        .with_loop_state(PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Search {
                        query: "What's the next step on the keel board?".to_string(),
                        mode: RetrievalMode::Linear,
                        strategy: RetrievalStrategy::Lexical,
                        retrievers: Vec::new(),
                        intent: Some("initial planner fallback".to_string()),
                    },
                },
                outcome: "searched operator docs".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "AGENTS.md".to_string(),
                snippet: "Inspect current demand with `keel mission next`.".to_string(),
                rationale: "board navigation guidance".to_string(),
                rank: 1,
            }],
            notes: vec![],
            target_resolution: None,
            pending_branches: vec![],
            latest_gatherer_trace: None,
            refinement_count: 0,
            last_refinement_step: None,
            refinement_signatures: Vec::new(),
            refinement_policy: RefinementPolicy::default(),
        });

        let decision = adapter
            .select_planner_action(&request, &NullTurnEventSink)
            .expect("planner action redecision");
        assert_eq!(
            decision.action,
            PlannerAction::Workspace {
                action: WorkspaceAction::Inspect {
                    command: "keel mission next".to_string()
                }
            }
        );
    }

    #[test]
    fn invalid_planner_replies_fail_closed_after_redecision_is_still_invalid() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(RecordingConversation::new(
                "not json",
                Arc::new(Mutex::new(Vec::new())),
            )),
        );

        let request = PlannerRequest::new(
            "What's next on the keel board?",
            workspace.path(),
            InterpretationContext {
                summary: "Operator docs include a board inspect procedure.".to_string(),
                documents: vec![],
                tool_hints: vec![],
                decision_framework: InterpretationDecisionFramework {
                    procedures: vec![InterpretationProcedure {
                        source: "INSTRUCTIONS.md".to_string(),
                        label: "Inspect".to_string(),
                        purpose: "Read current demand on the board.".to_string(),
                        steps: vec![InterpretationProcedureStep {
                            index: 0,
                            action: WorkspaceAction::Inspect {
                                command: "keel mission next".to_string(),
                            },
                            note: "Read current demand on the board.".to_string(),
                        }],
                    }],
                },
                ..Default::default()
            },
            crate::domain::ports::PlannerBudget::default(),
        )
        .with_loop_state(PlannerLoopState {
            steps: vec![PlannerStepRecord {
                step_id: "step-1".to_string(),
                sequence: 1,
                branch_id: None,
                action: PlannerAction::Workspace {
                    action: WorkspaceAction::Inspect {
                        command: "keel mission next".to_string(),
                    },
                },
                outcome: "inspected keel mission next".to_string(),
            }],
            evidence_items: vec![EvidenceItem {
                source: "command: keel mission next".to_string(),
                snippet: "No actionable missions found.".to_string(),
                rationale: "live board state".to_string(),
                rank: 0,
            }],
            notes: vec![],
            target_resolution: None,
            pending_branches: vec![],
            latest_gatherer_trace: None,
            refinement_count: 0,
            last_refinement_step: None,
            refinement_signatures: Vec::new(),
            refinement_policy: RefinementPolicy::default(),
        });

        let decision = adapter
            .select_planner_action(&request, &NullTurnEventSink)
            .expect("planner action fallback");
        assert_eq!(
            decision.action,
            PlannerAction::Stop {
                reason: "planner-action-unavailable after invalid planner replies".to_string()
            }
        );
    }

    #[test]
    fn grounded_answer_fallback_preserves_evidence_order_without_source_priority() {
        let evidence = EvidenceBundle::new(
            "Planner collected board evidence.",
            vec![
                EvidenceItem {
                    source: "AGENTS.md".to_string(),
                    snippet: "Inspect current demand with `keel mission next`.".to_string(),
                    rationale: "operator guidance".to_string(),
                    rank: 1,
                },
                EvidenceItem {
                    source: "command: keel mission next".to_string(),
                    snippet: "No actionable missions found.".to_string(),
                    rationale: "live board state".to_string(),
                    rank: 0,
                },
            ],
        );

        let reply = grounded_answer_fallback(Path::new("."), &evidence);
        let bullets = reply
            .lines()
            .filter(|line| line.starts_with("- "))
            .collect::<Vec<_>>();
        assert_eq!(bullets.len(), 2);
        assert!(bullets[0].contains("command: keel mission next"));
        assert!(bullets[1].contains("AGENTS.md"));
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
                &NullTurnEventSink,
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
                &NullTurnEventSink,
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
                &NullTurnEventSink,
            )
            .expect("list files");

        assert!(result.summary.contains("main.rs"));
        assert!(!result.summary.contains("secret.txt"));
        assert!(!result.summary.contains("vault"));
    }

    #[test]
    fn list_files_respects_repo_gitignore_patterns() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::create_dir_all(workspace.path().join("apps/docs/.docusaurus"))
            .expect("create generated docs dir");
        fs::create_dir_all(workspace.path().join("apps/web/src")).expect("create authored app dir");
        fs::write(
            workspace.path().join(".gitignore"),
            "/apps/docs/.docusaurus/\n",
        )
        .expect("write gitignore");
        fs::write(
            workspace
                .path()
                .join("apps/docs/.docusaurus/client-modules.js"),
            "export default [];\n",
        )
        .expect("write generated docs module");
        fs::write(
            workspace.path().join("apps/web/src/runtime-app.tsx"),
            "export function RuntimeApp() { return null; }\n",
        )
        .expect("write authored runtime app");

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
                &NullTurnEventSink,
            )
            .expect("list files");

        assert!(result.summary.contains("apps/web/src/runtime-app.tsx"));
        assert!(
            !result
                .summary
                .contains("apps/docs/.docusaurus/client-modules.js")
        );
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
            .respond_for_turn(
                "Try reading the missing file.",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
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
    fn casual_prompts_retry_after_empty_response() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                String::new(),
                "Hello.".to_string(),
            ])),
        );

        let reply = adapter.respond("Hello").expect("response");
        assert_eq!(reply, "Hello.");

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 0);
    }

    #[test]
    fn casual_prompts_fall_back_after_repeated_empty_responses() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![String::new(), String::new()])),
        );

        let reply = adapter.respond("What's up?").expect("response");
        assert_eq!(
            reply,
            "I couldn't produce a usable response. Ask again or request a concrete workspace action."
        );

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 0);
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
        assert_eq!(
            reply,
            "I couldn't produce a usable response. Ask again or request a concrete workspace action."
        );

        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 0);
    }

    #[test]
    fn general_prompts_retry_after_malformed_tool_protocol_response() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(vec![
                r#"{"tool":"search","query_terms":"howdydy"}"#.to_string(),
                "Sure. How can I help?".to_string(),
            ])),
        );

        let reply = adapter
            .respond_for_turn(
                "What can you help with?",
                TurnIntent::Planned,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert_eq!(reply, "Sure. How can I help?");

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
            .respond_for_turn(
                "Inspect the repository status",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
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
            .respond_for_turn(
                "Inspect the repository status",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert_eq!(reply, "Working tree is clean.");
        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 1);
    }

    #[test]
    fn action_prompts_retry_for_tool_calls_after_empty_response() {
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
                String::new(),
                r#"{"tool":"shell","command":"git status"}"#.to_string(),
                "Working tree is clean.".to_string(),
            ])),
        );

        let reply = adapter
            .respond_for_turn(
                "Inspect the repository status",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert_eq!(reply, "Working tree is clean.");
        let state = adapter.state.lock().expect("state");
        assert_eq!(state.tool_counter, 1);
    }

    #[test]
    fn deterministic_action_turns_require_model_selected_tool_calls() {
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
                r#"{"tool":"shell","command":"git status"}"#.to_string(),
                "Working tree is clean.".to_string(),
            ])),
        );

        let reply = adapter
            .respond_for_turn(
                "Show me the git status",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect("response");

        assert_eq!(reply, "Working tree is clean.");
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
                &NullTurnEventSink,
            )
            .expect_err("shell failure should propagate");

        assert!(err.to_string().contains("Exit status"));
        assert!(err.to_string().contains("7"));
    }

    #[test]
    fn shell_tool_emits_terminal_output_events() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );
        let sink = RecordingForensicSink::default();

        let result = adapter
            .execute_tool(
                &ToolCall::Shell {
                    command: "printf 'alpha\\n'; printf 'warning\\n' >&2".to_string(),
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
                &sink,
            )
            .expect("shell tool should succeed");

        assert!(result.summary.contains("Shell command"));
        assert!(
            sink.events
                .lock()
                .expect("events lock")
                .iter()
                .any(|event| matches!(
                    event,
                    TurnEvent::ToolOutput {
                        tool_name,
                        stream,
                        output,
                        ..
                    } if tool_name == "shell"
                        && stream == "stdout"
                        && output.contains("alpha")
                ))
        );
        assert!(
            sink.events
                .lock()
                .expect("events lock")
                .iter()
                .any(|event| matches!(
                    event,
                    TurnEvent::ToolOutput {
                        tool_name,
                        stream,
                        output,
                        ..
                    } if tool_name == "shell"
                        && stream == "stderr"
                        && output.contains("warning")
                ))
        );
    }

    #[test]
    fn apply_patch_returns_error_on_failure() {
        let workspace = tempfile::tempdir().expect("temp workspace");
        fs::write(workspace.path().join("notes.txt"), "before\n").expect("seed file");
        let adapter = SiftAgentAdapter::new_for_test(
            workspace.path(),
            "qwen-1.5b",
            Box::new(MockConversation::new(Vec::new())),
        );

        let err = adapter
            .execute_tool(
                &ToolCall::ApplyPatch {
                    patch: "diff --git a/notes.txt b/notes.txt\n--- a/notes.txt\n+++ b/notes.txt\n@@ -1 +1 @@\n-missing\n+after\n"
                        .to_string(),
                },
                "tool-1",
                &adapter.combined_local_context(&[]),
                &[],
                &NullTurnEventSink,
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
            .respond_for_turn(
                "Keep listing files forever.",
                TurnIntent::DeterministicAction,
                None,
                &SynthesisHandoff::default(),
                Arc::new(NullTurnEventSink),
            )
            .expect_err("tool budget error");
        assert!(err.to_string().contains("tool step limit exceeded"));
    }

    #[test]
    fn assess_context_relevance_produces_heuristic_decisions() {
        use super::*;
        use crate::domain::model::{CompactionBudget, CompactionDecision, CompactionRequest};

        let sandbox = tempfile::tempdir().expect("sandbox");
        let workspace_root = sandbox.path().join("project");
        std::fs::create_dir_all(&workspace_root).expect("create dir");

        let execution_hand_registry = Arc::new(ExecutionHandRegistry::default());
        let adapter = SiftAgentAdapter::from_factory(
            workspace_root,
            Arc::clone(&execution_hand_registry),
            Arc::new(TransportToolMediator::with_execution_hand_registry(
                execution_hand_registry,
            )),
            "qwen-1.5b",
            Box::new(StaticConversationFactory::new(Vec::new())),
            crate::infrastructure::rendering::RenderCapability::resolve("openai", "gpt-4o"),
        );

        let request = CompactionRequest {
            target_scope: vec![
                paddles_conversation::TraceArtifactId::new("art-1").unwrap(),
                paddles_conversation::TraceArtifactId::new("art-2").unwrap(),
                paddles_conversation::TraceArtifactId::new("art-3").unwrap(),
                paddles_conversation::TraceArtifactId::new("art-4").unwrap(),
            ],
            relevance_query: "test".to_string(),
            budget: CompactionBudget::default(),
        };

        let plan = adapter.assess_context_relevance(&request).expect("assess");
        assert_eq!(plan.decisions.len(), 4);

        assert!(matches!(
            plan.decisions
                .get(&paddles_conversation::TraceArtifactId::new("art-1").unwrap()),
            Some(CompactionDecision::Keep { .. })
        ));
        assert!(matches!(
            plan.decisions
                .get(&paddles_conversation::TraceArtifactId::new("art-2").unwrap()),
            Some(CompactionDecision::Compact { .. })
        ));
        assert!(matches!(
            plan.decisions
                .get(&paddles_conversation::TraceArtifactId::new("art-4").unwrap()),
            Some(CompactionDecision::Discard { .. })
        ));
    }
}
