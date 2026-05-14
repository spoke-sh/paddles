use crate::application::{GathererProvider, RuntimeLaneConfig};
use crate::infrastructure::providers::ModelProvider;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const TURN_RUNTIME_PREFERENCES_FILE: &str = "turn-runtime.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnRuntimePreferences {
    pub turn_runtime: TurnRuntimePreferenceDocument,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnRuntimePreferenceDocument {
    pub model_clients: TurnRuntimeModelClientPreferences,
    pub retrieval: TurnRuntimeRetrievalPreference,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnRuntimeModelClientPreferences {
    pub action_selection: ModelClientPreference,
    pub final_rendering: ModelClientPreference,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ModelClientPreference {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub thinking_mode: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct TurnRuntimeRetrievalPreference {
    pub provider: Option<String>,
    pub model: Option<String>,
}

pub type RuntimeLanePreferences = TurnRuntimePreferences;
pub type RuntimeLanePreferenceStore = TurnRuntimePreferenceStore;

impl TurnRuntimePreferences {
    pub fn from_runtime_lanes(runtime_lanes: &RuntimeLaneConfig) -> Self {
        let final_rendering_provider = runtime_lanes.synthesizer_provider();
        let final_rendering_model = runtime_lanes.synthesizer_model_id();
        let action_selection_provider = runtime_lanes.planner_provider();
        let action_selection_model = runtime_lanes
            .planner_model_id()
            .unwrap_or(runtime_lanes.synthesizer_model_id());
        let action_selection_thinking_mode = (action_selection_provider
            == final_rendering_provider
            && action_selection_model == final_rendering_model)
            .then(|| runtime_lanes.synthesizer_thinking_mode())
            .flatten()
            .map(ToString::to_string);

        Self {
            turn_runtime: TurnRuntimePreferenceDocument {
                model_clients: TurnRuntimeModelClientPreferences {
                    action_selection: ModelClientPreference {
                        provider: Some(action_selection_provider.name().to_string()),
                        model: Some(action_selection_model.to_string()),
                        thinking_mode: action_selection_thinking_mode,
                    },
                    final_rendering: ModelClientPreference {
                        provider: Some(final_rendering_provider.name().to_string()),
                        model: Some(final_rendering_model.to_string()),
                        thinking_mode: runtime_lanes
                            .synthesizer_thinking_mode()
                            .map(ToString::to_string),
                    },
                },
                retrieval: TurnRuntimeRetrievalPreference {
                    provider: Some(gatherer_provider_name(runtime_lanes.gatherer_provider())),
                    model: runtime_lanes.gatherer_model_id().map(ToString::to_string),
                },
            },
        }
    }

    pub fn action_selection(&self) -> &ModelClientPreference {
        &self.turn_runtime.model_clients.action_selection
    }

    pub fn final_rendering(&self) -> &ModelClientPreference {
        &self.turn_runtime.model_clients.final_rendering
    }

    pub fn retrieval(&self) -> &TurnRuntimeRetrievalPreference {
        &self.turn_runtime.retrieval
    }

    pub fn is_empty(&self) -> bool {
        self.action_selection().is_empty()
            && self.final_rendering().is_empty()
            && self.retrieval().is_empty()
    }

    fn normalize_machine_managed_aliases(&mut self) -> bool {
        let mut changed = false;

        if normalize_machine_managed_model_alias(
            &self.turn_runtime.model_clients.final_rendering.provider,
            &mut self.turn_runtime.model_clients.final_rendering.model,
        ) {
            changed = true;
        }

        if normalize_machine_managed_model_alias(
            &self.turn_runtime.model_clients.action_selection.provider,
            &mut self.turn_runtime.model_clients.action_selection.model,
        ) {
            changed = true;
        }

        changed
    }
}

impl ModelClientPreference {
    fn is_empty(&self) -> bool {
        self.provider.is_none() && self.model.is_none() && self.thinking_mode.is_none()
    }
}

impl TurnRuntimeRetrievalPreference {
    fn is_empty(&self) -> bool {
        self.provider.is_none() && self.model.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnRuntimePreferenceStore {
    path: PathBuf,
}

impl Default for TurnRuntimePreferenceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl TurnRuntimePreferenceStore {
    pub fn new() -> Self {
        Self::with_path(default_turn_runtime_preference_path())
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<TurnRuntimePreferences>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path).with_context(|| {
            format!("read turn runtime preferences from {}", self.path.display())
        })?;
        let mut preferences =
            toml::from_str::<TurnRuntimePreferences>(&contents).with_context(|| {
                format!(
                    "parse turn runtime preferences from {}",
                    self.path.display()
                )
            })?;

        let preferences_were_normalized = preferences.normalize_machine_managed_aliases();

        if preferences_were_normalized {
            self.save(&preferences)?;
        }

        if preferences.is_empty() {
            Ok(None)
        } else {
            Ok(Some(preferences))
        }
    }

    pub fn save(&self, preferences: &TurnRuntimePreferences) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "create turn runtime preference directory {}",
                    parent.display()
                )
            })?;
        }

        let contents =
            toml::to_string(preferences).context("serialize turn runtime preferences as toml")?;
        fs::write(&self.path, contents).with_context(|| {
            format!("write turn runtime preferences to {}", self.path.display())
        })?;
        Ok(())
    }
}

pub fn default_turn_runtime_preference_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles")
        && let Some(state_dir) = project_dirs.state_dir()
    {
        return state_dir.join(TURN_RUNTIME_PREFERENCES_FILE);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("paddles")
            .join(TURN_RUNTIME_PREFERENCES_FILE);
    }

    PathBuf::from(TURN_RUNTIME_PREFERENCES_FILE)
}

pub fn default_runtime_lane_preference_path() -> PathBuf {
    default_turn_runtime_preference_path()
}

fn normalize_machine_managed_model_alias(
    provider: &Option<String>,
    _model: &mut Option<String>,
) -> bool {
    if provider.is_none() {
        return false;
    }

    if !matches!(
        provider.as_deref().and_then(ModelProvider::from_name),
        Some(ModelProvider::Openai)
    ) {
        return false;
    }

    false
}

fn gatherer_provider_name(provider: GathererProvider) -> String {
    match provider {
        GathererProvider::Local => "local",
        GathererProvider::SiftDirect => "sift-direct",
        GathererProvider::Context1 => "context1",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeLanePreferenceStore, RuntimeLanePreferences, default_turn_runtime_preference_path,
    };
    use crate::application::{GathererProvider, RuntimeLaneConfig};
    use crate::infrastructure::providers::ModelProvider;

    #[test]
    fn turn_runtime_preferences_capture_model_clients_and_retrieval() {
        let runtime_lanes =
            RuntimeLaneConfig::new("gpt-4o".to_string(), Some("retrieval-qwen".to_string()))
                .with_synthesizer_provider(ModelProvider::Openai)
                .with_planner_provider(Some(ModelProvider::Anthropic))
                .with_planner_model_id(Some("claude-sonnet-4-20250514".to_string()))
                .with_gatherer_provider(GathererProvider::Local);

        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        assert_eq!(
            preferences.action_selection().provider.as_deref(),
            Some("anthropic")
        );
        assert_eq!(
            preferences.action_selection().model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );
        assert_eq!(
            preferences.final_rendering().provider.as_deref(),
            Some("openai")
        );
        assert_eq!(
            preferences.final_rendering().model.as_deref(),
            Some("gpt-4o")
        );
        assert_eq!(preferences.retrieval().provider.as_deref(), Some("local"));
        assert_eq!(
            preferences.retrieval().model.as_deref(),
            Some("retrieval-qwen")
        );
    }

    #[test]
    fn turn_runtime_preferences_record_shared_model_clients_without_lane_names() {
        let runtime_lanes = RuntimeLaneConfig::new("mercury-2".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Inception);

        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        assert_eq!(
            preferences.action_selection().provider.as_deref(),
            Some("inception")
        );
        assert_eq!(
            preferences.action_selection().model.as_deref(),
            Some("mercury-2")
        );
        assert_eq!(
            preferences.final_rendering().provider.as_deref(),
            Some("inception")
        );
        assert_eq!(
            preferences.final_rendering().model.as_deref(),
            Some("mercury-2")
        );
        assert_eq!(
            preferences.retrieval().provider.as_deref(),
            Some("sift-direct")
        );
    }

    #[test]
    fn turn_runtime_preference_store_writes_canonical_shape_without_lane_terms() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("state/turn-runtime.toml");
        let store = RuntimeLanePreferenceStore::with_path(&path);
        let runtime_lanes = RuntimeLaneConfig::new("gpt-4o".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_planner_provider(Some(ModelProvider::Anthropic))
            .with_planner_model_id(Some("claude-sonnet-4-20250514".to_string()));
        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        store.save(&preferences).expect("save runtime preferences");
        let contents = std::fs::read_to_string(&path).expect("read runtime preferences");

        assert!(contents.contains("[turn_runtime.model_clients.action_selection]"));
        assert!(contents.contains("[turn_runtime.model_clients.final_rendering]"));
        assert!(contents.contains("[turn_runtime.retrieval]"));
        assert!(!contents.contains("planner"));
        assert!(!contents.contains("synthesizer"));
        assert!(!contents.contains("gatherer"));
        assert!(!contents.contains("lane"));
    }

    #[test]
    fn turn_runtime_preference_store_round_trips_preferences() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store =
            RuntimeLanePreferenceStore::with_path(dir.path().join("state/turn-runtime.toml"));
        let preferences = RuntimeLanePreferences::from_runtime_lanes(
            &RuntimeLaneConfig::new("mercury-2".to_string(), None)
                .with_synthesizer_provider(ModelProvider::Inception),
        );

        store.save(&preferences).expect("save runtime preferences");
        let loaded = store
            .load()
            .expect("load runtime preferences")
            .expect("stored preferences");

        assert_eq!(loaded, preferences);
    }

    #[test]
    fn turn_runtime_preference_store_preserves_openai_responses_only_models() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("state/turn-runtime.toml");
        let store = RuntimeLanePreferenceStore::with_path(&path);
        std::fs::create_dir_all(path.parent().expect("turn runtime preference parent"))
            .expect("create turn runtime preference parent");
        std::fs::write(
            &path,
            r#"[turn_runtime.model_clients.action_selection]
provider = "openai"
model = "gpt-5.2-pro"

[turn_runtime.model_clients.final_rendering]
provider = "openai"
model = "gpt-5.4-pro"

[turn_runtime.retrieval]
provider = "sift-direct"
"#,
        )
        .expect("write stale runtime preferences");

        let loaded = store
            .load()
            .expect("load runtime preferences")
            .expect("stored preferences");

        assert_eq!(
            loaded.action_selection().provider.as_deref(),
            Some("openai")
        );
        assert_eq!(
            loaded.action_selection().model.as_deref(),
            Some("gpt-5.2-pro")
        );
        assert_eq!(loaded.final_rendering().provider.as_deref(), Some("openai"));
        assert_eq!(
            loaded.final_rendering().model.as_deref(),
            Some("gpt-5.4-pro")
        );
        assert_eq!(loaded.retrieval().provider.as_deref(), Some("sift-direct"));
    }

    #[test]
    fn default_turn_runtime_preference_path_targets_machine_state() {
        let path = default_turn_runtime_preference_path();
        assert!(path.ends_with("paddles/turn-runtime.toml"));
    }

    #[test]
    fn turn_runtime_preferences_capture_shared_thinking_mode() {
        let runtime_lanes = RuntimeLaneConfig::new("gpt-5.4".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_synthesizer_thinking_mode(Some("high".to_string()));

        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        assert_eq!(
            preferences.final_rendering().provider.as_deref(),
            Some("openai")
        );
        assert_eq!(
            preferences.final_rendering().model.as_deref(),
            Some("gpt-5.4")
        );
        assert_eq!(
            preferences.final_rendering().thinking_mode.as_deref(),
            Some("high")
        );
        assert_eq!(
            preferences.action_selection().thinking_mode.as_deref(),
            Some("high")
        );
    }
}
