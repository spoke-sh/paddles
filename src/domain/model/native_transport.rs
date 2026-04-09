use crate::domain::model::TaskTraceId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeTransportKind {
    HttpRequestResponse,
    ServerSentEvents,
    WebSocket,
    Transit,
}

impl NativeTransportKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::HttpRequestResponse => "http_request_response",
            Self::ServerSentEvents => "server_sent_events",
            Self::WebSocket => "websocket",
            Self::Transit => "transit",
        }
    }

    pub fn default_capabilities(self) -> Vec<NativeTransportCapability> {
        match self {
            Self::HttpRequestResponse => vec![NativeTransportCapability::RequestResponse],
            Self::ServerSentEvents => vec![
                NativeTransportCapability::ServerPush,
                NativeTransportCapability::SessionScoped,
            ],
            Self::WebSocket => vec![
                NativeTransportCapability::RequestResponse,
                NativeTransportCapability::ServerPush,
                NativeTransportCapability::Bidirectional,
                NativeTransportCapability::SessionScoped,
            ],
            Self::Transit => vec![
                NativeTransportCapability::RequestResponse,
                NativeTransportCapability::StructuredPayload,
            ],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeTransportCapability {
    RequestResponse,
    ServerPush,
    Bidirectional,
    SessionScoped,
    StructuredPayload,
}

impl NativeTransportCapability {
    pub fn label(self) -> &'static str {
        match self {
            Self::RequestResponse => "request_response",
            Self::ServerPush => "server_push",
            Self::Bidirectional => "bidirectional",
            Self::SessionScoped => "session_scoped",
            Self::StructuredPayload => "structured_payload",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeTransportPhase {
    Disabled,
    Configured,
    Binding,
    Ready,
    Degraded,
    Failed,
}

impl NativeTransportPhase {
    pub fn label(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Configured => "configured",
            Self::Binding => "binding",
            Self::Ready => "ready",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
        }
    }

    pub fn is_connected(self) -> bool {
        matches!(self, Self::Ready | Self::Degraded)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NativeTransportChannel {
    TurnRequestResponse,
    TurnEventStream,
    ConversationSession,
    TransitExchange,
}

impl NativeTransportChannel {
    pub fn label(self) -> &'static str {
        match self {
            Self::TurnRequestResponse => "turn_request_response",
            Self::TurnEventStream => "turn_event_stream",
            Self::ConversationSession => "conversation_session",
            Self::TransitExchange => "transit_exchange",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeTransportSessionIdentity {
    pub transport: NativeTransportKind,
    pub task_id: TaskTraceId,
    pub channel: NativeTransportChannel,
    pub connection_id: Option<String>,
}

impl NativeTransportSessionIdentity {
    pub fn stable_key(&self) -> String {
        match self.connection_id.as_deref() {
            Some(connection_id) if !connection_id.trim().is_empty() => format!(
                "{}:{}:{}:{}",
                self.transport.label(),
                self.task_id.as_str(),
                self.channel.label(),
                connection_id
            ),
            _ => format!(
                "{}:{}:{}",
                self.transport.label(),
                self.task_id.as_str(),
                self.channel.label()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_transport_kinds_publish_stable_labels_and_default_capabilities() {
        assert_eq!(
            NativeTransportKind::HttpRequestResponse.label(),
            "http_request_response"
        );
        assert_eq!(
            NativeTransportKind::ServerSentEvents.default_capabilities(),
            vec![
                NativeTransportCapability::ServerPush,
                NativeTransportCapability::SessionScoped,
            ]
        );
        assert_eq!(
            NativeTransportKind::WebSocket.default_capabilities(),
            vec![
                NativeTransportCapability::RequestResponse,
                NativeTransportCapability::ServerPush,
                NativeTransportCapability::Bidirectional,
                NativeTransportCapability::SessionScoped,
            ]
        );
        assert_eq!(
            NativeTransportKind::Transit.default_capabilities(),
            vec![
                NativeTransportCapability::RequestResponse,
                NativeTransportCapability::StructuredPayload,
            ]
        );
    }

    #[test]
    fn native_transport_phases_report_readiness_semantics() {
        assert!(!NativeTransportPhase::Configured.is_connected());
        assert!(NativeTransportPhase::Ready.is_connected());
        assert!(NativeTransportPhase::Degraded.is_connected());
        assert_eq!(NativeTransportPhase::Failed.label(), "failed");
    }

    #[test]
    fn native_transport_session_identity_builds_stable_connection_keys() {
        let task_id = TaskTraceId::new("task-transport").expect("task id");
        let identity = NativeTransportSessionIdentity {
            transport: NativeTransportKind::WebSocket,
            task_id: task_id.clone(),
            channel: NativeTransportChannel::ConversationSession,
            connection_id: Some("socket-7".to_string()),
        };
        assert_eq!(
            identity.stable_key(),
            "websocket:task-transport:conversation_session:socket-7"
        );

        let identity_without_connection = NativeTransportSessionIdentity {
            transport: NativeTransportKind::ServerSentEvents,
            task_id,
            channel: NativeTransportChannel::TurnEventStream,
            connection_id: None,
        };
        assert_eq!(
            identity_without_connection.stable_key(),
            "server_sent_events:task-transport:turn_event_stream"
        );
    }
}
