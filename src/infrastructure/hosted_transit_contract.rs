use crate::application::read_model::{
    ConversationProjectionSnapshot, ConversationTranscriptSpeaker, ForensicLifecycle,
};
use crate::domain::model::TraceReplay;
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
    pub provenance: HostedTransitProvenance,
    pub payload: T,
}

impl<T> HostedTransitEnvelope<T> {
    pub fn new(
        kind: HostedTransitEnvelopeKind,
        provenance: HostedTransitProvenance,
        payload: T,
    ) -> Result<Self> {
        provenance.validate()?;
        Ok(Self {
            contract_version: HOSTED_TRANSIT_CONTRACT_VERSION.to_string(),
            kind,
            provenance,
            payload,
        })
    }

    pub fn validate(&self) -> Result<()> {
        validate_hosted_transit_contract_version(&self.contract_version)?;
        self.provenance.validate()?;
        Ok(())
    }
}

impl HostedTransitEnvelope<HostedTransitSessionProjectionPayload> {
    pub fn session_projection(
        snapshot: &ConversationProjectionSnapshot,
        provenance: HostedTransitProvenance,
    ) -> Result<Self> {
        let payload =
            HostedTransitSessionProjectionPayload::from_projection_snapshot(snapshot, &provenance);
        Self::new(
            HostedTransitEnvelopeKind::SessionProjection,
            provenance,
            payload,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitProvenance {
    pub account_id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub route: String,
    pub request_id: String,
    pub workspace_posture: String,
}

impl HostedTransitProvenance {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.account_id.trim().is_empty(),
            "hosted transit provenance requires account_id"
        );
        ensure!(
            !self.session_id.trim().is_empty(),
            "hosted transit provenance requires session_id"
        );
        ensure!(
            !self.workspace_id.trim().is_empty(),
            "hosted transit provenance requires workspace_id"
        );
        ensure!(
            !self.route.trim().is_empty(),
            "hosted transit provenance requires route"
        );
        ensure!(
            !self.request_id.trim().is_empty(),
            "hosted transit provenance requires request_id"
        );
        ensure!(
            !self.workspace_posture.trim().is_empty(),
            "hosted transit provenance requires workspace_posture"
        );
        Ok(())
    }
}

pub fn validate_hosted_transit_contract_version(version: &str) -> Result<()> {
    ensure!(
        version == HOSTED_TRANSIT_CONTRACT_VERSION,
        "unsupported hosted transit contract version: {version}"
    );
    Ok(())
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
pub struct HostedTransitSessionProjectionPayload {
    pub task_id: String,
    pub replay_revision: u64,
    pub transcript_rows: Vec<HostedTransitTranscriptRow>,
    pub turn_statuses: Vec<HostedTransitTurnStatus>,
    pub detail_availability: HostedTransitDetailAvailability,
    pub restore_context: HostedTransitRestoreContext,
}

impl HostedTransitSessionProjectionPayload {
    pub fn from_trace_replay(replay: &TraceReplay, provenance: &HostedTransitProvenance) -> Self {
        let snapshot = ConversationProjectionSnapshot::from_trace_replay(replay);
        Self::from_projection_snapshot(&snapshot, provenance)
    }

    pub fn from_projection_snapshot(
        snapshot: &ConversationProjectionSnapshot,
        provenance: &HostedTransitProvenance,
    ) -> Self {
        Self {
            task_id: snapshot.task_id.as_str().to_string(),
            replay_revision: snapshot.version(),
            transcript_rows: snapshot
                .transcript
                .entries
                .iter()
                .map(|entry| HostedTransitTranscriptRow {
                    record_id: entry.record_id.as_str().to_string(),
                    turn_id: entry.turn_id.as_str().to_string(),
                    speaker: transcript_speaker_label(entry.speaker).to_string(),
                    content: entry.content.clone(),
                    response_mode: entry.response_mode.map(|mode| mode.label().to_string()),
                    citations: entry.citations.clone(),
                    grounded: entry.grounded,
                    render_present: entry.render.is_some(),
                })
                .collect(),
            turn_statuses: snapshot
                .forensics
                .turns
                .iter()
                .map(|turn| HostedTransitTurnStatus {
                    turn_id: turn.turn_id.as_str().to_string(),
                    status: forensic_lifecycle_label(turn.lifecycle).to_string(),
                })
                .collect(),
            detail_availability: HostedTransitDetailAvailability {
                trace_graph_available: !snapshot.trace_graph.nodes.is_empty(),
                manifold_available: !snapshot.manifold.turns.is_empty(),
                forensic_available: !snapshot.forensics.turns.is_empty(),
                delegation_available: !snapshot.delegation.workers.is_empty(),
            },
            restore_context: HostedTransitRestoreContext {
                can_restore: true,
                account_id: provenance.account_id.clone(),
                session_id: provenance.session_id.clone(),
                workspace_id: provenance.workspace_id.clone(),
                route: provenance.route.clone(),
                request_id: provenance.request_id.clone(),
                workspace_posture: provenance.workspace_posture.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitTranscriptRow {
    pub record_id: String,
    pub turn_id: String,
    pub speaker: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub citations: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grounded: Option<bool>,
    pub render_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitTurnStatus {
    pub turn_id: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitDetailAvailability {
    pub trace_graph_available: bool,
    pub manifold_available: bool,
    pub forensic_available: bool,
    pub delegation_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HostedTransitRestoreContext {
    pub can_restore: bool,
    pub account_id: String,
    pub session_id: String,
    pub workspace_id: String,
    pub route: String,
    pub request_id: String,
    pub workspace_posture: String,
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

fn transcript_speaker_label(speaker: ConversationTranscriptSpeaker) -> &'static str {
    match speaker {
        ConversationTranscriptSpeaker::User => "user",
        ConversationTranscriptSpeaker::Assistant => "assistant",
        ConversationTranscriptSpeaker::System => "system",
    }
}

fn forensic_lifecycle_label(lifecycle: ForensicLifecycle) -> &'static str {
    match lifecycle {
        ForensicLifecycle::Provisional => "provisional",
        ForensicLifecycle::Superseded => "superseded",
        ForensicLifecycle::Final => "final",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        HostedTransitEnvelope, HostedTransitEnvelopeKind, HostedTransitProvenance,
        HostedTransitStreamFamilies, HostedTransitStreamLayout, hosted_transit_contract_version,
    };
    use crate::application::read_model::ConversationProjectionSnapshot;
    use crate::domain::model::render::RenderDocument;
    use crate::domain::model::{
        ArtifactEnvelope, ArtifactKind, AuthoredResponse, ResponseMode, TaskTraceId,
        TraceCheckpointKind, TraceCompletionCheckpoint, TraceHarnessProfileSelection, TraceLineage,
        TraceRecord, TraceRecordId, TraceRecordKind, TraceReplay, TraceTaskRoot, TurnTraceId,
    };
    use paddles_conversation::{TraceArtifactId, TraceCheckpointId};

    fn sample_provenance() -> HostedTransitProvenance {
        HostedTransitProvenance {
            account_id: "acct-1".to_string(),
            session_id: "session-1".to_string(),
            workspace_id: "workspace-1".to_string(),
            route: "hub/workbench".to_string(),
            request_id: "request-1".to_string(),
            workspace_posture: "workspace_write".to_string(),
        }
    }

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
            let envelope = HostedTransitEnvelope::new(
                kind.clone(),
                sample_provenance(),
                serde_json::json!({"ok":true}),
            )
            .expect("versioned envelope");
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

    #[test]
    fn transit_provenance_envelopes_carry_account_session_workspace_route_request_and_posture() {
        let provenance = sample_provenance();
        let envelope = HostedTransitEnvelope::new(
            HostedTransitEnvelopeKind::SessionProjection,
            provenance.clone(),
            serde_json::json!({"projection":"session"}),
        )
        .expect("projection envelope");

        assert_eq!(envelope.provenance, provenance);
        assert_eq!(envelope.provenance.account_id, "acct-1");
        assert_eq!(envelope.provenance.session_id, "session-1");
        assert_eq!(envelope.provenance.workspace_id, "workspace-1");
        assert_eq!(envelope.provenance.route, "hub/workbench");
        assert_eq!(envelope.provenance.request_id, "request-1");
        assert_eq!(envelope.provenance.workspace_posture, "workspace_write");
    }

    #[test]
    fn transit_contract_rejects_missing_provenance() {
        let error = HostedTransitEnvelope::new(
            HostedTransitEnvelopeKind::TurnSubmissionCommand,
            HostedTransitProvenance {
                account_id: String::new(),
                session_id: "session-1".to_string(),
                workspace_id: "workspace-1".to_string(),
                route: "hub/workbench".to_string(),
                request_id: "request-1".to_string(),
                workspace_posture: "workspace_write".to_string(),
            },
            serde_json::json!({"prompt":"fix it"}),
        )
        .expect_err("missing provenance should reject");

        assert!(error.to_string().contains("account_id"));
    }

    #[test]
    fn consumer_projection_payloads_include_transcript_status_and_revision_metadata() {
        let replay = sample_trace_replay();
        let payload = super::HostedTransitSessionProjectionPayload::from_trace_replay(
            &replay,
            &sample_provenance(),
        );

        assert_eq!(payload.task_id, "task-1");
        assert_eq!(payload.replay_revision, 3);
        assert_eq!(payload.transcript_rows.len(), 2);
        assert_eq!(payload.transcript_rows[0].speaker, "user");
        assert_eq!(payload.transcript_rows[1].speaker, "assistant");
        assert_eq!(payload.turn_statuses.len(), 1);
        assert_eq!(payload.turn_statuses[0].status, "final");
        assert!(payload.detail_availability.trace_graph_available);
        assert!(payload.detail_availability.manifold_available);
        assert!(payload.restore_context.can_restore);
        assert_eq!(payload.restore_context.session_id, "session-1");
        assert_eq!(payload.restore_context.workspace_id, "workspace-1");
    }

    #[test]
    fn transit_projection_payloads_remain_replay_derived() {
        let replay = sample_trace_replay();
        let snapshot = ConversationProjectionSnapshot::from_trace_replay(&replay);
        let provenance = sample_provenance();

        let from_replay =
            super::HostedTransitSessionProjectionPayload::from_trace_replay(&replay, &provenance);
        let from_snapshot = super::HostedTransitSessionProjectionPayload::from_projection_snapshot(
            &snapshot,
            &provenance,
        );

        assert_eq!(from_replay, from_snapshot);
    }

    #[test]
    fn consumer_projection_payloads_remain_replay_derived_views() {
        let replay = sample_trace_replay();
        let provenance = sample_provenance();
        let payload =
            super::HostedTransitSessionProjectionPayload::from_trace_replay(&replay, &provenance);

        assert_eq!(
            payload
                .transcript_rows
                .iter()
                .map(|row| row.content.as_str())
                .collect::<Vec<_>>(),
            vec!["Need restore context", "Fixed"]
        );
        assert_eq!(
            payload
                .turn_statuses
                .iter()
                .map(|status| (status.turn_id.as_str(), status.status.as_str()))
                .collect::<Vec<_>>(),
            vec![("task-1.turn-0001", "final")]
        );
    }

    #[test]
    fn hosted_transit_contract_rejects_unsupported_versions() {
        let envelope = serde_json::json!({
            "contract_version": "paddles.hosted.transit.v999",
            "kind": "session_projection",
            "provenance": sample_provenance(),
            "payload": {"projection":"session"},
        });
        let parsed: HostedTransitEnvelope<serde_json::Value> =
            serde_json::from_value(envelope).expect("deserialize envelope");

        let error = parsed
            .validate()
            .expect_err("unsupported version should reject");
        assert!(
            error
                .to_string()
                .contains("unsupported hosted transit contract version")
        );
    }

    fn sample_trace_replay() -> TraceReplay {
        let task_id = TaskTraceId::new("task-1").expect("task");
        let turn_id = TurnTraceId::new("task-1.turn-0001").expect("turn");

        TraceReplay {
            task_id: task_id.clone(),
            records: vec![
                TraceRecord {
                    record_id: TraceRecordId::new("task-1.turn-0001.record-0001").expect("record"),
                    sequence: 1,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: None,
                    },
                    kind: TraceRecordKind::TaskRootStarted(TraceTaskRoot {
                        prompt: ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-1").expect("artifact"),
                            ArtifactKind::Prompt,
                            "prompt",
                            "Need restore context",
                            usize::MAX,
                        ),
                        interpretation: None,
                        planner_model: "planner".to_string(),
                        synthesizer_model: "synth".to_string(),
                        harness_profile: TraceHarnessProfileSelection {
                            requested_profile_id: "recursive-structured-v1".to_string(),
                            active_profile_id: "recursive-structured-v1".to_string(),
                            downgrade_reason: None,
                        },
                    }),
                },
                TraceRecord {
                    record_id: TraceRecordId::new("task-1.turn-0001.record-0002").expect("record"),
                    sequence: 2,
                    lineage: TraceLineage {
                        task_id: task_id.clone(),
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("task-1.turn-0001.record-0001").expect("parent"),
                        ),
                    },
                    kind: TraceRecordKind::PlannerAction {
                        action: "read `README.md`".to_string(),
                        rationale: "Gather context".to_string(),
                        signal_summary: None,
                    },
                },
                TraceRecord {
                    record_id: TraceRecordId::new("task-1.turn-0001.record-0003").expect("record"),
                    sequence: 3,
                    lineage: TraceLineage {
                        task_id,
                        turn_id: turn_id.clone(),
                        branch_id: None,
                        parent_record_id: Some(
                            TraceRecordId::new("task-1.turn-0001.record-0002").expect("parent"),
                        ),
                    },
                    kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                        checkpoint_id: TraceCheckpointId::new("checkpoint-1").expect("checkpoint"),
                        kind: TraceCheckpointKind::TurnCompleted,
                        summary: "turn completed".to_string(),
                        response: Some(ArtifactEnvelope::text(
                            TraceArtifactId::new("artifact-2").expect("artifact"),
                            ArtifactKind::ModelOutput,
                            "assistant response",
                            "Fixed",
                            200,
                        )),
                        authored_response: Some(AuthoredResponse {
                            mode: ResponseMode::DirectAnswer,
                            document: RenderDocument::from_assistant_plain_text("Fixed"),
                        }),
                        citations: Vec::new(),
                        grounded: false,
                    }),
                },
            ],
        }
    }
}
