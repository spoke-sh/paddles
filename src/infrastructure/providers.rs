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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlannerToolCallCapability {
    NativeFunctionTool,
    StructuredJsonEnvelope,
    PromptEnvelope,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProviderTransportSupport {
    Supported,
    Unsupported { reason: String },
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
}

const OPENAI_MODELS: &[&str] = &[
    "gpt-4.1-mini",
    "gpt-4.1",
    "gpt-4o",
    "gpt-4o-mini",
    "gpt-5.4",
    "gpt-5.4-mini",
    "gpt-5.4-nano",
    "gpt-5.3-chat-latest",
    "gpt-5.3-codex",
    "gpt-5.2",
    "gpt-5.2-chat-latest",
    "gpt-5.2-codex",
    "gpt-5.1",
    "gpt-5.1-chat-latest",
    "gpt-5.1-codex",
    "gpt-5.1-codex-max",
    "gpt-5.1-codex-mini",
    "gpt-5",
    "gpt-5-mini",
    "gpt-5-nano",
    "gpt-5-chat-latest",
    "gpt-5-codex",
];
const OPENAI_RESPONSES_ONLY_MODELS: &[&str] = &["gpt-5.4-pro", "gpt-5-pro", "gpt-5.2-pro"];
const INCEPTION_MODELS: &[&str] = &["mercury-2"];
const ANTHROPIC_MODELS: &[&str] = &["claude-sonnet-4-20250514"];
const GOOGLE_MODELS: &[&str] = &["gemini-2.5-flash"];
const MOONSHOT_MODELS: &[&str] = &["kimi-k2.5"];

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

    pub fn accepts_model(self, model: &str) -> bool {
        self.supports_freeform_model_id() || self.known_model_ids().contains(&model)
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
        let normalized_model = self.normalize_model_alias(model);
        let transport_support = match self {
            Self::Openai if OPENAI_RESPONSES_ONLY_MODELS.contains(&normalized_model.as_str()) => {
                ProviderTransportSupport::Unsupported {
                    reason: format!(
                        "Model `{}` is Responses API only and is not supported by Paddles' current OpenAI transport. Paddles currently uses Chat Completions with structured JSON/tool calls. Choose `openai:gpt-5.4`, `openai:gpt-5.4-mini`, or `openai:gpt-4o` instead.",
                        self.qualified_model_label(&normalized_model)
                    ),
                }
            }
            _ => ProviderTransportSupport::Supported,
        };

        let (http_format, render_capability, planner_tool_call) = match self {
            Self::Sift => (
                None,
                RenderCapability::PromptEnvelope,
                PlannerToolCallCapability::PromptEnvelope,
            ),
            Self::Openai | Self::Inception | Self::Moonshot | Self::Ollama => (
                Some(ApiFormat::OpenAi),
                RenderCapability::OpenAiJsonSchema,
                PlannerToolCallCapability::NativeFunctionTool,
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
        }
    }

    pub fn qualified_model_label(self, model: &str) -> String {
        format!("{}:{}", self.name(), self.normalize_model_alias(model))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownModel {
    pub provider: ModelProvider,
    pub model_id: String,
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
        ApiFormat, ModelProvider, PlannerToolCallCapability, ProviderAuthRequirement,
        ProviderTransportSupport, known_state_space_models,
    };
    use crate::infrastructure::rendering::RenderCapability;

    #[test]
    fn moonshot_aliases_are_normalized() {
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-2.5"),
            "kimi-k2.5"
        );
        assert_eq!(
            ModelProvider::Moonshot.normalize_model_alias("kimi-k2.5"),
            "kimi-k2.5"
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
                "gpt-5.4",
                "gpt-5.4-mini",
                "gpt-5.4-nano",
                "gpt-5.3-chat-latest",
                "gpt-5.3-codex",
                "gpt-5.2",
                "gpt-5.2-chat-latest",
                "gpt-5.2-codex",
                "gpt-5.1",
                "gpt-5.1-chat-latest",
                "gpt-5.1-codex",
                "gpt-5.1-codex-max",
                "gpt-5.1-codex-mini",
                "gpt-5",
                "gpt-5-mini",
                "gpt-5-nano",
                "gpt-5-chat-latest",
                "gpt-5-codex",
            ]
        );
    }

    #[test]
    fn openai_transport_rejects_responses_only_pro_models() {
        assert!(!ModelProvider::Openai.supports_paddles_http_transport("gpt-5.4-pro"));
        assert!(!ModelProvider::Openai.supports_paddles_http_transport("gpt-5-pro"));
        assert!(!ModelProvider::Openai.supports_paddles_http_transport("gpt-5.2-pro"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-5.4"));
        assert!(ModelProvider::Openai.supports_paddles_http_transport("gpt-4o"));
    }

    #[test]
    fn openai_transport_error_mentions_supported_replacements() {
        let error = ModelProvider::Openai
            .paddles_http_transport_error("gpt-5.4-pro")
            .expect("unsupported model should return an error");

        assert!(error.contains("Responses API only"));
        assert!(error.contains("openai:gpt-5.4"));
        assert!(error.contains("openai:gpt-4o"));
    }

    #[test]
    fn capability_surface_negotiates_shared_http_render_and_tool_call_behavior() {
        let openai = ModelProvider::Openai.capability_surface("gpt-5.4");
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

        let gemini = ModelProvider::Google.capability_surface("gemini-2.5-flash");
        assert_eq!(gemini.http_format, Some(ApiFormat::Gemini));
        assert_eq!(gemini.render_capability, RenderCapability::GeminiJsonSchema);
        assert_eq!(
            gemini.planner_tool_call,
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
            model.provider == ModelProvider::Moonshot && model.model_id == "kimi-k2.5"
        }));
        assert!(models.iter().any(|model| {
            model.provider == ModelProvider::Inception && model.model_id == "mercury-2"
        }));
    }
}
