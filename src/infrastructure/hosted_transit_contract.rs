use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};

const HOSTED_TRANSIT_CONTRACT_VERSION: &str = "paddles.hosted.transit.v1";

pub fn hosted_transit_contract_version() -> &'static str {
    HOSTED_TRANSIT_CONTRACT_VERSION
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostedTransitEnvelopeKind {
    BootstrapCommand,
    TurnSubmissionCommand,
    TurnProgressEvent,
    ProjectionRebuildEvent,
    TurnCompletionEvent,
    TurnFailureEvent,
    RestoreCommand,
    SessionProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitEnvelope<T> {
    pub contract_version: String,
    pub kind: HostedTransitEnvelopeKind,
    pub payload: T,
}

impl<T> HostedTransitEnvelope<T> {
    pub fn new(kind: HostedTransitEnvelopeKind, payload: T) -> Self {
        Self {
            contract_version: HOSTED_TRANSIT_CONTRACT_VERSION.to_string(),
            kind,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitStreamFamilies {
    pub bootstrap_commands: String,
    pub turn_commands: String,
    pub restore_commands: String,
    pub progress_events: String,
    pub completion_events: String,
    pub failure_events: String,
    pub projection_rebuild_events: String,
    pub session_projection: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitStreamLayout {
    pub contract_version: String,
    pub namespace: String,
    pub service_identity: String,
    pub families: HostedTransitStreamFamilies,
}

impl HostedTransitStreamLayout {
    pub fn for_service(namespace: &str, service_identity: &str) -> Result<Self> {
        ensure!(
            !namespace.trim().is_empty(),
            "hosted transit contract namespace must not be empty"
        );
        ensure!(
            !service_identity.trim().is_empty(),
            "hosted transit contract service identity must not be empty"
        );

        let namespace = sanitize_stream_component(namespace);
        let service_identity = sanitize_stream_component(service_identity);
        let root = format!("{namespace}.paddles.hosted.{service_identity}");

        Ok(Self {
            contract_version: HOSTED_TRANSIT_CONTRACT_VERSION.to_string(),
            namespace,
            service_identity,
            families: HostedTransitStreamFamilies {
                bootstrap_commands: format!("{root}.command.bootstrap"),
                turn_commands: format!("{root}.command.turn"),
                restore_commands: format!("{root}.command.restore"),
                progress_events: format!("{root}.event.progress"),
                completion_events: format!("{root}.event.completion"),
                failure_events: format!("{root}.event.failure"),
                projection_rebuild_events: format!("{root}.event.projection-rebuild"),
                session_projection: format!("{root}.projection.session"),
            },
        })
    }
}

fn sanitize_stream_component(component: &str) -> String {
    component
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        HostedTransitEnvelope, HostedTransitEnvelopeKind, HostedTransitStreamFamilies,
        HostedTransitStreamLayout, hosted_transit_contract_version,
    };

    #[test]
    fn hosted_transit_contract_versions_define_envelopes_for_bootstrap_turn_progress_rebuild_completion_failure_and_restore()
     {
        let version = hosted_transit_contract_version();
        assert_eq!(version, "paddles.hosted.transit.v1");

        let kinds = [
            HostedTransitEnvelopeKind::BootstrapCommand,
            HostedTransitEnvelopeKind::TurnSubmissionCommand,
            HostedTransitEnvelopeKind::TurnProgressEvent,
            HostedTransitEnvelopeKind::ProjectionRebuildEvent,
            HostedTransitEnvelopeKind::TurnCompletionEvent,
            HostedTransitEnvelopeKind::TurnFailureEvent,
            HostedTransitEnvelopeKind::RestoreCommand,
        ];

        for kind in kinds {
            let envelope = HostedTransitEnvelope::new(kind.clone(), serde_json::json!({"ok":true}));
            assert_eq!(envelope.contract_version, version);
            assert_eq!(envelope.kind, kind);
        }
    }

    #[test]
    fn hosted_transit_stream_families_define_runtime_layout() {
        let layout = HostedTransitStreamLayout::for_service("prod", "paddles-primary")
            .expect("stream layout");

        assert_eq!(
            layout.families,
            HostedTransitStreamFamilies {
                bootstrap_commands: "prod.paddles.hosted.paddles-primary.command.bootstrap"
                    .to_string(),
                turn_commands: "prod.paddles.hosted.paddles-primary.command.turn".to_string(),
                restore_commands: "prod.paddles.hosted.paddles-primary.command.restore".to_string(),
                progress_events: "prod.paddles.hosted.paddles-primary.event.progress".to_string(),
                completion_events: "prod.paddles.hosted.paddles-primary.event.completion"
                    .to_string(),
                failure_events: "prod.paddles.hosted.paddles-primary.event.failure".to_string(),
                projection_rebuild_events:
                    "prod.paddles.hosted.paddles-primary.event.projection-rebuild".to_string(),
                session_projection: "prod.paddles.hosted.paddles-primary.projection.session"
                    .to_string(),
            }
        );
        assert_eq!(
            layout.contract_version,
            hosted_transit_contract_version().to_string()
        );
    }
}
