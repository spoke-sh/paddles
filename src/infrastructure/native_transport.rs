use crate::domain::model::{
    NativeTransportConfiguration, NativeTransportConfigurations, NativeTransportDiagnostic,
    NativeTransportKind, NativeTransportPhase, NativeTransportSessionIdentity,
};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug)]
pub struct NativeTransportRegistry {
    diagnostics: Mutex<BTreeMap<NativeTransportKind, NativeTransportDiagnostic>>,
}

impl Default for NativeTransportRegistry {
    fn default() -> Self {
        Self::new(NativeTransportConfigurations::default())
    }
}

impl NativeTransportRegistry {
    pub fn new(configurations: NativeTransportConfigurations) -> Self {
        let diagnostics = configurations
            .all()
            .into_iter()
            .map(|configuration| {
                (
                    configuration.transport,
                    NativeTransportDiagnostic::from_configuration(configuration),
                )
            })
            .collect();
        Self {
            diagnostics: Mutex::new(diagnostics),
        }
    }

    pub fn diagnostics(&self) -> Vec<NativeTransportDiagnostic> {
        self.diagnostics
            .lock()
            .expect("native transport diagnostics lock")
            .values()
            .cloned()
            .collect()
    }

    pub fn diagnostic(&self, transport: NativeTransportKind) -> Option<NativeTransportDiagnostic> {
        self.diagnostics
            .lock()
            .expect("native transport diagnostics lock")
            .get(&transport)
            .cloned()
    }

    pub fn replace_configurations(&self, configurations: NativeTransportConfigurations) {
        let mut diagnostics = self
            .diagnostics
            .lock()
            .expect("native transport diagnostics lock");
        diagnostics.clear();
        diagnostics.extend(configurations.all().into_iter().map(|configuration| {
            (
                configuration.transport,
                NativeTransportDiagnostic::from_configuration(configuration),
            )
        }));
    }

    pub fn record_phase(&self, transport: NativeTransportKind, phase: NativeTransportPhase) {
        self.update_diagnostic(transport, |diagnostic| {
            diagnostic.phase = phase;
            if phase != NativeTransportPhase::Failed {
                diagnostic.last_error = None;
            }
        });
    }

    pub fn record_ready(
        &self,
        configuration: &NativeTransportConfiguration,
        session: Option<NativeTransportSessionIdentity>,
    ) {
        self.update_diagnostic(configuration.transport, |diagnostic| {
            diagnostic.enabled = configuration.enabled;
            diagnostic.bind_target = configuration.bind_target.clone();
            diagnostic.auth_mode = configuration.auth.mode;
            diagnostic.capabilities = configuration.transport.default_capabilities();
            diagnostic.phase = NativeTransportPhase::Ready;
            diagnostic.session = session;
            diagnostic.last_error = None;
        });
    }

    pub fn record_session(
        &self,
        transport: NativeTransportKind,
        session: NativeTransportSessionIdentity,
    ) {
        self.update_diagnostic(transport, |diagnostic| {
            diagnostic.phase = NativeTransportPhase::Ready;
            diagnostic.capabilities = transport.default_capabilities();
            diagnostic.session = Some(session);
            diagnostic.last_error = None;
        });
    }

    pub fn clear_session(&self, transport: NativeTransportKind) {
        self.update_diagnostic(transport, |diagnostic| {
            diagnostic.session = None;
        });
    }

    pub fn record_degraded(&self, transport: NativeTransportKind, error: impl Into<String>) {
        self.update_diagnostic(transport, |diagnostic| {
            diagnostic.phase = NativeTransportPhase::Degraded;
            diagnostic.session = None;
            diagnostic.last_error = Some(error.into());
        });
    }

    pub fn record_failure(&self, transport: NativeTransportKind, error: impl Into<String>) {
        self.update_diagnostic(transport, |diagnostic| {
            diagnostic.phase = NativeTransportPhase::Failed;
            diagnostic.session = None;
            diagnostic.last_error = Some(error.into());
        });
    }

    fn update_diagnostic(
        &self,
        transport: NativeTransportKind,
        update: impl FnOnce(&mut NativeTransportDiagnostic),
    ) {
        if let Some(diagnostic) = self
            .diagnostics
            .lock()
            .expect("native transport diagnostics lock")
            .get_mut(&transport)
        {
            update(diagnostic);
        }
    }
}

pub fn resolve_bind_target(
    configuration: &NativeTransportConfiguration,
    fallback_bind_target: &str,
) -> String {
    if configuration.enabled {
        configuration
            .bind_target
            .clone()
            .unwrap_or_else(|| fallback_bind_target.to_string())
    } else {
        fallback_bind_target.to_string()
    }
}

pub fn resolve_shared_web_bind_target(
    http_request_response: &NativeTransportConfiguration,
    server_sent_events: &NativeTransportConfiguration,
    websocket: &NativeTransportConfiguration,
    transit: &NativeTransportConfiguration,
    fallback_bind_target: &str,
) -> Result<String, String> {
    let shared_web_transports = [
        http_request_response,
        server_sent_events,
        websocket,
        transit,
    ];
    let enabled_targets = shared_web_transports
        .into_iter()
        .filter(|configuration| configuration.enabled)
        .filter_map(|configuration| {
            configuration
                .bind_target
                .as_ref()
                .map(|bind_target| (configuration.transport.label(), bind_target.as_str()))
        })
        .collect::<Vec<_>>();

    if let Some((_, canonical_target)) = enabled_targets.first() {
        let conflicting_labels = enabled_targets
            .iter()
            .filter_map(|(label, bind_target)| {
                (*bind_target != *canonical_target).then_some(*label)
            })
            .collect::<Vec<_>>();
        if !conflicting_labels.is_empty() {
            let shared_labels = shared_web_transports
                .into_iter()
                .filter(|configuration| configuration.enabled)
                .map(|configuration| configuration.transport.label())
                .collect::<Vec<_>>()
                .join(", ");
            return Err(format!(
                "{shared_labels} must share the same bind_target when they are enabled on the shared web listener"
            ));
        }
    }

    Ok(shared_web_transports.into_iter().fold(
        fallback_bind_target.to_string(),
        |resolved, configuration| resolve_bind_target(configuration, &resolved),
    ))
}

pub fn record_binding_started(
    registry: &NativeTransportRegistry,
    configuration: &NativeTransportConfiguration,
) {
    if configuration.enabled {
        registry.record_phase(configuration.transport, NativeTransportPhase::Binding);
    }
}

pub fn record_bound_transport(
    registry: &NativeTransportRegistry,
    configuration: &NativeTransportConfiguration,
    actual_bind_target: &str,
) {
    if configuration.enabled {
        let mut ready_configuration = configuration.clone();
        if ready_configuration.bind_target.is_none() {
            ready_configuration.bind_target = Some(actual_bind_target.to_string());
        }
        registry.record_ready(&ready_configuration, None);
    }
}

pub fn record_transport_failure(
    registry: &NativeTransportRegistry,
    configuration: &NativeTransportConfiguration,
    error: impl Into<String>,
) {
    if configuration.enabled {
        registry.record_failure(configuration.transport, error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::model::{NativeTransportAuth, NativeTransportAuthMode};

    #[test]
    fn registry_builds_diagnostics_from_transport_configuration() {
        let configurations = NativeTransportConfigurations {
            http_request_response: NativeTransportConfiguration {
                transport: NativeTransportKind::HttpRequestResponse,
                enabled: true,
                bind_target: Some("127.0.0.1:4100".to_string()),
                auth: NativeTransportAuth {
                    mode: NativeTransportAuthMode::BearerToken,
                    token_env: Some("PADDLES_HTTP_TOKEN".to_string()),
                },
            },
            ..NativeTransportConfigurations::default()
        };

        let registry = NativeTransportRegistry::new(configurations);
        let diagnostic = registry
            .diagnostic(NativeTransportKind::HttpRequestResponse)
            .expect("http diagnostic");

        assert!(diagnostic.enabled);
        assert_eq!(diagnostic.phase, NativeTransportPhase::Configured);
        assert_eq!(diagnostic.bind_target.as_deref(), Some("127.0.0.1:4100"));
        assert_eq!(diagnostic.auth_mode, NativeTransportAuthMode::BearerToken);
    }

    #[test]
    fn resolve_bind_target_prefers_authored_transport_target() {
        let configuration = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: Some("127.0.0.1:4100".to_string()),
            auth: NativeTransportAuth::default(),
        };

        assert_eq!(
            resolve_bind_target(&configuration, "0.0.0.0:3000"),
            "127.0.0.1:4100"
        );
    }

    #[test]
    fn record_bound_transport_promotes_phase_and_runtime_bind_target() {
        let configuration = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: None,
            auth: NativeTransportAuth::default(),
        };
        let registry = NativeTransportRegistry::new(NativeTransportConfigurations {
            http_request_response: configuration.clone(),
            ..NativeTransportConfigurations::default()
        });

        record_binding_started(&registry, &configuration);
        record_bound_transport(&registry, &configuration, "0.0.0.0:37175");

        let diagnostic = registry
            .diagnostic(NativeTransportKind::HttpRequestResponse)
            .expect("http diagnostic");
        assert_eq!(diagnostic.phase, NativeTransportPhase::Ready);
        assert_eq!(diagnostic.bind_target.as_deref(), Some("0.0.0.0:37175"));
    }

    #[test]
    fn record_transport_failure_captures_latest_error() {
        let configuration = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: Some("127.0.0.1:4100".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let registry = NativeTransportRegistry::new(NativeTransportConfigurations {
            http_request_response: configuration.clone(),
            ..NativeTransportConfigurations::default()
        });

        record_transport_failure(&registry, &configuration, "address already in use");

        let diagnostic = registry
            .diagnostic(NativeTransportKind::HttpRequestResponse)
            .expect("http diagnostic");
        assert_eq!(diagnostic.phase, NativeTransportPhase::Failed);
        assert_eq!(
            diagnostic.last_error.as_deref(),
            Some("address already in use")
        );
    }

    #[test]
    fn resolve_shared_web_bind_target_uses_sse_target_when_http_is_disabled() {
        let http = NativeTransportConfiguration::for_kind(NativeTransportKind::HttpRequestResponse);
        let sse = NativeTransportConfiguration {
            transport: NativeTransportKind::ServerSentEvents,
            enabled: true,
            bind_target: Some("127.0.0.1:4200".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let websocket = NativeTransportConfiguration::for_kind(NativeTransportKind::WebSocket);
        let transit = NativeTransportConfiguration::for_kind(NativeTransportKind::Transit);

        let bind_target =
            resolve_shared_web_bind_target(&http, &sse, &websocket, &transit, "0.0.0.0:3000")
                .expect("bind target");

        assert_eq!(bind_target, "127.0.0.1:4200");
    }

    #[test]
    fn resolve_shared_web_bind_target_rejects_conflicting_http_and_sse_targets() {
        let http = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: Some("127.0.0.1:4100".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let sse = NativeTransportConfiguration {
            transport: NativeTransportKind::ServerSentEvents,
            enabled: true,
            bind_target: Some("127.0.0.1:4200".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let websocket = NativeTransportConfiguration::for_kind(NativeTransportKind::WebSocket);
        let transit = NativeTransportConfiguration::for_kind(NativeTransportKind::Transit);

        let error =
            resolve_shared_web_bind_target(&http, &sse, &websocket, &transit, "0.0.0.0:3000")
                .expect_err("conflicting authored bind targets should fail closed");

        assert!(error.contains("http_request_response"));
        assert!(error.contains("server_sent_events"));
    }

    #[test]
    fn resolve_shared_web_bind_target_rejects_conflicting_http_and_websocket_targets() {
        let http = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: Some("127.0.0.1:4100".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let sse = NativeTransportConfiguration::for_kind(NativeTransportKind::ServerSentEvents);
        let websocket = NativeTransportConfiguration {
            transport: NativeTransportKind::WebSocket,
            enabled: true,
            bind_target: Some("127.0.0.1:4200".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let transit = NativeTransportConfiguration::for_kind(NativeTransportKind::Transit);

        let error =
            resolve_shared_web_bind_target(&http, &sse, &websocket, &transit, "0.0.0.0:3000")
                .expect_err("conflicting authored bind targets should fail closed");

        assert!(error.contains("http_request_response"));
        assert!(error.contains("websocket"));
    }

    #[test]
    fn resolve_shared_web_bind_target_rejects_conflicting_http_and_transit_targets() {
        let http = NativeTransportConfiguration {
            transport: NativeTransportKind::HttpRequestResponse,
            enabled: true,
            bind_target: Some("127.0.0.1:4100".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let sse = NativeTransportConfiguration::for_kind(NativeTransportKind::ServerSentEvents);
        let websocket = NativeTransportConfiguration::for_kind(NativeTransportKind::WebSocket);
        let transit = NativeTransportConfiguration {
            transport: NativeTransportKind::Transit,
            enabled: true,
            bind_target: Some("127.0.0.1:4400".to_string()),
            auth: NativeTransportAuth::default(),
        };

        let error =
            resolve_shared_web_bind_target(&http, &sse, &websocket, &transit, "0.0.0.0:3000")
                .expect_err("conflicting authored bind targets should fail closed");

        assert!(error.contains("http_request_response"));
        assert!(error.contains("transit"));
    }

    #[test]
    fn record_degraded_clears_websocket_session_and_preserves_bind_target() {
        let websocket = NativeTransportConfiguration {
            transport: NativeTransportKind::WebSocket,
            enabled: true,
            bind_target: Some("127.0.0.1:4300".to_string()),
            auth: NativeTransportAuth::default(),
        };
        let registry = NativeTransportRegistry::new(NativeTransportConfigurations {
            websocket: websocket.clone(),
            ..NativeTransportConfigurations::default()
        });
        registry.record_session(
            NativeTransportKind::WebSocket,
            NativeTransportSessionIdentity {
                transport: NativeTransportKind::WebSocket,
                task_id: crate::domain::model::TaskTraceId::new("task-websocket").expect("task id"),
                channel: crate::domain::model::NativeTransportChannel::ConversationSession,
                connection_id: Some("socket-1".to_string()),
            },
        );

        registry.record_degraded(
            NativeTransportKind::WebSocket,
            "websocket transport expects UTF-8 text prompt frames",
        );

        let diagnostic = registry
            .diagnostic(NativeTransportKind::WebSocket)
            .expect("websocket diagnostic");
        assert_eq!(diagnostic.phase, NativeTransportPhase::Degraded);
        assert_eq!(diagnostic.bind_target.as_deref(), Some("127.0.0.1:4300"));
        assert_eq!(diagnostic.session, None);
        assert_eq!(
            diagnostic.last_error.as_deref(),
            Some("websocket transport expects UTF-8 text prompt frames")
        );
    }
}
