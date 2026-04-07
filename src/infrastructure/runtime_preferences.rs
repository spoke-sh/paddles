use crate::application::RuntimeLaneConfig;
use crate::infrastructure::providers::ModelProvider;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const RUNTIME_LANE_PREFERENCES_FILE: &str = "runtime-lanes.toml";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct RuntimeLanePreferences {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub planner_provider: Option<String>,
    pub planner_model: Option<String>,
}

impl RuntimeLanePreferences {
    pub fn from_runtime_lanes(runtime_lanes: &RuntimeLaneConfig) -> Self {
        let synthesizer_provider = runtime_lanes.synthesizer_provider();
        let synthesizer_model = runtime_lanes.synthesizer_model_id();
        let planner_provider = runtime_lanes.planner_provider_override();
        let effective_planner_model = runtime_lanes
            .planner_model_id()
            .unwrap_or(synthesizer_model);

        Self {
            provider: Some(synthesizer_provider.name().to_string()),
            model: Some(synthesizer_model.to_string()),
            planner_provider: planner_provider.map(|provider| provider.name().to_string()),
            planner_model: if planner_provider.is_some()
                || effective_planner_model != synthesizer_model
            {
                Some(effective_planner_model.to_string())
            } else {
                None
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.provider.is_none()
            && self.model.is_none()
            && self.planner_provider.is_none()
            && self.planner_model.is_none()
    }

    fn normalize_machine_managed_aliases(&mut self) -> bool {
        let mut changed = false;

        if normalize_machine_managed_model_alias(&self.provider, &mut self.model) {
            changed = true;
        }

        if normalize_machine_managed_model_alias(&self.planner_provider, &mut self.planner_model) {
            changed = true;
        }

        changed
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLanePreferenceStore {
    path: PathBuf,
}

impl Default for RuntimeLanePreferenceStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeLanePreferenceStore {
    pub fn new() -> Self {
        Self::with_path(default_runtime_lane_preference_path())
    }

    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<Option<RuntimeLanePreferences>> {
        if !self.path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&self.path).with_context(|| {
            format!("read runtime lane preferences from {}", self.path.display())
        })?;
        let mut preferences =
            toml::from_str::<RuntimeLanePreferences>(&contents).with_context(|| {
                format!(
                    "parse runtime lane preferences from {}",
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

    pub fn save(&self, preferences: &RuntimeLanePreferences) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "create runtime lane preference directory {}",
                    parent.display()
                )
            })?;
        }

        let contents =
            toml::to_string(preferences).context("serialize runtime lane preferences as toml")?;
        fs::write(&self.path, contents).with_context(|| {
            format!("write runtime lane preferences to {}", self.path.display())
        })?;
        Ok(())
    }
}

pub fn default_runtime_lane_preference_path() -> PathBuf {
    if let Some(project_dirs) = ProjectDirs::from("", "", "paddles")
        && let Some(state_dir) = project_dirs.state_dir()
    {
        return state_dir.join(RUNTIME_LANE_PREFERENCES_FILE);
    }

    if let Ok(home) = std::env::var("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("paddles")
            .join(RUNTIME_LANE_PREFERENCES_FILE);
    }

    PathBuf::from(RUNTIME_LANE_PREFERENCES_FILE)
}

fn normalize_machine_managed_model_alias(
    provider: &Option<String>,
    model: &mut Option<String>,
) -> bool {
    let Some(model) = model.as_mut() else {
        return false;
    };

    if !matches!(
        provider.as_deref().and_then(ModelProvider::from_name),
        Some(ModelProvider::Openai)
    ) {
        return false;
    }

    let normalized = match model.as_str() {
        "gpt-5.4-pro" => "gpt-5.4",
        "gpt-5-pro" => "gpt-5",
        "gpt-5.2-pro" => "gpt-5.2",
        _ => return false,
    };

    *model = normalized.to_string();
    true
}

#[cfg(test)]
mod tests {
    use super::{
        RuntimeLanePreferenceStore, RuntimeLanePreferences, default_runtime_lane_preference_path,
    };
    use crate::application::RuntimeLaneConfig;
    use crate::infrastructure::providers::ModelProvider;

    #[test]
    fn runtime_lane_preferences_capture_planner_override_details() {
        let runtime_lanes = RuntimeLaneConfig::new("gpt-4o".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Openai)
            .with_planner_provider(Some(ModelProvider::Anthropic))
            .with_planner_model_id(Some("claude-sonnet-4-20250514".to_string()));

        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        assert_eq!(preferences.provider.as_deref(), Some("openai"));
        assert_eq!(preferences.model.as_deref(), Some("gpt-4o"));
        assert_eq!(preferences.planner_provider.as_deref(), Some("anthropic"));
        assert_eq!(
            preferences.planner_model.as_deref(),
            Some("claude-sonnet-4-20250514")
        );
    }

    #[test]
    fn runtime_lane_preferences_omit_redundant_planner_defaults() {
        let runtime_lanes = RuntimeLaneConfig::new("mercury-2".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Inception);

        let preferences = RuntimeLanePreferences::from_runtime_lanes(&runtime_lanes);

        assert_eq!(preferences.provider.as_deref(), Some("inception"));
        assert_eq!(preferences.model.as_deref(), Some("mercury-2"));
        assert!(preferences.planner_provider.is_none());
        assert!(preferences.planner_model.is_none());
    }

    #[test]
    fn runtime_lane_preference_store_round_trips_preferences() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store =
            RuntimeLanePreferenceStore::with_path(dir.path().join("state/runtime-lanes.toml"));
        let preferences = RuntimeLanePreferences {
            provider: Some("inception".to_string()),
            model: Some("mercury-2".to_string()),
            planner_provider: Some("openai".to_string()),
            planner_model: Some("gpt-4o".to_string()),
        };

        store.save(&preferences).expect("save runtime preferences");
        let loaded = store
            .load()
            .expect("load runtime preferences")
            .expect("stored preferences");

        assert_eq!(loaded, preferences);
    }

    #[test]
    fn runtime_lane_preference_store_migrates_openai_responses_only_models_to_chat_models() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("state/runtime-lanes.toml");
        let store = RuntimeLanePreferenceStore::with_path(&path);
        std::fs::create_dir_all(path.parent().expect("runtime lane preference parent"))
            .expect("create runtime lane preference parent");
        std::fs::write(
            &path,
            r#"provider = "openai"
model = "gpt-5.4-pro"
planner_provider = "openai"
planner_model = "gpt-5.2-pro"
"#,
        )
        .expect("write stale runtime preferences");

        let loaded = store
            .load()
            .expect("load runtime preferences")
            .expect("stored preferences");

        assert_eq!(loaded.provider.as_deref(), Some("openai"));
        assert_eq!(loaded.model.as_deref(), Some("gpt-5.4"));
        assert_eq!(loaded.planner_provider.as_deref(), Some("openai"));
        assert_eq!(loaded.planner_model.as_deref(), Some("gpt-5.2"));
        assert_eq!(
            std::fs::read_to_string(&path).expect("read normalized runtime preferences"),
            "provider = \"openai\"\nmodel = \"gpt-5.4\"\nplanner_provider = \"openai\"\nplanner_model = \"gpt-5.2\"\n"
        );
    }

    #[test]
    fn default_runtime_lane_preference_path_targets_machine_state() {
        let path = default_runtime_lane_preference_path();
        assert!(path.ends_with("paddles/runtime-lanes.toml"));
    }
}
