use serde::Deserialize;
use std::path::{Path, PathBuf};

use crate::infrastructure::providers::ModelProvider;

const CONFIG_FILE_NAME: &str = "paddles.toml";
const USER_CONFIG_RELATIVE_PATH: &str = ".config/paddles/paddles.toml";
const SYSTEM_CONFIG_PATH: &str = "/etc/paddles/paddles.toml";

/// Paddles configuration loaded from paddles.toml.
///
/// Search order: `./paddles.toml`, `~/.config/paddles/paddles.toml`, `/etc/paddles/paddles.toml`.
/// First file found wins. CLI flags override all config values.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct PaddlesConfig {
    pub provider: String,
    pub provider_url: Option<String>,
    pub model: String,
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

impl Default for PaddlesConfig {
    fn default() -> Self {
        Self {
            provider: "sift".to_string(),
            provider_url: None,
            model: "qwen-1.5b".to_string(),
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
    /// Load configuration from the first paddles.toml found in the search path.
    pub fn load(workspace_root: &Path) -> Self {
        let candidates = config_search_paths(workspace_root);
        for path in &candidates {
            if let Ok(contents) = std::fs::read_to_string(path) {
                match toml::from_str::<PaddlesConfig>(&contents) {
                    Ok(config) => return config,
                    Err(err) => {
                        eprintln!("[WARN] Failed to parse {}: {err}", path.display());
                    }
                }
            }
        }
        Self::default()
    }

    /// Path where the config was found, if any.
    pub fn find_config_path(workspace_root: &Path) -> Option<PathBuf> {
        config_search_paths(workspace_root)
            .into_iter()
            .find(|p| p.exists())
    }
}

/// Normalizes provider-specific model aliases so legacy configs keep working.
pub fn normalize_provider_model_alias(provider: &str, model: &str) -> String {
    ModelProvider::from_name(provider)
        .map(|provider| provider.normalize_model_alias(model))
        .unwrap_or_else(|| model.to_string())
}

/// Normalizes legacy gatherer provider aliases so old configs remain explicit and compatible.
pub fn normalize_gatherer_provider_alias(provider: &str) -> String {
    match provider {
        "sift-autonomous" => "sift-direct".to_string(),
        _ => provider.to_string(),
    }
}

fn config_search_paths(workspace_root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![workspace_root.join(CONFIG_FILE_NAME)];
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(USER_CONFIG_RELATIVE_PATH));
    }
    paths.push(PathBuf::from(SYSTEM_CONFIG_PATH));
    paths
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
    fn normalizes_legacy_gatherer_provider_alias() {
        assert_eq!(
            normalize_gatherer_provider_alias("sift-autonomous"),
            "sift-direct"
        );
        assert_eq!(
            normalize_gatherer_provider_alias("sift-direct"),
            "sift-direct"
        );
        assert_eq!(normalize_gatherer_provider_alias("context1"), "context1");
    }

    #[test]
    fn readme_documents_inception_authentication_and_model_selection() {
        let readme = repo_doc("README.md");
        assert!(readme.contains("Inception"));
        assert!(readme.contains("/login inception"));
        assert!(readme.contains("/model synthesizer inception mercury-2"));
        assert!(readme.contains("mercury-2"));
    }

    #[test]
    fn configuration_guidance_distinguishes_core_inception_support_from_optional_capabilities() {
        let configuration = repo_doc("CONFIGURATION.md");
        assert!(configuration.contains("Inception"));
        assert!(configuration.contains("mercury-2"));
        assert!(configuration.contains("streaming/diffusion"));
        assert!(configuration.contains("edit-native"));
        assert!(configuration.contains("optional"));
    }

    #[test]
    fn configuration_guidance_marks_inception_core_path_as_immediately_usable() {
        let configuration = repo_doc("CONFIGURATION.md");
        assert!(configuration.contains("usable today"));
        assert!(configuration.contains("without"));
        assert!(configuration.contains("streaming/diffusion"));
        assert!(configuration.contains("edit-native"));
    }
}
