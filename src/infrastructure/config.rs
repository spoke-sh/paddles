use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::infrastructure::providers::ModelProvider;
use crate::infrastructure::runtime_preferences::RuntimeLanePreferences;

const CONFIG_FILE_NAME: &str = "paddles.toml";
const USER_CONFIG_RELATIVE_PATH: &str = ".config/paddles/paddles.toml";
const SYSTEM_CONFIG_PATH: &str = "/etc/paddles/paddles.toml";

/// Paddles configuration loaded from paddles.toml.
///
/// Layering order:
/// `/etc/paddles/paddles.toml` < `~/.config/paddles/paddles.toml` <
/// `./paddles.toml` < `~/.local/state/paddles/runtime-lanes.toml`.
/// CLI flags override all persisted values.
#[derive(Debug, Clone, PartialEq)]
pub struct PaddlesConfig {
    pub provider: String,
    pub provider_url: Option<String>,
    pub model: String,
    pub synthesizer_provider: Option<String>,
    pub synthesizer_model: Option<String>,
    pub planner_model: Option<String>,
    pub planner_provider: Option<String>,
    pub gatherer_model: Option<String>,
    pub gatherer_provider: String,
    pub port: u16,
    pub verbose: u8,
    pub hf_token: Option<String>,
    pub context1_harness_ready: bool,
    pub credits: u64,
    pub weights: f64,
    pub biases: f64,
    pub reality_mode: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct ModelLaneOverlay {
    provider: Option<String>,
    model: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
struct PaddlesConfigOverlay {
    provider: Option<String>,
    provider_url: Option<String>,
    model: Option<String>,
    shared: Option<ModelLaneOverlay>,
    synthesizer: Option<ModelLaneOverlay>,
    planner: Option<ModelLaneOverlay>,
    planner_model: Option<String>,
    planner_provider: Option<String>,
    gatherer_model: Option<String>,
    gatherer_provider: Option<String>,
    port: Option<u16>,
    verbose: Option<u8>,
    hf_token: Option<String>,
    context1_harness_ready: Option<bool>,
    credits: Option<u64>,
    weights: Option<f64>,
    biases: Option<f64>,
    reality_mode: Option<bool>,
}

impl Default for PaddlesConfig {
    fn default() -> Self {
        Self {
            provider: "sift".to_string(),
            provider_url: None,
            model: "qwen-1.5b".to_string(),
            synthesizer_provider: None,
            synthesizer_model: None,
            planner_model: None,
            planner_provider: None,
            gatherer_model: None,
            gatherer_provider: "sift-direct".to_string(),
            port: 3000,
            verbose: 0,
            hf_token: None,
            context1_harness_ready: false,
            credits: 0,
            weights: 0.5,
            biases: 0.0,
            reality_mode: false,
        }
    }
}

impl PaddlesConfig {
    /// Load layered authored configuration without machine-managed runtime preferences.
    pub fn load(workspace_root: &Path) -> Self {
        Self::load_with_runtime_preferences(workspace_root, None)
    }

    /// Load layered configuration with optional machine-managed runtime lane preferences.
    pub fn load_with_runtime_preferences(
        workspace_root: &Path,
        runtime_preferences: Option<&RuntimeLanePreferences>,
    ) -> Self {
        let user_config = user_config_path();
        let system_config = PathBuf::from(SYSTEM_CONFIG_PATH);
        let workspace_config = workspace_root.join(CONFIG_FILE_NAME);
        Self::load_from_explicit_paths(
            Some(workspace_config.as_path()),
            user_config.as_deref(),
            Some(system_config.as_path()),
            runtime_preferences,
        )
    }

    /// Highest-precedence authored config path, if any.
    pub fn find_config_path(workspace_root: &Path) -> Option<PathBuf> {
        authored_config_search_paths(workspace_root)
            .into_iter()
            .find(|p| p.exists())
    }

    /// Returns whether any authored config layer explicitly sets `port`.
    pub fn authored_port_is_configured(workspace_root: &Path) -> bool {
        let user_config = user_config_path();
        let system_config = PathBuf::from(SYSTEM_CONFIG_PATH);
        let workspace_config = workspace_root.join(CONFIG_FILE_NAME);
        authored_port_is_configured_in_explicit_paths(
            Some(workspace_config.as_path()),
            user_config.as_deref(),
            Some(system_config.as_path()),
        )
    }

    fn load_from_explicit_paths(
        workspace_config: Option<&Path>,
        user_config: Option<&Path>,
        system_config: Option<&Path>,
        runtime_preferences: Option<&RuntimeLanePreferences>,
    ) -> Self {
        let mut config = Self::default();

        for path in [system_config, user_config, workspace_config]
            .into_iter()
            .flatten()
        {
            if let Some(overlay) = parse_config_overlay(path) {
                config.apply_overlay(overlay);
            }
        }
        if let Some(runtime_preferences) = runtime_preferences {
            config.apply_runtime_preferences(runtime_preferences);
        }

        config
    }

    fn apply_overlay(&mut self, overlay: PaddlesConfigOverlay) {
        if let Some(provider) = overlay.provider.filter(|value| !value.trim().is_empty()) {
            self.provider = provider;
        }
        if let Some(provider_url) = overlay.provider_url {
            self.provider_url = normalize_optional_string(provider_url);
        }
        if let Some(model) = overlay.model.filter(|value| !value.trim().is_empty()) {
            self.model = model;
        }
        if let Some(shared) = overlay.shared {
            if let Some(provider) = shared.provider.filter(|value| !value.trim().is_empty()) {
                self.provider = provider;
            }
            if let Some(model) = shared.model.filter(|value| !value.trim().is_empty()) {
                self.model = model;
            }
        }
        if let Some(synthesizer) = overlay.synthesizer {
            if let Some(provider) = synthesizer.provider {
                self.synthesizer_provider = normalize_optional_string(provider);
            }
            if let Some(model) = synthesizer.model {
                self.synthesizer_model = normalize_optional_string(model);
            }
        }
        if let Some(planner_model) = overlay.planner_model {
            self.planner_model = normalize_optional_string(planner_model);
        }
        if let Some(planner_provider) = overlay.planner_provider {
            self.planner_provider = normalize_optional_string(planner_provider);
        }
        if let Some(planner) = overlay.planner {
            if let Some(provider) = planner.provider {
                self.planner_provider = normalize_optional_string(provider);
            }
            if let Some(model) = planner.model {
                self.planner_model = normalize_optional_string(model);
            }
        }
        if let Some(gatherer_model) = overlay.gatherer_model {
            self.gatherer_model = normalize_optional_string(gatherer_model);
        }
        if let Some(gatherer_provider) = overlay
            .gatherer_provider
            .filter(|value| !value.trim().is_empty())
        {
            self.gatherer_provider = gatherer_provider;
        }
        if let Some(port) = overlay.port {
            self.port = port;
        }
        if let Some(verbose) = overlay.verbose {
            self.verbose = verbose;
        }
        if let Some(hf_token) = overlay.hf_token {
            self.hf_token = normalize_optional_string(hf_token);
        }
        if let Some(context1_harness_ready) = overlay.context1_harness_ready {
            self.context1_harness_ready = context1_harness_ready;
        }
        if let Some(credits) = overlay.credits {
            self.credits = credits;
        }
        if let Some(weights) = overlay.weights {
            self.weights = weights;
        }
        if let Some(biases) = overlay.biases {
            self.biases = biases;
        }
        if let Some(reality_mode) = overlay.reality_mode {
            self.reality_mode = reality_mode;
        }
    }

    fn apply_runtime_preferences(&mut self, preferences: &RuntimeLanePreferences) {
        if let Some(provider) = preferences
            .provider
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            self.provider = provider.clone();
        }
        if let Some(model) = preferences
            .model
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            self.model = model.clone();
        }
        self.synthesizer_provider = None;
        self.synthesizer_model = None;
        self.planner_provider = None;
        self.planner_model = None;
    }
}

/// Normalizes provider-specific model aliases so legacy configs keep working.
pub fn normalize_provider_model_alias(provider: &str, model: &str) -> String {
    ModelProvider::from_name(provider)
        .map(|provider| provider.normalize_model_alias(model))
        .unwrap_or_else(|| model.to_string())
}

/// Returns the configured gatherer provider without legacy alias remapping.
pub fn normalize_gatherer_provider_alias(provider: &str) -> String {
    provider.to_string()
}

/// Resolve the requested web server port before binding.
///
/// CLI flags win over everything. When no authored config layer explicitly sets
/// `port`, the default startup behavior is to request an ephemeral port from
/// the OS.
pub fn resolve_web_server_port(
    cli_port: Option<u16>,
    config_port: u16,
    authored_port_configured: bool,
) -> u16 {
    cli_port.unwrap_or({
        if authored_port_configured {
            config_port
        } else {
            0
        }
    })
}

/// Resolve the effective runtime verbosity from CLI and authored config.
///
/// CLI flags win over persisted config. When the CLI does not set `-v`, the
/// layered config value applies to every interactive surface.
pub fn resolve_runtime_verbosity(cli_verbose: u8, config_verbose: u8) -> u8 {
    if cli_verbose > 0 {
        cli_verbose
    } else {
        config_verbose
    }
}

fn authored_port_is_configured_in_explicit_paths(
    workspace_config: Option<&Path>,
    user_config: Option<&Path>,
    system_config: Option<&Path>,
) -> bool {
    [system_config, user_config, workspace_config]
        .into_iter()
        .flatten()
        .filter_map(parse_config_overlay)
        .any(|overlay| overlay.port.is_some())
}

fn authored_config_search_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![workspace_root.join(CONFIG_FILE_NAME)];
    if let Some(user_config) = user_config_path() {
        paths.push(user_config);
    }
    paths.push(PathBuf::from(SYSTEM_CONFIG_PATH));
    paths
}

fn user_config_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join(USER_CONFIG_RELATIVE_PATH))
}

fn parse_config_overlay(path: &Path) -> Option<PaddlesConfigOverlay> {
    let contents = std::fs::read_to_string(path).ok()?;
    match toml::from_str::<PaddlesConfigOverlay>(&contents) {
        Ok(config) => Some(config),
        Err(err) => {
            eprintln!("[WARN] Failed to parse {}: {err}", path.display());
            None
        }
    }
}

fn normalize_optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn repo_doc(path: &str) -> String {
        fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
            .unwrap_or_else(|err| panic!("read {path}: {err}"))
    }

    #[test]
    fn default_config_uses_sift_and_qwen() {
        let config = PaddlesConfig::default();
        assert_eq!(config.provider, "sift");
        assert_eq!(config.model, "qwen-1.5b");
        assert_eq!(config.port, 3000);
        assert_eq!(config.weights, 0.5);
        assert_eq!(config.biases, 0.0);
    }

    #[test]
    fn startup_port_requests_ephemeral_binding_without_authored_config() {
        let requested = resolve_web_server_port(None, PaddlesConfig::default().port, false);

        assert_eq!(requested, 0);
    }

    #[test]
    fn startup_port_requests_ephemeral_binding_when_authored_config_omits_port() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace-paddles.toml");
        fs::write(
            &workspace,
            r#"
provider = "moonshot"
model = "kimi-k2.5"
"#,
        )
        .expect("write workspace config");

        let requested = resolve_web_server_port(
            None,
            PaddlesConfig::default().port,
            authored_port_is_configured_in_explicit_paths(Some(workspace.as_path()), None, None),
        );

        assert_eq!(requested, 0);
    }

    #[test]
    fn runtime_verbosity_prefers_cli_over_config() {
        assert_eq!(resolve_runtime_verbosity(1, 3), 1);
        assert_eq!(resolve_runtime_verbosity(3, 1), 3);
    }

    #[test]
    fn runtime_verbosity_uses_config_when_cli_is_unset() {
        assert_eq!(resolve_runtime_verbosity(0, 2), 2);
        assert_eq!(resolve_runtime_verbosity(0, 0), 0);
    }

    #[test]
    fn load_parses_toml_from_workspace() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("paddles.toml"),
            r#"
provider = "openai"
model = "gpt-4o"
port = 8080
"#,
        )
        .expect("write config");

        let config = PaddlesConfig::load(dir.path());
        assert_eq!(config.provider, "openai");
        assert_eq!(config.model, "gpt-4o");
        assert_eq!(config.port, 8080);
        // Unset fields use defaults
        assert_eq!(config.weights, 0.5);
        assert!(config.planner_model.is_none());
        assert!(config.planner_provider.is_none());
    }

    #[test]
    fn load_parses_lane_sections_with_shared_fallback_structure() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(
            dir.path().join("paddles.toml"),
            r#"
[shared]
provider = "moonshot"
model = "kimi-k2.5"

[synthesizer]
model = "gpt-4o-mini"

[planner]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
"#,
        )
        .expect("write config");

        let config = PaddlesConfig::load(dir.path());
        assert_eq!(config.provider, "moonshot");
        assert_eq!(config.model, "kimi-k2.5");
        assert_eq!(config.synthesizer_provider, None);
        assert_eq!(config.synthesizer_model.as_deref(), Some("gpt-4o-mini"));
        assert_eq!(config.planner_provider.as_deref(), Some("anthropic"));
        assert_eq!(
            config.planner_model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn load_layers_runtime_preferences_after_authored_config() {
        let dir = tempfile::tempdir().expect("tempdir");
        let system = dir.path().join("etc-paddles.toml");
        let user = dir.path().join("user-paddles.toml");
        let workspace = dir.path().join("workspace-paddles.toml");
        fs::write(
            &system,
            r#"
provider = "openai"
model = "gpt-4o-mini"
port = 8080
"#,
        )
        .expect("write system config");
        fs::write(
            &user,
            r#"
provider = "moonshot"
model = "kimi-k2.5"
"#,
        )
        .expect("write user config");
        fs::write(
            &workspace,
            r#"
port = 9090
"#,
        )
        .expect("write workspace config");

        let preferences = RuntimeLanePreferences {
            provider: Some("inception".to_string()),
            model: Some("mercury-2".to_string()),
            planner_provider: Some("anthropic".to_string()),
            planner_model: Some("claude-sonnet-4-20250514".to_string()),
        };

        let config = PaddlesConfig::load_from_explicit_paths(
            Some(workspace.as_path()),
            Some(user.as_path()),
            Some(system.as_path()),
            Some(&preferences),
        );

        assert_eq!(config.provider, "inception");
        assert_eq!(config.model, "mercury-2");
        assert!(config.synthesizer_provider.is_none());
        assert!(config.synthesizer_model.is_none());
        assert!(config.planner_provider.is_none());
        assert!(config.planner_model.is_none());
        assert_eq!(config.port, 9090);
    }

    #[test]
    fn runtime_preferences_override_user_config_when_workspace_is_absent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let user = dir.path().join("user-paddles.toml");
        fs::write(
            &user,
            r#"
provider = "openai"
model = "gpt-4o"
"#,
        )
        .expect("write user config");

        let preferences = RuntimeLanePreferences {
            provider: Some("inception".to_string()),
            model: Some("mercury-2".to_string()),
            planner_provider: None,
            planner_model: None,
        };

        let config = PaddlesConfig::load_from_explicit_paths(
            None,
            Some(user.as_path()),
            None,
            Some(&preferences),
        );

        assert_eq!(config.provider, "inception");
        assert_eq!(config.model, "mercury-2");
    }

    #[test]
    fn runtime_preferences_override_workspace_config_for_lane_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let workspace = dir.path().join("workspace-paddles.toml");
        fs::write(
            &workspace,
            r#"
[shared]
provider = "sift"
model = "qwen-1.5b"

[synthesizer]
provider = "moonshot"
model = "kimi-k2.5"

[planner]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
"#,
        )
        .expect("write workspace config");

        let preferences = RuntimeLanePreferences {
            provider: Some("inception".to_string()),
            model: Some("mercury-2".to_string()),
            planner_provider: Some("anthropic".to_string()),
            planner_model: Some("claude-sonnet-4-20250514".to_string()),
        };

        let config = PaddlesConfig::load_from_explicit_paths(
            Some(workspace.as_path()),
            None,
            None,
            Some(&preferences),
        );

        assert_eq!(config.provider, "inception");
        assert_eq!(config.model, "mercury-2");
        assert!(config.synthesizer_provider.is_none());
        assert!(config.synthesizer_model.is_none());
        assert!(config.planner_provider.is_none());
        assert!(config.planner_model.is_none());
    }

    #[test]
    fn load_returns_defaults_when_no_file_exists() {
        let dir = tempfile::tempdir().expect("tempdir");
        let config = PaddlesConfig::load(dir.path());
        assert_eq!(config.provider, "sift");
        assert_eq!(config.model, "qwen-1.5b");
    }

    #[test]
    fn normalizes_legacy_moonshot_model_alias() {
        assert_eq!(
            normalize_provider_model_alias("moonshot", "kimi-2.5"),
            "kimi-k2.5"
        );
        assert_eq!(
            normalize_provider_model_alias("moonshot", "kimi-k2.5"),
            "kimi-k2.5"
        );
        assert_eq!(normalize_provider_model_alias("openai", "gpt-4o"), "gpt-4o");
    }

    #[test]
    fn normalizes_gatherer_provider_alias() {
        assert_eq!(
            normalize_gatherer_provider_alias("sift-direct"),
            "sift-direct"
        );
        assert_eq!(normalize_gatherer_provider_alias("local"), "local");
        assert_eq!(normalize_gatherer_provider_alias("context1"), "context1");
    }

    #[test]
    fn readme_documents_inception_authentication_and_model_selection() {
        let readme = repo_doc("README.md");
        assert!(readme.contains("Inception"));
        assert!(readme.contains("/login inception"));
        assert!(readme.contains("/model inception mercury-2"));
        assert!(readme.contains("mercury-2"));
        assert!(readme.contains("runtime-lanes.toml"));
    }

    #[test]
    fn configuration_guidance_distinguishes_core_inception_support_from_optional_capabilities() {
        let configuration = repo_doc("CONFIGURATION.md");
        assert!(configuration.contains("Inception"));
        assert!(configuration.contains("mercury-2"));
        assert!(configuration.contains("workspace editor"));
        assert!(configuration.contains("streaming/diffusion"));
        assert!(configuration.contains("Optional native capabilities"));
    }

    #[test]
    fn configuration_guidance_marks_inception_core_path_as_immediately_usable() {
        let configuration = repo_doc("CONFIGURATION.md");
        assert!(configuration.contains("usable today"));
        assert!(configuration.contains("without"));
        assert!(configuration.contains("streaming/diffusion"));
        assert!(configuration.contains("workspace editor"));
        assert!(configuration.contains("runtime-lanes.toml"));
    }
}
