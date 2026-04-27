use crate::infrastructure::rendering::RenderCapability;
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ValueEnum)]
pub enum ModelProvider {
    /// Local Qwen inference via sift (default)
    Sift,
    /// OpenAI chat completions API
    Openai,
    /// Inception Labs chat completions API
    Inception,
    /// Anthropic messages API
    Anthropic,
    /// Google Gemini generateContent API
    Google,
    /// Moonshot Kimi (OpenAI-compatible)
    Moonshot,
    /// Ollama (OpenAI-compatible, localhost:11434)
    Ollama,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProviderAuthRequirement {
    None,
    OptionalApiKey,
    RequiredApiKey,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApiFormat {
    OpenAi,
    Anthropic,
    Gemini,
}

impl ApiFormat {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerToolCallCapability {
    NativeFunctionTool,
    StructuredJsonEnvelope,
    PromptEnvelope,
}

impl PlannerToolCallCapability {
    pub fn label(self) -> &'static str {
        match self {
            Self::NativeFunctionTool => "native-function-tool",
            Self::StructuredJsonEnvelope => "structured-json-envelope",
            Self::PromptEnvelope => "prompt-envelope",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliberationSupport {
    NativeContinuation,
    SummaryOnly,
    ToggleOnly,
    Unsupported,
}

impl DeliberationSupport {
    pub fn label(self) -> &'static str {
        match self {
            Self::NativeContinuation => "native_continuation",
            Self::SummaryOnly => "summary_only",
            Self::ToggleOnly => "toggle_only",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeliberationStateContract {
    OpaqueRoundTrip,
    None,
}

impl DeliberationStateContract {
    pub fn label(self) -> &'static str {
        match self {
            Self::OpaqueRoundTrip => "opaque_round_trip",
            Self::None => "none",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DeliberationCapabilitySurface {
    pub support: DeliberationSupport,
    pub state_contract: DeliberationStateContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderTransportSupport {
    Supported,
    Unsupported { reason: String },
}

impl ProviderTransportSupport {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Unsupported { .. } => "unsupported",
        }
    }
}

/// Shared provider/runtime contract that the harness negotiates from a
/// provider + model id pair before planner or synthesizer execution begins.
///
/// The rest of the runtime should consume this surface instead of matching on
/// provider names when the behavior is conceptually shared across providers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelCapabilitySurface {
    pub http_format: Option<ApiFormat>,
    pub render_capability: RenderCapability,
    pub planner_tool_call: PlannerToolCallCapability,
    pub transport_support: ProviderTransportSupport,
    pub deliberation: DeliberationCapabilitySurface,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModelThinkingMode {
    pub key: &'static str,
    pub label: &'static str,
    pub model_override: Option<&'static str>,
    pub thinking_mode: Option<&'static str>,
    pub runtime_control: bool,
}

const OPENAI_MODELS: &[&str] = &[
    "gpt-4.1-mini",
    "gpt-4.1",
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-5.5",
    "gpt-5.5-pro",
    "gpt-5.4",
    "gpt-5.4-pro",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.3-chat-latest",
    "gpt-5.3-codex",
    "gpt-5.2",
    "gpt-5.2-pro",
    "gpt-5.2-chat-latest",
    "gpt-5.2-codex",
    "gpt-5.1",
    "gpt-5.1-chat-latest",
    "gpt-5.1-codex",
    "gpt-5.1-codex-max",
    "gpt-5.1-codex-mini",
    "gpt-5",
    "gpt-5-pro",
    "gpt-5-mini",
    "gpt-5-nano",
    "gpt-5-chat-latest",
    "gpt-5-codex",
    "o3-pro",
    "o1-pro",
];
const OPENAI_RESPONSES_ONLY_MODELS: &[&str] = &[
    "gpt-5.5-pro",
    "gpt-5.4-pro",
    "gpt-5.2-pro",
    "gpt-5-pro",
    "o3-pro",
    "o1-pro",
];
const INCEPTION_MODELS: &[&str] = &["mercury-2"];
const ANTHROPIC_MODELS: &[&str] = &["claude-sonnet-4-20250514"];
const GOOGLE_MODELS: &[&str] = &["gemini-2.5-flash"];
const MOONSHOT_MODELS: &[&str] = &[
    "kimi-k2.6",
    "kimi-k2.5",
    "kimi-k2",
    "kimi-k2-0905-preview",
    "kimi-k2-0711-preview",
    "kimi-k2-turbo-preview",
    "kimi-k2-thinking",
    "kimi-k2-thinking-turbo",
];
const NONE_ONLY_THINKING_MODES: &[ModelThinkingMode] = &[ModelThinkingMode {
    key: "none",
    label: "None",
    model_override: None,
    thinking_mode: Some("none"),
    runtime_control: false,
}];
const MOONSHOT_SELECTABLE_MODELS: &[&str] = &[
    "kimi-k2.6",
    "kimi-k2.5",
    "kimi-k2",
    "kimi-k2-0905-preview",
    "kimi-k2-0711-preview",
];
const MOONSHOT_K2_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "default",
        label: "Default",
        model_override: Some("kimi-k2"),
        thinking_mode: None,
        runtime_control: false,
    },
    ModelThinkingMode {
        key: "turbo-preview",
        label: "Turbo preview",
        model_override: Some("kimi-k2-turbo-preview"),
        thinking_mode: None,
        runtime_control: false,
    },
    ModelThinkingMode {
        key: "thinking",
        label: "Thinking",
        model_override: Some("kimi-k2-thinking"),
        thinking_mode: None,
        runtime_control: false,
    },
    ModelThinkingMode {
        key: "thinking-turbo",
        label: "Thinking turbo",
        model_override: Some("kimi-k2-thinking-turbo"),
        thinking_mode: None,
        runtime_control: false,
    },
];
const MOONSHOT_BOOLEAN_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "thinking",
        label: "Thinking",
        model_override: None,
        thinking_mode: Some("enabled"),
        runtime_control: true,
    },
];
const OPENAI_NONE_LOW_MEDIUM_HIGH_XHIGH_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "xhigh",
        label: "XHigh",
        model_override: None,
        thinking_mode: Some("xhigh"),
        runtime_control: true,
    },
];
const OPENAI_NONE_LOW_MEDIUM_HIGH_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const OPENAI_MINIMAL_LOW_MEDIUM_HIGH_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "minimal",
        label: "Minimal",
        model_override: None,
        thinking_mode: Some("minimal"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const OPENAI_NONE_MEDIUM_HIGH_XHIGH_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "xhigh",
        label: "XHigh",
        model_override: None,
        thinking_mode: Some("xhigh"),
        runtime_control: true,
    },
];
const OPENAI_MEDIUM_HIGH_XHIGH_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "xhigh",
        label: "XHigh",
        model_override: None,
        thinking_mode: Some("xhigh"),
        runtime_control: true,
    },
];
const OPENAI_HIGH_ONLY_THINKING_MODES: &[ModelThinkingMode] = &[ModelThinkingMode {
    key: "high",
    label: "High",
    model_override: None,
    thinking_mode: Some("high"),
    runtime_control: true,
}];
const ANTHROPIC_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const GEMINI_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const INCEPTION_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "instant",
        label: "Instant",
        model_override: None,
        thinking_mode: Some("instant"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const OLLAMA_BOOLEAN_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "none",
        label: "None",
        model_override: None,
        thinking_mode: Some("none"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "thinking",
        label: "Thinking",
        model_override: None,
        thinking_mode: Some("thinking"),
        runtime_control: true,
    },
];
const OLLAMA_GPT_OSS_THINKING_MODES: &[ModelThinkingMode] = &[
    ModelThinkingMode {
        key: "low",
        label: "Low",
        model_override: None,
        thinking_mode: Some("low"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "medium",
        label: "Medium",
        model_override: None,
        thinking_mode: Some("medium"),
        runtime_control: true,
    },
    ModelThinkingMode {
        key: "high",
        label: "High",
        model_override: None,
        thinking_mode: Some("high"),
        runtime_control: true,
    },
];
const OPENAI_RUNTIME_THINKING_DELIMITER: &str = "@@thinking=";

fn ollama_model_family(model: &str) -> String {
    let without_namespace = model.rsplit('/').next().unwrap_or(model);
    without_namespace
        .split(':')
        .next()
        .unwrap_or(without_namespace)
        .trim()
        .to_ascii_lowercase()
}

fn ollama_supports_thinking(model: &str) -> bool {
    let family = ollama_model_family(model);
    family.starts_with("qwen3")
        || family.starts_with("gpt-oss")
        || family.starts_with("deepseek-v3.1")
        || family.starts_with("deepseek-r1")
}

fn openai_thinking_modes(model: &str) -> &'static [ModelThinkingMode] {
    if model == "gpt-5.5-pro" {
        return NONE_ONLY_THINKING_MODES;
    }
    if matches!(model, "o3-pro" | "o1-pro") {
        return NONE_ONLY_THINKING_MODES;
    }
    if model == "gpt-5-pro" {
        return OPENAI_HIGH_ONLY_THINKING_MODES;
    }
    if matches!(model, "gpt-5.4-pro" | "gpt-5.2-pro") {
        return OPENAI_MEDIUM_HIGH_XHIGH_THINKING_MODES;
    }
    if model == "gpt-5.1-codex-max" {
        return OPENAI_NONE_MEDIUM_HIGH_XHIGH_THINKING_MODES;
    }
    if model.starts_with("gpt-5.5") || model.starts_with("gpt-5.4") || model.starts_with("gpt-5.2")
    {
        return OPENAI_NONE_LOW_MEDIUM_HIGH_XHIGH_THINKING_MODES;
    }
    if model.starts_with("gpt-5.1") {
        return OPENAI_NONE_LOW_MEDIUM_HIGH_THINKING_MODES;
    }
    if model.starts_with("gpt-5") {
        return OPENAI_MINIMAL_LOW_MEDIUM_HIGH_THINKING_MODES;
    }
    NONE_ONLY_THINKING_MODES
}

fn moonshot_thinking_modes(model: &str) -> &'static [ModelThinkingMode] {
    match model {
        "kimi-k2.6" | "kimi-k2.5" => MOONSHOT_BOOLEAN_THINKING_MODES,
        "kimi-k2" => MOONSHOT_K2_THINKING_MODES,
        _ => NONE_ONLY_THINKING_MODES,
    }
}

fn ollama_thinking_modes(model: &str) -> &'static [ModelThinkingMode] {
    let family = ollama_model_family(model);
    if family.starts_with("gpt-oss") {
        return OLLAMA_GPT_OSS_THINKING_MODES;
    }
    if family.starts_with("qwen3")
        || family.starts_with("deepseek-v3.1")
        || family.starts_with("deepseek-r1")
    {
        return OLLAMA_BOOLEAN_THINKING_MODES;
    }
    NONE_ONLY_THINKING_MODES
}

impl ModelProvider {
    pub const ALL: [Self; 7] = [
        Self::Sift,
        Self::Openai,
        Self::Inception,
        Self::Anthropic,
        Self::Google,
        Self::Moonshot,
        Self::Ollama,
    ];

    pub fn all() -> &'static [Self] {
        &Self::ALL
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Sift => "sift",
            Self::Openai => "openai",
            Self::Inception => "inception",
            Self::Anthropic => "anthropic",
            Self::Google => "google",
            Self::Moonshot => "moonshot",
            Self::Ollama => "ollama",
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Sift => "Sift",
            Self::Openai => "OpenAI",
            Self::Inception => "Inception",
            Self::Anthropic => "Anthropic",
            Self::Google => "Google",
            Self::Moonshot => "Moonshot",
            Self::Ollama => "Ollama",
        }
    }

    pub fn from_name(value: &str) -> Option<Self> {
        match value {
            "sift" => Some(Self::Sift),
            "openai" => Some(Self::Openai),
            "inception" => Some(Self::Inception),
            "anthropic" => Some(Self::Anthropic),
            "google" => Some(Self::Google),
            "moonshot" => Some(Self::Moonshot),
            "ollama" => Some(Self::Ollama),
            _ => None,
        }
    }

    pub fn auth_requirement(self) -> ProviderAuthRequirement {
        match self {
            Self::Sift => ProviderAuthRequirement::None,
            Self::Ollama => ProviderAuthRequirement::OptionalApiKey,
            Self::Openai | Self::Inception | Self::Anthropic | Self::Google | Self::Moonshot => {
                ProviderAuthRequirement::RequiredApiKey
            }
        }
    }

    pub fn credential_env_var(self) -> Option<&'static str> {
        match self {
            Self::Sift => None,
            Self::Openai => Some("OPENAI_API_KEY"),
            Self::Inception => Some("INCEPTION_API_KEY"),
            Self::Anthropic => Some("ANTHROPIC_API_KEY"),
            Self::Google => Some("GOOGLE_API_KEY"),
            Self::Moonshot => Some("MOONSHOT_API_KEY"),
            Self::Ollama => Some("OLLAMA_API_KEY"),
        }
    }

    pub fn default_base_url(self) -> Option<&'static str> {
        match self {
            Self::Sift => None,
            Self::Openai => Some("https://api.openai.com"),
            Self::Inception => Some("https://api.inceptionlabs.ai"),
            Self::Anthropic => Some("https://api.anthropic.com"),
            Self::Google => Some("https://generativelanguage.googleapis.com"),
            Self::Moonshot => Some("https://api.moonshot.ai"),
            Self::Ollama => Some("http://localhost:11434"),
        }
    }

    pub fn supports_interactive_login(self) -> bool {
        self.credential_env_var().is_some()
    }

    pub fn normalize_model_alias(self, model: &str) -> String {
        match (self, model) {
            (Self::Moonshot, "kimi-2.6") => "kimi-k2.6".to_string(),
            (Self::Moonshot, "kimi-2.5") => "kimi-k2.5".to_string(),
            _ => model.to_string(),
        }
    }

    pub fn known_model_ids(self) -> &'static [&'static str] {
        match self {
            Self::Sift => crate::infrastructure::adapters::sift_registry::supported_model_ids(),
            Self::Openai => OPENAI_MODELS,
            Self::Inception => INCEPTION_MODELS,
            Self::Anthropic => ANTHROPIC_MODELS,
            Self::Google => GOOGLE_MODELS,
            Self::Moonshot => MOONSHOT_MODELS,
            Self::Ollama => &[],
        }
    }

    pub fn supports_freeform_model_id(self) -> bool {
        matches!(self, Self::Ollama)
    }

    pub fn selectable_model_ids(self) -> &'static [&'static str] {
        match self {
            Self::Moonshot => MOONSHOT_SELECTABLE_MODELS,
            _ => self.known_model_ids(),
        }
    }

    pub fn thinking_modes(self, model: &str) -> &'static [ModelThinkingMode] {
        let normalized_model = self.runtime_model_id(model);
        match self {
            Self::Sift => NONE_ONLY_THINKING_MODES,
            Self::Openai => openai_thinking_modes(&normalized_model),
            Self::Inception => INCEPTION_THINKING_MODES,
            Self::Anthropic => ANTHROPIC_THINKING_MODES,
            Self::Google => GEMINI_THINKING_MODES,
            Self::Moonshot => moonshot_thinking_modes(&normalized_model),
            Self::Ollama => ollama_thinking_modes(&normalized_model),
        }
    }

    pub fn selectable_model_matches_runtime_model(
        self,
        selectable_model: &str,
        runtime_model: &str,
    ) -> bool {
        let normalized_runtime = self.runtime_model_id(runtime_model);
        let normalized_selectable = self.normalize_model_alias(selectable_model);
        normalized_runtime == normalized_selectable
            || self
                .thinking_modes(&normalized_selectable)
                .iter()
                .filter_map(|mode| mode.model_override)
                .any(|model_override| model_override == normalized_runtime)
    }

    pub fn thinking_mode_index_for_runtime_model(
        self,
        selectable_model: &str,
        runtime_model: &str,
        runtime_thinking_mode: Option<&str>,
    ) -> Option<usize> {
        let normalized_runtime = self.runtime_model_id(runtime_model);
        let normalized_runtime_thinking_mode = runtime_thinking_mode
            .map(str::trim)
            .filter(|value| !value.is_empty());
        self.thinking_modes(selectable_model)
            .iter()
            .position(|mode| {
                if let Some(model_override) = mode.model_override {
                    return model_override == normalized_runtime;
                }
                normalized_runtime == self.normalize_model_alias(selectable_model)
                    && match (mode.thinking_mode, normalized_runtime_thinking_mode) {
                        (Some("none"), None) => true,
                        (Some(mode), Some(current)) => mode == current,
                        _ => false,
                    }
            })
    }

    pub fn accepts_model(self, model: &str) -> bool {
        let runtime_model_id = self.runtime_model_id(model);
        self.supports_freeform_model_id()
            || self.known_model_ids().contains(&runtime_model_id.as_str())
    }

    pub fn supports_paddles_http_transport(self, model: &str) -> bool {
        matches!(
            self.capability_surface(model).transport_support,
            ProviderTransportSupport::Supported
        )
    }

    pub fn paddles_http_transport_error(self, model: &str) -> Option<String> {
        match self.capability_surface(model).transport_support {
            ProviderTransportSupport::Supported => None,
            ProviderTransportSupport::Unsupported { reason } => Some(reason),
        }
    }

    pub fn capability_surface(self, model: &str) -> ModelCapabilitySurface {
        let normalized_model = self.runtime_model_id(model);
        let runtime_thinking_mode = self.runtime_model_thinking_mode(model);
        let openai_reasoning_uses_responses = matches!(self, Self::Openai)
            && normalized_model.starts_with("gpt-5")
            && matches!(
                runtime_thinking_mode.as_deref(),
                Some(thinking_mode) if thinking_mode != "none"
            );
        let transport_support = ProviderTransportSupport::Supported;
        let deliberation = match self {
            Self::Sift => DeliberationCapabilitySurface {
                support: DeliberationSupport::Unsupported,
                state_contract: DeliberationStateContract::None,
            },
            Self::Openai
                if OPENAI_RESPONSES_ONLY_MODELS.contains(&normalized_model.as_str())
                    || openai_reasoning_uses_responses =>
            {
                DeliberationCapabilitySurface {
                    support: DeliberationSupport::NativeContinuation,
                    state_contract: DeliberationStateContract::OpaqueRoundTrip,
                }
            }
            Self::Openai if normalized_model.starts_with("gpt-5") => {
                DeliberationCapabilitySurface {
                    support: DeliberationSupport::ToggleOnly,
                    state_contract: DeliberationStateContract::None,
                }
            }
            Self::Openai => DeliberationCapabilitySurface {
                support: DeliberationSupport::Unsupported,
                state_contract: DeliberationStateContract::None,
            },
            Self::Inception => DeliberationCapabilitySurface {
                support: DeliberationSupport::SummaryOnly,
                state_contract: DeliberationStateContract::None,
            },
            Self::Anthropic | Self::Google | Self::Moonshot => DeliberationCapabilitySurface {
                support: DeliberationSupport::NativeContinuation,
                state_contract: DeliberationStateContract::OpaqueRoundTrip,
            },
            Self::Ollama if ollama_supports_thinking(&normalized_model) => {
                DeliberationCapabilitySurface {
                    support: DeliberationSupport::ToggleOnly,
                    state_contract: DeliberationStateContract::None,
                }
            }
            Self::Ollama => DeliberationCapabilitySurface {
                support: DeliberationSupport::Unsupported,
                state_contract: DeliberationStateContract::None,
            },
        };

        let (http_format, render_capability, planner_tool_call) = match self {
            Self::Sift => (
                None,
                RenderCapability::PromptEnvelope,
                PlannerToolCallCapability::PromptEnvelope,
            ),
            Self::Openai if OPENAI_RESPONSES_ONLY_MODELS.contains(&normalized_model.as_str()) => (
                Some(ApiFormat::OpenAi),
                RenderCapability::PromptEnvelope,
                PlannerToolCallCapability::PromptEnvelope,
            ),
            Self::Openai | Self::Inception | Self::Ollama => (
                Some(ApiFormat::OpenAi),
                RenderCapability::OpenAiJsonSchema,
                PlannerToolCallCapability::NativeFunctionTool,
            ),
            Self::Moonshot => (
                Some(ApiFormat::OpenAi),
                RenderCapability::OpenAiJsonSchema,
                PlannerToolCallCapability::StructuredJsonEnvelope,
            ),
            Self::Anthropic => (
                Some(ApiFormat::Anthropic),
                RenderCapability::AnthropicToolUse,
                PlannerToolCallCapability::PromptEnvelope,
            ),
            Self::Google => (
                Some(ApiFormat::Gemini),
                RenderCapability::GeminiJsonSchema,
                PlannerToolCallCapability::StructuredJsonEnvelope,
            ),
        };

        ModelCapabilitySurface {
            http_format,
            render_capability,
            planner_tool_call,
            transport_support,
            deliberation,
        }
    }

    pub fn prepare_runtime_model_id(self, model: &str, thinking_mode: Option<&str>) -> String {
        let normalized_model = self.normalize_model_alias(model);
        let normalized_thinking_mode = thinking_mode
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(thinking_mode) = normalized_thinking_mode
            && self
                .thinking_modes(&normalized_model)
                .iter()
                .any(|mode| mode.thinking_mode == Some(thinking_mode) && mode.runtime_control)
        {
            return format!("{normalized_model}{OPENAI_RUNTIME_THINKING_DELIMITER}{thinking_mode}");
        }
        normalized_model
    }

    pub fn runtime_model_id(self, model: &str) -> String {
        if let Some((model_id, _)) = model.split_once(OPENAI_RUNTIME_THINKING_DELIMITER) {
            return self.normalize_model_alias(model_id);
        }
        self.normalize_model_alias(model)
    }

    pub fn runtime_model_thinking_mode(self, model: &str) -> Option<String> {
        if let Some((_, thinking_mode)) = model.split_once(OPENAI_RUNTIME_THINKING_DELIMITER) {
            let thinking_mode = thinking_mode.trim();
            if !thinking_mode.is_empty() {
                return Some(thinking_mode.to_string());
            }
        }
        None
    }

    pub fn qualified_model_label_with_thinking(
        self,
        model: &str,
        thinking_mode: Option<&str>,
    ) -> String {
        let normalized_model = self.normalize_model_alias(model);
        let normalized_thinking_mode = thinking_mode
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let suffix = normalized_thinking_mode
            .and_then(|mode| {
                self.thinking_modes(&normalized_model)
                    .iter()
                    .find(|candidate| candidate.thinking_mode == Some(mode))
            })
            .map(|mode| format!(" ({})", mode.label))
            .unwrap_or_default();
        format!("{}:{}{}", self.name(), normalized_model, suffix)
    }

    pub fn qualified_model_label(self, model: &str) -> String {
        self.qualified_model_label_with_thinking(
            &self.runtime_model_id(model),
            self.runtime_model_thinking_mode(model).as_deref(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownModel {
    pub provider: ModelProvider,
    pub model_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderCapabilityMatrixRow {
    pub provider: ModelProvider,
    pub model_id: String,
    pub http_format: Option<ApiFormat>,
    pub transport_support: ProviderTransportSupport,
    pub render_capability: RenderCapability,
    pub planner_tool_call: PlannerToolCallCapability,
    pub deliberation_support: DeliberationSupport,
    pub state_contract: DeliberationStateContract,
    pub thinking_modes: Vec<String>,
    pub notes: String,
}

const DOCUMENTED_PROVIDER_CAPABILITY_PATHS: &[(ModelProvider, &str, &str)] = &[
    (
        ModelProvider::Sift,
        "qwen-1.5b",
        "Local native runtime; no provider-native reasoning substrate.",
    ),
    (
        ModelProvider::Openai,
        "gpt-5.5",
        "Chat Completions by default; thinking-enabled GPT-5.5 turns switch planner/schema requests to Responses.",
    ),
    (
        ModelProvider::Openai,
        "gpt-5.5-pro",
        "Responses path with reusable previous_response_id continuity.",
    ),
    (
        ModelProvider::Inception,
        "mercury-2",
        "OpenAI-compatible chat with reasoning summaries but no reusable state.",
    ),
    (
        ModelProvider::Anthropic,
        "claude-sonnet-4-20250514",
        "Messages API with thinking blocks and interleaved-thinking support.",
    ),
    (
        ModelProvider::Google,
        "gemini-2.5-flash",
        "generateContent path with thought-signature continuity.",
    ),
    (
        ModelProvider::Moonshot,
        "kimi-k2.6",
        "OpenAI-compatible chat with reasoning_content continuity.",
    ),
    (
        ModelProvider::Ollama,
        "qwen3",
        "Freeform local models; qwen3 shown for thinking-capable toggle behavior.",
    ),
];

pub fn documented_provider_capability_matrix() -> Vec<ProviderCapabilityMatrixRow> {
    DOCUMENTED_PROVIDER_CAPABILITY_PATHS
        .iter()
        .map(|(provider, model_id, notes)| {
            let surface = provider.capability_surface(model_id);
            ProviderCapabilityMatrixRow {
                provider: *provider,
                model_id: (*model_id).to_string(),
                http_format: surface.http_format,
                transport_support: surface.transport_support,
                render_capability: surface.render_capability,
                planner_tool_call: surface.planner_tool_call,
                deliberation_support: surface.deliberation.support,
                state_contract: surface.deliberation.state_contract,
                thinking_modes: provider
                    .thinking_modes(model_id)
                    .iter()
                    .map(|mode| mode.key.to_string())
                    .collect(),
                notes: (*notes).to_string(),
            }
        })
        .collect()
}

pub fn render_documented_provider_capability_matrix_markdown() -> String {
    let mut lines = vec![
        "| Provider | Model path | Wire | Support | Render | Planner | Deliberation | State | Thinking modes | Notes |"
            .to_string(),
        "|---|---|---|---|---|---|---|---|---|---|".to_string(),
    ];
    for row in documented_provider_capability_matrix() {
        let wire = row.http_format.map(ApiFormat::label).unwrap_or("local");
        let thinking_modes = row
            .thinking_modes
            .iter()
            .map(|mode| format!("`{mode}`"))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | {} | {} |",
            row.provider.name(),
            row.model_id,
            wire,
            row.transport_support.label(),
            row.render_capability.label(),
            row.planner_tool_call.label(),
            row.deliberation_support.label(),
            row.state_contract.label(),
            thinking_modes,
            row.notes,
        ));
    }
    lines.join("\n")
}

pub fn known_state_space_models() -> Vec<KnownModel> {
    let mut models = Vec::new();
    for provider in ModelProvider::all() {
        for model in provider.known_model_ids() {
            models.push(KnownModel {
                provider: *provider,
                model_id: (*model).to_string(),
            });
        }
    }
    models
}

#[cfg(test)]
mod tests {
    use super::{
        ApiFormat, DeliberationStateContract, DeliberationSupport, ModelProvider,
        ModelThinkingMode, PlannerToolCallCapability, ProviderAuthRequirement,
        ProviderTransportSupport, documented_provider_capability_matrix, known_state_space_models,
        render_documented_provider_capability_matrix_markdown,
    };
    use crate::infrastructure::rendering::RenderCapability;
    use std::fs;

    #[test]
    fn moonshot_aliases_are_normalized() {
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-2.5"),
            "kimi-k2.5"
        );
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-2.6"),
            "kimi-k2.6"
        );
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-k2.5"),
            "kimi-k2.5"
        );
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-k2.6"),
            "kimi-k2.6"
        );
    }

    #[test]
    fn moonshot_provider_exposes_current_kimi_model_ids() {
        assert_eq!(
            ModelProvider::Moonshot.known_model_ids(),
            [
                "kimi-k2.6",
                "kimi-k2.5",
                "kimi-k2",
                "kimi-k2-0905-preview",
                "kimi-k2-0711-preview",
                "kimi-k2-turbo-preview",
                "kimi-k2-thinking",
                "kimi-k2-thinking-turbo",
            ]
        );
    }

    #[test]
    fn moonshot_selectable_models_group_thinking_variants_under_kimi_k2() {
        assert_eq!(
            ModelProvider::Moonshot.selectable_model_ids(),
            [
                "kimi-k2.6",
                "kimi-k2.5",
                "kimi-k2",
                "kimi-k2-0905-preview",
                "kimi-k2-0711-preview",
            ]
        );
        assert_eq!(
            ModelProvider::Moonshot.thinking_modes("kimi-k2"),
            [
                ModelThinkingMode {
                    key: "default",
                    label: "Default",
                    model_override: Some("kimi-k2"),
                    thinking_mode: None,
                    runtime_control: false,
                },
                ModelThinkingMode {
                    key: "turbo-preview",
                    label: "Turbo preview",
                    model_override: Some("kimi-k2-turbo-preview"),
                    thinking_mode: None,
                    runtime_control: false,
                },
                ModelThinkingMode {
                    key: "thinking",
                    label: "Thinking",
                    model_override: Some("kimi-k2-thinking"),
                    thinking_mode: None,
                    runtime_control: false,
                },
                ModelThinkingMode {
                    key: "thinking-turbo",
                    label: "Thinking turbo",
                    model_override: Some("kimi-k2-thinking-turbo"),
                    thinking_mode: None,
                    runtime_control: false,
                },
            ]
        );
        assert!(
            ModelProvider::Moonshot
                .selectable_model_matches_runtime_model("kimi-k2", "kimi-k2-thinking")
        );
        assert_eq!(
            ModelProvider::Moonshot.thinking_mode_index_for_runtime_model(
                "kimi-k2",
                "kimi-k2-thinking",
                None
            ),
            Some(2)
        );
    }

    #[test]
    fn openai_gpt_5_5_models_expose_parameterized_thinking_modes() {
        assert_eq!(
            ModelProvider::Openai.thinking_modes("gpt-5.5"),
            [
                ModelThinkingMode {
                    key: "none",
                    label: "None",
                    model_override: None,
                    thinking_mode: Some("none"),
                    runtime_control: true,
                },
                ModelThinkingMode {
                    key: "low",
                    label: "Low",
                    model_override: None,
                    thinking_mode: Some("low"),
                    runtime_control: true,
                },
                ModelThinkingMode {
                    key: "medium",
                    label: "Medium",
                    model_override: None,
                    thinking_mode: Some("medium"),
                    runtime_control: true,
                },
                ModelThinkingMode {
                    key: "high",
                    label: "High",
                    model_override: None,
                    thinking_mode: Some("high"),
                    runtime_control: true,
                },
                ModelThinkingMode {
                    key: "xhigh",
                    label: "XHigh",
                    model_override: None,
                    thinking_mode: Some("xhigh"),
                    runtime_control: true,
                },
            ]
        );
        assert_eq!(
            ModelProvider::Openai.thinking_mode_index_for_runtime_model(
                "gpt-5.5",
                "gpt-5.5",
                Some("xhigh")
            ),
            Some(4)
        );
        assert_eq!(
            ModelProvider::Openai.prepare_runtime_model_id("gpt-5.5", Some("xhigh")),
            "gpt-5.5@@thinking=xhigh"
        );
        assert_eq!(
            ModelProvider::Openai.qualified_model_label("gpt-5.5@@thinking=xhigh"),
            "openai:gpt-5.5 (XHigh)"
        );
    }

    #[test]
    fn openai_pro_models_expose_only_documented_thinking_controls() {
        let thinking_mode_keys = |model: &str| {
            ModelProvider::Openai
                .thinking_modes(model)
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>()
        };

        assert_eq!(thinking_mode_keys("gpt-5.5-pro"), ["none"]);
        assert_eq!(
            thinking_mode_keys("gpt-5.4-pro"),
            ["medium", "high", "xhigh"]
        );
        assert_eq!(
            thinking_mode_keys("gpt-5.2-pro"),
            ["medium", "high", "xhigh"]
        );
        assert_eq!(thinking_mode_keys("gpt-5-pro"), ["high"]);
        assert_eq!(thinking_mode_keys("o3-pro"), ["none"]);
        assert_eq!(thinking_mode_keys("o1-pro"), ["none"]);
    }

    #[test]
    fn provider_catalog_exposes_thinking_modes_across_supported_providers() {
        assert_eq!(
            ModelProvider::Sift.thinking_modes("qwen-1.5b"),
            [ModelThinkingMode {
                key: "none",
                label: "None",
                model_override: None,
                thinking_mode: Some("none"),
                runtime_control: false,
            }]
        );
        assert_eq!(
            ModelProvider::Inception
                .thinking_modes("mercury-2")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["instant", "low", "medium", "high"]
        );
        assert_eq!(
            ModelProvider::Anthropic
                .thinking_modes("claude-sonnet-4-20250514")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["none", "low", "medium", "high"]
        );
        assert_eq!(
            ModelProvider::Google
                .thinking_modes("gemini-2.5-flash")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["none", "low", "medium", "high"]
        );
        assert_eq!(
            ModelProvider::Moonshot
                .thinking_modes("kimi-k2.6")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["none", "thinking"]
        );
        assert_eq!(
            ModelProvider::Ollama
                .thinking_modes("qwen3")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["none", "thinking"]
        );
        assert_eq!(
            ModelProvider::Ollama
                .thinking_modes("gpt-oss:20b")
                .iter()
                .map(|mode| mode.key)
                .collect::<Vec<_>>(),
            ["low", "medium", "high"]
        );
    }

    #[test]
    fn runtime_model_ids_preserve_provider_specific_thinking_modes_when_needed() {
        assert_eq!(
            ModelProvider::Openai.prepare_runtime_model_id("gpt-5.4", Some("high")),
            "gpt-5.4@@thinking=high"
        );
        assert_eq!(
            ModelProvider::Inception.prepare_runtime_model_id("mercury-2", Some("instant")),
            "mercury-2@@thinking=instant"
        );
        assert_eq!(
            ModelProvider::Anthropic
                .prepare_runtime_model_id("claude-sonnet-4-20250514", Some("none")),
            "claude-sonnet-4-20250514@@thinking=none"
        );
        assert_eq!(
            ModelProvider::Google.prepare_runtime_model_id("gemini-2.5-flash", Some("high")),
            "gemini-2.5-flash@@thinking=high"
        );
        assert_eq!(
            ModelProvider::Moonshot.prepare_runtime_model_id("kimi-k2.6", Some("enabled")),
            "kimi-k2.6@@thinking=enabled"
        );
        assert_eq!(
            ModelProvider::Ollama.prepare_runtime_model_id("qwen3", Some("thinking")),
            "qwen3@@thinking=thinking"
        );
        assert_eq!(
            ModelProvider::Sift.prepare_runtime_model_id("qwen-1.5b", Some("none")),
            "qwen-1.5b"
        );
        assert_eq!(
            ModelProvider::Openai.prepare_runtime_model_id("gpt-4o", Some("none")),
            "gpt-4o"
        );
    }

    #[test]
    fn auth_requirements_distinguish_local_optional_and_required_providers() {
        assert_eq!(
            ModelProvider::Sift.auth_requirement(),
            ProviderAuthRequirement::None
        );
        assert_eq!(
            ModelProvider::Ollama.auth_requirement(),
            ProviderAuthRequirement::OptionalApiKey
        );
        assert_eq!(
            ModelProvider::Openai.auth_requirement(),
            ProviderAuthRequirement::RequiredApiKey
        );
        assert_eq!(
            ModelProvider::Inception.auth_requirement(),
            ProviderAuthRequirement::RequiredApiKey
        );
    }

    #[test]
    fn openai_provider_exposes_additional_model_ids() {
        assert_eq!(
            ModelProvider::Openai.known_model_ids(),
            [
                "gpt-4.1-mini",
                "gpt-4.1",
                "gpt-4o",
                "gpt-4o-mini",
                "gpt-5.5",
                "gpt-5.5-pro",
                "gpt-5.4",
                "gpt-5.4-pro",
                "gpt-5.4-mini",
                "gpt-5.4-nano",
                "gpt-5.3-chat-latest",
                "gpt-5.3-codex",
                "gpt-5.2",
                "gpt-5.2-pro",
                "gpt-5.2-chat-latest",
                "gpt-5.2-codex",
                "gpt-5.1",
                "gpt-5.1-chat-latest",
                "gpt-5.1-codex",
                "gpt-5.1-codex-max",
                "gpt-5.1-codex-mini",
                "gpt-5",
                "gpt-5-pro",
                "gpt-5-mini",
                "gpt-5-nano",
                "gpt-5-chat-latest",
                "gpt-5-codex",
                "o3-pro",
                "o1-pro",
            ]
        );
    }

    #[test]
    fn openai_transport_supports_responses_only_pro_models() {
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.5-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.4-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.2-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("o3-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("o1-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.5"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.4"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-4o"));

        for model in [
            "gpt-5.5-pro",
            "gpt-5.4-pro",
            "gpt-5.2-pro",
            "gpt-5-pro",
            "o3-pro",
            "o1-pro",
        ] {
            let surface = ModelProvider::Openai.capability_surface(model);
            assert_eq!(
                surface.render_capability,
                RenderCapability::PromptEnvelope,
                "{model} render path"
            );
            assert_eq!(
                surface.planner_tool_call,
                PlannerToolCallCapability::PromptEnvelope,
                "{model} planner path"
            );
            assert_eq!(
                surface.deliberation.support,
                DeliberationSupport::NativeContinuation,
                "{model} deliberation support"
            );
            assert_eq!(
                surface.deliberation.state_contract,
                DeliberationStateContract::OpaqueRoundTrip,
                "{model} state contract"
            );
        }
    }

    #[test]
    fn openai_transport_error_is_absent_for_responses_models() {
        assert!(
            ModelProvider::Openai
                .paddles_http_transport_error("gpt-5.4-pro")
                .is_none()
        );
    }

    #[test]
    fn capability_surface_negotiates_shared_http_render_and_tool_call_behavior() {
        let openai = ModelProvider::Openai.capability_surface("gpt-5.5");
        assert_eq!(openai.http_format, Some(ApiFormat::OpenAi));
        assert_eq!(openai.render_capability, RenderCapability::OpenAiJsonSchema);
        assert_eq!(
            openai.planner_tool_call,
            PlannerToolCallCapability::NativeFunctionTool
        );
        assert!(matches!(
            openai.transport_support,
            ProviderTransportSupport::Supported
        ));

        let openai_responses = ModelProvider::Openai.capability_surface("gpt-5.5-pro");
        assert_eq!(openai_responses.http_format, Some(ApiFormat::OpenAi));
        assert_eq!(
            openai_responses.render_capability,
            RenderCapability::PromptEnvelope
        );
        assert_eq!(
            openai_responses.planner_tool_call,
            PlannerToolCallCapability::PromptEnvelope
        );
        assert!(matches!(
            openai_responses.transport_support,
            ProviderTransportSupport::Supported
        ));

        let gemini = ModelProvider::Google.capability_surface("gemini-2.5-flash");
        assert_eq!(gemini.http_format, Some(ApiFormat::Gemini));
        assert_eq!(gemini.render_capability, RenderCapability::GeminiJsonSchema);
        assert_eq!(
            gemini.planner_tool_call,
            PlannerToolCallCapability::StructuredJsonEnvelope
        );

        let moonshot = ModelProvider::Moonshot.capability_surface("kimi-k2.6");
        assert_eq!(moonshot.http_format, Some(ApiFormat::OpenAi));
        assert_eq!(
            moonshot.render_capability,
            RenderCapability::OpenAiJsonSchema
        );
        assert_eq!(
            moonshot.planner_tool_call,
            PlannerToolCallCapability::StructuredJsonEnvelope
        );

        let sift = ModelProvider::Sift.capability_surface("qwen-1.5b");
        assert_eq!(sift.http_format, None);
        assert_eq!(sift.render_capability, RenderCapability::PromptEnvelope);
        assert_eq!(
            sift.planner_tool_call,
            PlannerToolCallCapability::PromptEnvelope
        );
    }

    #[test]
    fn capability_surface_classifies_deliberation_support_for_runtime_provider_paths() {
        let sift = ModelProvider::Sift.capability_surface("qwen-1.5b");
        assert_eq!(sift.deliberation.support, DeliberationSupport::Unsupported);
        assert_eq!(
            sift.deliberation.state_contract,
            DeliberationStateContract::None
        );

        let openai = ModelProvider::Openai.capability_surface("gpt-5.5");
        assert_eq!(openai.deliberation.support, DeliberationSupport::ToggleOnly);
        assert_eq!(
            openai.deliberation.state_contract,
            DeliberationStateContract::None
        );

        let openai_reasoning = ModelProvider::Openai.capability_surface("gpt-5.5@@thinking=xhigh");
        assert_eq!(
            openai_reasoning.deliberation.support,
            DeliberationSupport::NativeContinuation
        );
        assert_eq!(
            openai_reasoning.deliberation.state_contract,
            DeliberationStateContract::OpaqueRoundTrip
        );

        let openai_responses = ModelProvider::Openai.capability_surface("gpt-5.5-pro");
        assert_eq!(
            openai_responses.deliberation.support,
            DeliberationSupport::NativeContinuation
        );
        assert_eq!(
            openai_responses.deliberation.state_contract,
            DeliberationStateContract::OpaqueRoundTrip
        );

        let inception = ModelProvider::Inception.capability_surface("mercury-2");
        assert_eq!(
            inception.deliberation.support,
            DeliberationSupport::SummaryOnly
        );
        assert_eq!(
            inception.deliberation.state_contract,
            DeliberationStateContract::None
        );

        let anthropic = ModelProvider::Anthropic.capability_surface("claude-sonnet-4-20250514");
        assert_eq!(
            anthropic.deliberation.support,
            DeliberationSupport::NativeContinuation
        );
        assert_eq!(
            anthropic.deliberation.state_contract,
            DeliberationStateContract::OpaqueRoundTrip
        );

        let google = ModelProvider::Google.capability_surface("gemini-2.5-flash");
        assert_eq!(
            google.deliberation.support,
            DeliberationSupport::NativeContinuation
        );
        assert_eq!(
            google.deliberation.state_contract,
            DeliberationStateContract::OpaqueRoundTrip
        );

        let moonshot = ModelProvider::Moonshot.capability_surface("kimi-k2.6");
        assert_eq!(
            moonshot.deliberation.support,
            DeliberationSupport::NativeContinuation
        );
        assert_eq!(
            moonshot.deliberation.state_contract,
            DeliberationStateContract::OpaqueRoundTrip
        );

        let ollama = ModelProvider::Ollama.capability_surface("qwen3");
        assert_eq!(ollama.deliberation.support, DeliberationSupport::ToggleOnly);
        assert_eq!(
            ollama.deliberation.state_contract,
            DeliberationStateContract::None
        );

        let ollama_without_thinking = ModelProvider::Ollama.capability_surface("llama3.2");
        assert_eq!(
            ollama_without_thinking.deliberation.support,
            DeliberationSupport::Unsupported
        );
        assert_eq!(
            ollama_without_thinking.deliberation.state_contract,
            DeliberationStateContract::None
        );
    }

    #[test]
    fn ollama_deliberation_support_tracks_thinking_family_through_tags_and_namespaces() {
        let namespaced_qwen = ModelProvider::Ollama.capability_surface("library/qwen3:8b");
        assert_eq!(
            namespaced_qwen.deliberation.support,
            DeliberationSupport::ToggleOnly
        );
        assert_eq!(
            namespaced_qwen.deliberation.state_contract,
            DeliberationStateContract::None
        );

        let tagged_llama = ModelProvider::Ollama.capability_surface("llama3.2:latest");
        assert_eq!(
            tagged_llama.deliberation.support,
            DeliberationSupport::Unsupported
        );
        assert_eq!(
            tagged_llama.deliberation.state_contract,
            DeliberationStateContract::None
        );
    }

    #[test]
    fn provider_capability_matrix_covers_documented_provider_paths() {
        let rows = documented_provider_capability_matrix();
        let summarized = rows
            .iter()
            .map(|row| {
                (
                    row.provider,
                    row.model_id.as_str(),
                    row.http_format,
                    row.render_capability,
                    row.planner_tool_call,
                    &row.transport_support,
                    row.deliberation_support,
                    row.state_contract,
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            summarized,
            vec![
                (
                    ModelProvider::Sift,
                    "qwen-1.5b",
                    None,
                    RenderCapability::PromptEnvelope,
                    PlannerToolCallCapability::PromptEnvelope,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::Unsupported,
                    DeliberationStateContract::None,
                ),
                (
                    ModelProvider::Openai,
                    "gpt-5.5",
                    Some(ApiFormat::OpenAi),
                    RenderCapability::OpenAiJsonSchema,
                    PlannerToolCallCapability::NativeFunctionTool,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::ToggleOnly,
                    DeliberationStateContract::None,
                ),
                (
                    ModelProvider::Openai,
                    "gpt-5.5-pro",
                    Some(ApiFormat::OpenAi),
                    RenderCapability::PromptEnvelope,
                    PlannerToolCallCapability::PromptEnvelope,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::NativeContinuation,
                    DeliberationStateContract::OpaqueRoundTrip,
                ),
                (
                    ModelProvider::Inception,
                    "mercury-2",
                    Some(ApiFormat::OpenAi),
                    RenderCapability::OpenAiJsonSchema,
                    PlannerToolCallCapability::NativeFunctionTool,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::SummaryOnly,
                    DeliberationStateContract::None,
                ),
                (
                    ModelProvider::Anthropic,
                    "claude-sonnet-4-20250514",
                    Some(ApiFormat::Anthropic),
                    RenderCapability::AnthropicToolUse,
                    PlannerToolCallCapability::PromptEnvelope,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::NativeContinuation,
                    DeliberationStateContract::OpaqueRoundTrip,
                ),
                (
                    ModelProvider::Google,
                    "gemini-2.5-flash",
                    Some(ApiFormat::Gemini),
                    RenderCapability::GeminiJsonSchema,
                    PlannerToolCallCapability::StructuredJsonEnvelope,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::NativeContinuation,
                    DeliberationStateContract::OpaqueRoundTrip,
                ),
                (
                    ModelProvider::Moonshot,
                    "kimi-k2.6",
                    Some(ApiFormat::OpenAi),
                    RenderCapability::OpenAiJsonSchema,
                    PlannerToolCallCapability::StructuredJsonEnvelope,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::NativeContinuation,
                    DeliberationStateContract::OpaqueRoundTrip,
                ),
                (
                    ModelProvider::Ollama,
                    "qwen3",
                    Some(ApiFormat::OpenAi),
                    RenderCapability::OpenAiJsonSchema,
                    PlannerToolCallCapability::NativeFunctionTool,
                    &ProviderTransportSupport::Supported,
                    DeliberationSupport::ToggleOnly,
                    DeliberationStateContract::None,
                ),
            ]
        );
    }

    #[test]
    fn configuration_docs_embed_current_provider_capability_matrix() {
        let config = fs::read_to_string(format!("{}/CONFIGURATION.md", env!("CARGO_MANIFEST_DIR")))
            .expect("read configuration docs");
        let rendered = render_documented_provider_capability_matrix_markdown();
        let actual = extract_marked_section(
            &config,
            "<!-- BEGIN_PROVIDER_CAPABILITY_MATRIX -->",
            "<!-- END_PROVIDER_CAPABILITY_MATRIX -->",
        )
        .expect("capability matrix markers");

        assert_eq!(actual.trim(), rendered.trim());
    }

    #[test]
    fn inception_provider_metadata_is_registered() {
        assert_eq!(ModelProvider::Inception.name(), "inception");
        assert_eq!(ModelProvider::Inception.display_name(), "Inception");
        assert_eq!(
            ModelProvider::Inception.credential_env_var(),
            Some("INCEPTION_API_KEY")
        );
        assert_eq!(
            ModelProvider::Inception.default_base_url(),
            Some("https://api.inceptionlabs.ai")
        );
        assert_eq!(ModelProvider::Inception.known_model_ids(), ["mercury-2"]);
        assert_eq!(
            ModelProvider::Inception.qualified_model_label("mercury-2"),
            "inception:mercury-2"
        );
    }

    #[test]
    fn known_state_space_models_include_remote_and_local_catalog_entries() {
        let models = known_state_space_models();
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Sift && model.model_id == "qwen-1.5b"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Sift && model.model_id == "bonsai-8b"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Openai && model.model_id == "gpt-4o"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Moonshot && model.model_id == "kimi-k2.6"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Moonshot && model.model_id == "kimi-k2.5"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Moonshot && model.model_id == "kimi-k2"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Moonshot && model.model_id == "kimi-k2-thinking-turbo"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Inception && model.model_id == "mercury-2"
        }));
    }

    fn extract_marked_section<'a>(
        content: &'a str,
        start_marker: &str,
        end_marker: &str,
    ) -> Option<&'a str> {
        let start = content.find(start_marker)?;
        let remainder = &content[start + start_marker.len()..];
        let end = remainder.find(end_marker)?;
        Some(remainder[..end].trim())
    }
}
