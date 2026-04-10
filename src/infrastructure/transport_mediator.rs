use crate::domain::model::{
    ExecutionHandDescriptor, ExecutionHandDiagnostic, ExecutionHandKind, ExecutionHandOperation,
    ExecutionHandPhase, NativeTransportConfigurations, NativeTransportKind,
};
use crate::domain::ports::ExecutionHand;
use crate::infrastructure::credentials::{CredentialStore, ResolvedApiKey};
use crate::infrastructure::execution_hand::ExecutionHandRegistry;
use crate::infrastructure::providers::{ModelProvider, ProviderAuthRequirement};
use anyhow::{Result, bail};
use std::collections::BTreeSet;
use std::process::Command;
use std::sync::Arc;

const HUGGING_FACE_TOKEN_ENV: &str = "HF_TOKEN";

#[derive(Clone, Debug)]
pub struct TransportToolMediator {
    credential_store: Arc<CredentialStore>,
    execution_hand_registry: Arc<ExecutionHandRegistry>,
    protected_env_vars: BTreeSet<String>,
}

impl TransportToolMediator {
    pub fn new(
        credential_store: Arc<CredentialStore>,
        execution_hand_registry: Arc<ExecutionHandRegistry>,
        native_transport_configurations: &NativeTransportConfigurations,
    ) -> Self {
        Self {
            credential_store,
            execution_hand_registry,
            protected_env_vars: collect_protected_env_vars(native_transport_configurations),
        }
    }

    pub fn with_execution_hand_registry(
        execution_hand_registry: Arc<ExecutionHandRegistry>,
    ) -> Self {
        Self::new(
            Arc::new(CredentialStore::new()),
            execution_hand_registry,
            &NativeTransportConfigurations::default(),
        )
    }

    fn descriptor() -> ExecutionHandDescriptor {
        ExecutionHandDescriptor::new(
            ExecutionHandKind::TransportMediator,
            ExecutionHandKind::TransportMediator.default_authority(),
            ExecutionHandKind::TransportMediator.default_summary(),
            vec![
                ExecutionHandOperation::Describe,
                ExecutionHandOperation::Provision,
                ExecutionHandOperation::Execute,
                ExecutionHandOperation::Recover,
                ExecutionHandOperation::Degrade,
            ],
        )
    }

    fn record_phase(
        &self,
        phase: ExecutionHandPhase,
        operation: ExecutionHandOperation,
        summary: impl Into<String>,
        last_error: Option<String>,
    ) {
        self.execution_hand_registry.record_phase(
            ExecutionHandKind::TransportMediator,
            phase,
            operation,
            summary,
            last_error,
        );
    }

    pub fn resolve_provider_api_key(
        &self,
        provider: ModelProvider,
        model_id: &str,
    ) -> Result<String> {
        self.record_phase(
            ExecutionHandPhase::Executing,
            ExecutionHandOperation::Execute,
            format!(
                "transport mediator resolving provider credential for `{}`",
                provider.name()
            ),
            None,
        );
        let resolved = self.credential_store.resolve_provider_api_key(provider);
        if provider.auth_requirement() == ProviderAuthRequirement::RequiredApiKey
            && resolved.value.trim().is_empty()
        {
            let env_var = provider.credential_env_var().unwrap_or("PROVIDER_API_KEY");
            let message = format!(
                "provider `{}` is not authenticated; set `{}` or use `/login {}` before selecting `{}`",
                provider.name(),
                env_var,
                provider.name(),
                provider.qualified_model_label(model_id)
            );
            self.record_phase(
                ExecutionHandPhase::Failed,
                ExecutionHandOperation::Execute,
                format!(
                    "transport mediator could not resolve provider credential for `{}`",
                    provider.name()
                ),
                Some(message.clone()),
            );
            bail!(message);
        }

        let source = describe_api_key_source(&resolved);
        self.record_phase(
            ExecutionHandPhase::Ready,
            ExecutionHandOperation::Provision,
            format!(
                "transport mediator resolved provider credential for `{}` via {source}",
                provider.name()
            ),
            None,
        );
        Ok(resolved.value)
    }

    pub fn resolve_transport_bearer_token(
        &self,
        transport: NativeTransportKind,
        token_env: &str,
    ) -> Result<String> {
        self.record_phase(
            ExecutionHandPhase::Executing,
            ExecutionHandOperation::Execute,
            format!(
                "transport mediator resolving bearer token for `{}`",
                transport.label()
            ),
            None,
        );
        match std::env::var(token_env)
            .ok()
            .filter(|value| !value.trim().is_empty())
        {
            Some(value) => {
                self.record_phase(
                    ExecutionHandPhase::Ready,
                    ExecutionHandOperation::Provision,
                    format!(
                        "transport mediator resolved bearer token for `{}` from `{token_env}`",
                        transport.label()
                    ),
                    None,
                );
                Ok(value)
            }
            None => {
                let error = format!(
                    "{} transport bearer token env `{token_env}` is not set",
                    transport.label()
                );
                self.record_phase(
                    ExecutionHandPhase::Failed,
                    ExecutionHandOperation::Execute,
                    format!(
                        "transport mediator could not resolve bearer token for `{}`",
                        transport.label()
                    ),
                    Some(error.clone()),
                );
                bail!(error);
            }
        }
    }

    pub fn protect_command_env(&self, command: &mut Command, purpose: &str) {
        for env_var in &self.protected_env_vars {
            command.env_remove(env_var);
        }
        self.record_phase(
            ExecutionHandPhase::Ready,
            ExecutionHandOperation::Provision,
            format!(
                "transport mediator withheld {} protected credential env vars from {purpose}",
                self.protected_env_vars.len()
            ),
            None,
        );
    }

    #[cfg(test)]
    pub fn protected_env_vars(&self) -> Vec<String> {
        self.protected_env_vars.iter().cloned().collect()
    }
}

impl ExecutionHand for TransportToolMediator {
    fn describe(&self) -> ExecutionHandDescriptor {
        Self::descriptor()
    }

    fn diagnostic(&self) -> ExecutionHandDiagnostic {
        self.execution_hand_registry
            .diagnostic(ExecutionHandKind::TransportMediator)
            .unwrap_or_else(|| ExecutionHandDiagnostic::from_descriptor(&Self::descriptor()))
    }
}

fn collect_protected_env_vars(
    native_transport_configurations: &NativeTransportConfigurations,
) -> BTreeSet<String> {
    let mut env_vars = ModelProvider::all()
        .iter()
        .filter_map(|provider| provider.credential_env_var())
        .map(str::to_string)
        .collect::<BTreeSet<_>>();
    env_vars.insert(HUGGING_FACE_TOKEN_ENV.to_string());
    for configuration in native_transport_configurations.all() {
        if let Some(token_env) = configuration.auth.token_env.as_ref() {
            env_vars.insert(token_env.clone());
        }
    }
    for extra in [
        "PADDLES_HTTP_BEARER_TOKEN",
        "PADDLES_SSE_BEARER_TOKEN",
        "PADDLES_WEBSOCKET_BEARER_TOKEN",
        "PADDLES_TRANSIT_BEARER_TOKEN",
    ] {
        env_vars.insert(extra.to_string());
    }
    env_vars
}

fn describe_api_key_source(resolved: &ResolvedApiKey) -> String {
    match &resolved.source {
        crate::infrastructure::credentials::ApiKeySource::Environment { env_var } => {
            format!("environment `{env_var}`")
        }
        crate::infrastructure::credentials::ApiKeySource::StoredFile { .. } => {
            "local credential store".to_string()
        }
        crate::infrastructure::credentials::ApiKeySource::Missing { provider } => {
            format!("missing `{provider}` credential")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransportToolMediator;
    use crate::domain::model::{
        ExecutionHandKind, ExecutionHandPhase, NativeTransportAuth, NativeTransportAuthMode,
        NativeTransportConfiguration, NativeTransportConfigurations, NativeTransportKind,
    };
    use crate::infrastructure::credentials::CredentialStore;
    use crate::infrastructure::execution_hand::ExecutionHandRegistry;
    use std::sync::Arc;

    #[test]
    fn mediator_collects_provider_and_native_transport_secret_env_vars() {
        let registry = Arc::new(ExecutionHandRegistry::default());
        let mediator = TransportToolMediator::new(
            Arc::new(CredentialStore::new()),
            Arc::clone(&registry),
            &NativeTransportConfigurations {
                transit: NativeTransportConfiguration {
                    transport: NativeTransportKind::Transit,
                    enabled: true,
                    bind_target: None,
                    auth: NativeTransportAuth {
                        mode: NativeTransportAuthMode::BearerToken,
                        token_env: Some("PADDLES_NATIVE_TRANSIT_TOKEN".to_string()),
                    },
                },
                ..NativeTransportConfigurations::default()
            },
        );

        let protected = mediator.protected_env_vars();
        assert!(protected.contains(&"OPENAI_API_KEY".to_string()));
        assert!(protected.contains(&"HF_TOKEN".to_string()));
        assert!(protected.contains(&"PADDLES_NATIVE_TRANSIT_TOKEN".to_string()));
    }

    #[test]
    fn mediator_reports_failed_transport_diagnostics_when_bearer_token_is_missing() {
        unsafe {
            std::env::remove_var("PADDLES_MISSING_TRANSPORT_MEDIATOR_TOKEN");
        }
        let registry = Arc::new(ExecutionHandRegistry::default());
        let mediator = TransportToolMediator::new(
            Arc::new(CredentialStore::new()),
            Arc::clone(&registry),
            &NativeTransportConfigurations::default(),
        );

        let error = mediator
            .resolve_transport_bearer_token(
                NativeTransportKind::Transit,
                "PADDLES_MISSING_TRANSPORT_MEDIATOR_TOKEN",
            )
            .expect_err("missing bearer token should fail closed");
        assert!(
            error
                .to_string()
                .contains("PADDLES_MISSING_TRANSPORT_MEDIATOR_TOKEN")
        );

        let diagnostic = registry
            .diagnostic(ExecutionHandKind::TransportMediator)
            .expect("transport mediator diagnostic");
        assert_eq!(diagnostic.phase, ExecutionHandPhase::Failed);
        assert!(
            diagnostic
                .last_error
                .as_deref()
                .is_some_and(|error| error.contains("PADDLES_MISSING_TRANSPORT_MEDIATOR_TOKEN"))
        );
    }
}
