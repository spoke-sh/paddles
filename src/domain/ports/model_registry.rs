use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Port for model discovery and acquisition.
#[async_trait]
pub trait ModelRegistry: Send + Sync {
    /// Get the local paths for a model by its ID.
    async fn get_model_paths(&self, model_id: &str) -> Result<ModelPaths, anyhow::Error>;

    /// Report registry posture without requiring discovery side effects by default.
    fn provider_posture(&self, request: ProviderRegistryPostureRequest) -> ProviderRegistryPosture {
        ProviderRegistryPosture {
            entries: Vec::new(),
            network_discovery_required: request.allow_network_discovery,
        }
    }
}

/// Paths to local model assets.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelPaths {
    pub weights: Vec<PathBuf>,
    pub tokenizer: PathBuf,
    pub config: PathBuf,
    pub generation_config: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderModelPostureStatus {
    Configured,
    Discovered,
    Unavailable,
    Deprecated,
}

impl ProviderModelPostureStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Configured => "configured",
            Self::Discovered => "discovered",
            Self::Unavailable => "unavailable",
            Self::Deprecated => "deprecated",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderModelPostureEntry {
    pub provider: String,
    pub model_id: String,
    pub status: ProviderModelPostureStatus,
    pub reason: Option<String>,
}

impl ProviderModelPostureEntry {
    pub fn configured(provider: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self::new(
            provider,
            model_id,
            ProviderModelPostureStatus::Configured,
            None,
        )
    }

    pub fn discovered(provider: impl Into<String>, model_id: impl Into<String>) -> Self {
        Self::new(
            provider,
            model_id,
            ProviderModelPostureStatus::Discovered,
            None,
        )
    }

    pub fn unavailable(
        provider: impl Into<String>,
        model_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            provider,
            model_id,
            ProviderModelPostureStatus::Unavailable,
            Some(reason.into()),
        )
    }

    pub fn deprecated(
        provider: impl Into<String>,
        model_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::new(
            provider,
            model_id,
            ProviderModelPostureStatus::Deprecated,
            Some(reason.into()),
        )
    }

    fn new(
        provider: impl Into<String>,
        model_id: impl Into<String>,
        status: ProviderModelPostureStatus,
        reason: Option<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            model_id: model_id.into(),
            status,
            reason,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRegistryPostureRequest {
    pub allow_network_discovery: bool,
}

impl ProviderRegistryPostureRequest {
    pub fn local_first() -> Self {
        Self {
            allow_network_discovery: false,
        }
    }

    pub fn with_network_discovery() -> Self {
        Self {
            allow_network_discovery: true,
        }
    }
}

impl Default for ProviderRegistryPostureRequest {
    fn default() -> Self {
        Self::local_first()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRegistryPosture {
    pub entries: Vec<ProviderModelPostureEntry>,
    pub network_discovery_required: bool,
}

impl ProviderRegistryPosture {
    pub fn local_first(entries: Vec<ProviderModelPostureEntry>) -> Self {
        Self {
            entries,
            network_discovery_required: false,
        }
    }

    pub fn from_configured_models<I, P, M>(
        request: ProviderRegistryPostureRequest,
        models: I,
    ) -> Self
    where
        I: IntoIterator<Item = (P, M)>,
        P: Into<String>,
        M: Into<String>,
    {
        Self {
            entries: models
                .into_iter()
                .map(|(provider, model_id)| {
                    ProviderModelPostureEntry::configured(provider, model_id)
                })
                .collect(),
            network_discovery_required: request.allow_network_discovery,
        }
    }

    pub fn is_offline_safe(&self) -> bool {
        !self.network_discovery_required
    }

    pub fn entries_by_status(
        &self,
        status: ProviderModelPostureStatus,
    ) -> Vec<&ProviderModelPostureEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.status == status)
            .collect()
    }

    pub fn has_status(&self, status: ProviderModelPostureStatus) -> bool {
        self.entries.iter().any(|entry| entry.status == status)
    }
}
