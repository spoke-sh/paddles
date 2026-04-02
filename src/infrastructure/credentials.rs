use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::infrastructure::providers::{ModelProvider, ProviderAuthRequirement};

const CREDENTIALS_FILE: &str = "credentials.toml";
const KEYS_DIR: &str = "keys";

/// Maps provider names to relative paths of their key files.
///
/// ```toml
/// [keys]
/// moonshot = "keys/moonshot.key"
/// openai = "keys/openai.key"
/// ```
#[derive(Debug, Default, Deserialize, Serialize)]
struct CredentialsManifest {
    #[serde(default)]
    keys: BTreeMap<String, String>,
}

/// Stores API keys in individual files under `~/.config/paddles/keys/`,
/// indexed by `~/.config/paddles/credentials.toml`.
pub struct CredentialStore {
    base_dir: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ApiKeySource {
    Environment { env_var: String },
    StoredFile { provider: String },
    Missing { provider: String },
}

impl ApiKeySource {
    pub fn interactive_status(&self, provider_name: &str) -> String {
        match self {
            Self::Environment { env_var } => format!(
                "Provider: `{provider_name}`. Auth: using `{env_var}` from the environment. Environment values override stored credentials on restart."
            ),
            Self::StoredFile { .. } => format!(
                "Provider: `{provider_name}`. Auth: loaded from the local credential store."
            ),
            Self::Missing { .. } => format!("Provider: `{provider_name}`. Auth: not configured."),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedApiKey {
    pub value: String,
    pub source: ApiKeySource,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderAvailability {
    pub provider: ModelProvider,
    pub enabled: bool,
    pub detail: String,
}

impl Default for CredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore {
    pub fn new() -> Self {
        let base_dir = std::env::var("HOME")
            .map(|h| PathBuf::from(h).join(".config/paddles"))
            .unwrap_or_else(|_| PathBuf::from(".config/paddles"));
        Self { base_dir }
    }

    #[cfg(test)]
    fn with_base_dir(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Load the API key for a provider, returning `None` if not configured.
    pub fn load_api_key(&self, provider: &str) -> Option<String> {
        let manifest = self.load_manifest().ok()?;
        let rel_path = manifest.keys.get(provider)?;
        let key_path = self.base_dir.join(rel_path);
        std::fs::read_to_string(key_path)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Save an API key for a provider. Creates directories and sets 0600
    /// permissions on the key file.
    pub fn save_api_key(&self, provider: &str, key: &str) -> Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        let keys_dir = self.base_dir.join(KEYS_DIR);
        std::fs::create_dir_all(&keys_dir)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&self.base_dir, std::fs::Permissions::from_mode(0o700))?;
            std::fs::set_permissions(&keys_dir, std::fs::Permissions::from_mode(0o700))?;
        }

        let key_filename = format!("{provider}.key");
        let key_path = keys_dir.join(&key_filename);
        std::fs::write(&key_path, key.trim())?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
        }

        let mut manifest = self.load_manifest().unwrap_or_default();
        manifest
            .keys
            .insert(provider.to_string(), format!("{KEYS_DIR}/{key_filename}"));
        let manifest_path = self.base_dir.join(CREDENTIALS_FILE);
        std::fs::write(&manifest_path, toml::to_string_pretty(&manifest)?)?;

        Ok(())
    }

    fn load_manifest(&self) -> Result<CredentialsManifest> {
        let path = self.base_dir.join(CREDENTIALS_FILE);
        let contents = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&contents)?)
    }

    /// The provider name used as the credential key for a given env-var name.
    pub fn provider_for_env(env_name: &str) -> Option<&'static str> {
        match env_name {
            "OPENAI_API_KEY" => Some("openai"),
            "ANTHROPIC_API_KEY" => Some("anthropic"),
            "GOOGLE_API_KEY" => Some("google"),
            "MOONSHOT_API_KEY" => Some("moonshot"),
            "OLLAMA_API_KEY" => Some("ollama"),
            _ => None,
        }
    }

    pub fn resolve_api_key(&self, env_name: &str) -> ResolvedApiKey {
        let env_value = std::env::var(env_name).ok().filter(|key| !key.is_empty());
        self.resolve_api_key_from(env_name, env_value.as_deref())
    }

    pub fn resolve_provider_api_key(&self, provider: ModelProvider) -> ResolvedApiKey {
        match provider.credential_env_var() {
            Some(env_name) => self.resolve_api_key(env_name),
            None => ResolvedApiKey {
                value: String::new(),
                source: ApiKeySource::Missing {
                    provider: provider.name().to_string(),
                },
            },
        }
    }

    pub fn provider_availability(&self, provider: ModelProvider) -> ProviderAvailability {
        match provider.auth_requirement() {
            ProviderAuthRequirement::None => ProviderAvailability {
                provider,
                enabled: true,
                detail: "auth not required".to_string(),
            },
            ProviderAuthRequirement::OptionalApiKey => {
                let resolved = self.resolve_provider_api_key(provider);
                let detail = match resolved.source {
                    ApiKeySource::Environment { env_var } => {
                        format!("optional key from environment `{env_var}`")
                    }
                    ApiKeySource::StoredFile { .. } => {
                        "optional key from local credential store".to_string()
                    }
                    ApiKeySource::Missing { .. } => "auth not required".to_string(),
                };
                ProviderAvailability {
                    provider,
                    enabled: true,
                    detail,
                }
            }
            ProviderAuthRequirement::RequiredApiKey => {
                let resolved = self.resolve_provider_api_key(provider);
                let (enabled, detail) = match resolved.source {
                    ApiKeySource::Environment { env_var } => {
                        (true, format!("using `{env_var}` from environment"))
                    }
                    ApiKeySource::StoredFile { .. } => {
                        (true, "using local credential store".to_string())
                    }
                    ApiKeySource::Missing { .. } => (false, "login required".to_string()),
                };
                ProviderAvailability {
                    provider,
                    enabled,
                    detail,
                }
            }
        }
    }

    pub fn all_provider_availability(&self) -> Vec<ProviderAvailability> {
        ModelProvider::all()
            .iter()
            .copied()
            .map(|provider| self.provider_availability(provider))
            .collect()
    }

    fn resolve_api_key_from(&self, env_name: &str, env_value: Option<&str>) -> ResolvedApiKey {
        if let Some(value) = env_value {
            return ResolvedApiKey {
                value: value.to_string(),
                source: ApiKeySource::Environment {
                    env_var: env_name.to_string(),
                },
            };
        }

        let provider = Self::provider_for_env(env_name).unwrap_or(env_name);
        if let Some(value) = self.load_api_key(provider) {
            return ResolvedApiKey {
                value,
                source: ApiKeySource::StoredFile {
                    provider: provider.to_string(),
                },
            };
        }

        ResolvedApiKey {
            value: String::new(),
            source: ApiKeySource::Missing {
                provider: provider.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::providers::ModelProvider;

    #[test]
    fn save_and_load_round_trips() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store
            .save_api_key("moonshot", "sk-test-key-123")
            .expect("save");
        let loaded = store.load_api_key("moonshot");
        assert_eq!(loaded.as_deref(), Some("sk-test-key-123"));
    }

    #[test]
    fn load_returns_none_for_unknown_provider() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        assert!(store.load_api_key("nonexistent").is_none());
    }

    #[test]
    fn save_overwrites_existing_key() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store.save_api_key("openai", "old-key").expect("save old");
        store.save_api_key("openai", "new-key").expect("save new");
        assert_eq!(store.load_api_key("openai").as_deref(), Some("new-key"));
    }

    #[test]
    fn key_files_are_stored_separately() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store.save_api_key("moonshot", "moon-key").expect("save");
        store.save_api_key("openai", "oai-key").expect("save");

        // Each key lives in its own file
        let moon = std::fs::read_to_string(dir.path().join("keys/moonshot.key")).expect("read");
        let oai = std::fs::read_to_string(dir.path().join("keys/openai.key")).expect("read");
        assert_eq!(moon, "moon-key");
        assert_eq!(oai, "oai-key");

        // Manifest references both
        let manifest =
            std::fs::read_to_string(dir.path().join("credentials.toml")).expect("read manifest");
        assert!(manifest.contains("moonshot"));
        assert!(manifest.contains("openai"));
    }

    #[cfg(unix)]
    #[test]
    fn key_file_has_restricted_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store.save_api_key("moonshot", "secret").expect("save");
        let perms = std::fs::metadata(dir.path().join("keys/moonshot.key"))
            .expect("metadata")
            .permissions();
        assert_eq!(perms.mode() & 0o777, 0o600);
    }

    #[cfg(unix)]
    #[test]
    fn credential_directories_are_restricted() {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store.save_api_key("moonshot", "secret").expect("save");

        let base_perms = std::fs::metadata(dir.path())
            .expect("base metadata")
            .permissions();
        let keys_perms = std::fs::metadata(dir.path().join("keys"))
            .expect("keys metadata")
            .permissions();
        assert_eq!(base_perms.mode() & 0o777, 0o700);
        assert_eq!(keys_perms.mode() & 0o777, 0o700);
    }

    #[test]
    fn empty_key_file_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        store.save_api_key("moonshot", "  ").expect("save");
        assert!(store.load_api_key("moonshot").is_none());
    }

    #[test]
    fn provider_for_env_maps_known_vars() {
        assert_eq!(
            CredentialStore::provider_for_env("MOONSHOT_API_KEY"),
            Some("moonshot")
        );
        assert_eq!(
            CredentialStore::provider_for_env("OPENAI_API_KEY"),
            Some("openai")
        );
        assert_eq!(CredentialStore::provider_for_env("UNKNOWN_KEY"), None);
    }

    #[test]
    fn resolve_api_key_prefers_environment_values() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());
        store.save_api_key("moonshot", "stored-key").expect("save");

        let resolved = store.resolve_api_key_from("MOONSHOT_API_KEY", Some("env-key"));
        assert_eq!(resolved.value, "env-key");
        assert_eq!(
            resolved.source,
            ApiKeySource::Environment {
                env_var: "MOONSHOT_API_KEY".to_string()
            }
        );
    }

    #[test]
    fn resolve_api_key_falls_back_to_stored_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());
        store.save_api_key("moonshot", "stored-key").expect("save");

        let resolved = store.resolve_api_key_from("MOONSHOT_API_KEY", None);
        assert_eq!(resolved.value, "stored-key");
        assert_eq!(
            resolved.source,
            ApiKeySource::StoredFile {
                provider: "moonshot".to_string()
            }
        );
    }

    #[test]
    fn resolve_api_key_reports_missing_configuration() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        let resolved = store.resolve_api_key_from("MOONSHOT_API_KEY", None);
        assert_eq!(resolved.value, "");
        assert_eq!(
            resolved.source,
            ApiKeySource::Missing {
                provider: "moonshot".to_string()
            }
        );
    }

    #[test]
    fn required_remote_provider_is_disabled_when_missing_credentials() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        let availability = store.provider_availability(ModelProvider::Openai);

        assert_eq!(availability.provider, ModelProvider::Openai);
        assert!(!availability.enabled);
        assert_eq!(availability.detail, "login required");
    }

    #[test]
    fn local_provider_is_enabled_without_credentials() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        let availability = store.provider_availability(ModelProvider::Sift);

        assert_eq!(availability.provider, ModelProvider::Sift);
        assert!(availability.enabled);
        assert_eq!(availability.detail, "auth not required");
    }

    #[test]
    fn optional_local_provider_stays_enabled_without_credentials() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = CredentialStore::with_base_dir(dir.path());

        let availability = store.provider_availability(ModelProvider::Ollama);

        assert_eq!(availability.provider, ModelProvider::Ollama);
        assert!(availability.enabled);
        assert_eq!(availability.detail, "auth not required");
    }
}
