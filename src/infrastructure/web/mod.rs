use crate::application::MechSuitService;
use crate::domain::model::{
    ConversationForensicProjection, ConversationForensicUpdate, ConversationTranscript,
    ConversationTranscriptUpdate, ForensicLifecycle, ForensicRecordProjection,
    ForensicTurnProjection, ForensicUpdateSink, TaskTraceId, TraceRecordKind, TranscriptUpdateSink,
    TurnEvent, TurnEventSink, TurnTraceId,
};
use crate::domain::ports::TraceRecorder;
use axum::Router;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, Json};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;

struct AppState {
    service: Arc<MechSuitService>,
    trace_recorder: Arc<dyn TraceRecorder>,
    event_tx: broadcast::Sender<(String, TurnEvent)>,
    transcript_tx: broadcast::Sender<ConversationTranscriptUpdate>,
    forensic_tx: broadcast::Sender<ConversationForensicUpdate>,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: String,
}

#[derive(Deserialize)]
struct TurnRequest {
    prompt: String,
}

#[derive(Serialize)]
struct TurnResponse {
    response: String,
}

#[derive(Serialize)]
struct TranscriptResponse(ConversationTranscript);

#[derive(Serialize)]
struct ForensicProjectionResponse(ConversationForensicProjection);

#[derive(Serialize)]
struct ForensicTurnProjectionResponse(ForensicTurnProjection);

#[derive(Serialize)]
struct ForensicUpdateEventResponse {
    update: ConversationForensicUpdate,
    turn_lifecycle: Option<ForensicLifecycle>,
    record: Option<ForensicRecordProjection>,
}

#[derive(Serialize)]
struct TraceGraphResponse {
    nodes: Vec<TraceGraphNode>,
    edges: Vec<TraceGraphEdge>,
    branches: Vec<TraceGraphBranch>,
}

#[derive(Serialize)]
struct TraceGraphNode {
    id: String,
    kind: String,
    label: String,
    branch_id: Option<String>,
    sequence: u64,
}

#[derive(Serialize)]
struct TraceGraphEdge {
    from: String,
    to: String,
}

#[derive(Serialize)]
struct TraceGraphBranch {
    id: String,
    label: String,
    status: String,
    parent_branch_id: Option<String>,
}

struct BroadcastEventSink {
    session_id: String,
    tx: broadcast::Sender<(String, TurnEvent)>,
}

impl TurnEventSink for BroadcastEventSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.tx.send((self.session_id.clone(), event));
    }
}

/// Create the web router and return it along with a TurnEventSink that
/// broadcasts events to all connected SSE clients. Register this sink
/// as an event observer on MechSuitService so CLI turns are visible too.
pub fn router(
    service: Arc<MechSuitService>,
    trace_recorder: Arc<dyn TraceRecorder>,
) -> (Router, Arc<dyn TurnEventSink>) {
    let (event_tx, _) = broadcast::channel::<(String, TurnEvent)>(256);
    let (transcript_tx, _) = broadcast::channel::<ConversationTranscriptUpdate>(256);
    let (forensic_tx, _) = broadcast::channel::<ConversationForensicUpdate>(512);
    let observer: Arc<dyn TurnEventSink> = Arc::new(GlobalBroadcastSink {
        tx: event_tx.clone(),
    });
    service.register_transcript_observer(Arc::new(BroadcastTranscriptSink {
        tx: transcript_tx.clone(),
    }));
    service.register_forensic_observer(Arc::new(BroadcastForensicSink {
        tx: forensic_tx.clone(),
    }));
    let state = Arc::new(AppState {
        service,
        trace_recorder,
        event_tx,
        transcript_tx,
        forensic_tx,
    });

    let app = Router::new()
        .route("/", get(index_page))
        .route("/health", get(health))
        .route("/sessions", post(create_session))
        .route("/sessions/{id}/turns", post(submit_turn))
        .route("/sessions/{id}/transcript", get(conversation_transcript))
        .route(
            "/sessions/{id}/transcript/events",
            get(conversation_transcript_event_stream),
        )
        .route("/sessions/{id}/forensics", get(conversation_forensics))
        .route(
            "/sessions/{id}/forensics/turns/{turn_id}",
            get(turn_forensics),
        )
        .route(
            "/sessions/{id}/forensics/events",
            get(conversation_forensic_event_stream),
        )
        .route("/sessions/{id}/events", get(event_stream))
        .route("/events", get(global_event_stream))
        .route("/sessions/{id}/trace/graph", get(trace_graph))
        .route("/trace/graph", get(trace_graph_all))
        .route("/trace/replay", get(trace_replay_all))
        .layer(CorsLayer::permissive())
        .with_state(state);

    (app, observer)
}

/// Broadcasts all events to the SSE channel regardless of session.
/// Used as a global observer on MechSuitService so CLI turns are visible.
struct GlobalBroadcastSink {
    tx: broadcast::Sender<(String, TurnEvent)>,
}

impl TurnEventSink for GlobalBroadcastSink {
    fn emit(&self, event: TurnEvent) {
        // Broadcast with empty session_id — SSE filtering uses session-specific sinks.
        // The per-session BroadcastEventSink tags with the correct session_id.
        let _ = self.tx.send((String::new(), event));
    }
}

struct BroadcastTranscriptSink {
    tx: broadcast::Sender<ConversationTranscriptUpdate>,
}

impl TranscriptUpdateSink for BroadcastTranscriptSink {
    fn emit(&self, update: ConversationTranscriptUpdate) {
        let _ = self.tx.send(update);
    }
}

struct BroadcastForensicSink {
    tx: broadcast::Sender<ConversationForensicUpdate>,
}

impl ForensicUpdateSink for BroadcastForensicSink {
    fn emit(&self, update: ConversationForensicUpdate) {
        let _ = self.tx.send(update);
    }
}

async fn index_page() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn create_session(State(state): State<Arc<AppState>>) -> Json<SessionResponse> {
    let session = state.service.shared_conversation_session();
    let session_id = session.task_id().as_str().to_string();
    Json(SessionResponse { session_id })
}

async fn submit_turn(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<TurnRequest>,
) -> Result<Json<TurnResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let session = state
        .service
        .conversation_session(&task_id)
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;

    let sink: Arc<dyn TurnEventSink> = Arc::new(BroadcastEventSink {
        session_id: id,
        tx: state.event_tx.clone(),
    });

    let response = state
        .service
        .process_prompt_in_session_with_sink(&body.prompt, session, sink)
        .await
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(TurnResponse { response }))
}

async fn conversation_transcript(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TranscriptResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let transcript = state
        .service
        .replay_conversation_transcript(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(TranscriptResponse(transcript)))
}

async fn conversation_forensics(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ForensicProjectionResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let projection = state
        .service
        .replay_conversation_forensics(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ForensicProjectionResponse(projection)))
}

async fn turn_forensics(
    State(state): State<Arc<AppState>>,
    Path((id, turn_id)): Path<(String, String)>,
) -> Result<Json<ForensicTurnProjectionResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let turn_id = parse_turn_id(&turn_id)?;
    let turn = state
        .service
        .replay_turn_forensics(&task_id, &turn_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(ForensicTurnProjectionResponse(turn)))
}

async fn conversation_transcript_event_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.transcript_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(update) if update.task_id.as_str() == id => {
            let json = serde_json::to_string(&update).unwrap_or_default();
            Some(Ok(Event::default().event("transcript_update").data(json)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn conversation_forensic_event_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.forensic_tx.subscribe();
    let state = Arc::clone(&state);
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(update) if update.task_id.as_str() == id => {
            let payload = state
                .service
                .replay_turn_forensics(&update.task_id, &update.turn_id)
                .ok()
                .flatten()
                .and_then(|turn| {
                    let turn_lifecycle = turn.lifecycle;
                    turn.records
                        .into_iter()
                        .find(|record| record.record.record_id == update.record_id)
                        .map(|record| ForensicUpdateEventResponse {
                            update: update.clone(),
                            turn_lifecycle: Some(turn_lifecycle),
                            record: Some(record),
                        })
                })
                .unwrap_or(ForensicUpdateEventResponse {
                    update,
                    turn_lifecycle: None,
                    record: None,
                });
            let json = serde_json::to_string(&payload).unwrap_or_default();
            Some(Ok(Event::default().event("forensic_update").data(json)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn event_stream(
    State(state): State<Arc<AppState>>,
    Path(_id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok((_session_id, event)) => {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().event("turn_event").data(json)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn global_event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok((_session_id, event)) => {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().event("turn_event").data(json)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn trace_graph(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<TraceGraphResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;

    let replay = state
        .trace_recorder
        .replay(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(build_trace_graph(&[replay])))
}

fn parse_task_id(id: &str) -> Result<TaskTraceId, axum::http::StatusCode> {
    TaskTraceId::new(id).map_err(|_| axum::http::StatusCode::BAD_REQUEST)
}

fn parse_turn_id(id: &str) -> Result<TurnTraceId, axum::http::StatusCode> {
    TurnTraceId::new(id).map_err(|_| axum::http::StatusCode::BAD_REQUEST)
}

async fn trace_replay_all(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<crate::domain::model::TraceReplay>> {
    let replays: Vec<_> = state
        .trace_recorder
        .task_ids()
        .iter()
        .filter_map(|id| state.trace_recorder.replay(id).ok())
        .filter(|r| !r.records.is_empty())
        .collect();
    Json(replays)
}

async fn trace_graph_all(State(state): State<Arc<AppState>>) -> Json<TraceGraphResponse> {
    let replays: Vec<_> = state
        .trace_recorder
        .task_ids()
        .iter()
        .filter_map(|id| state.trace_recorder.replay(id).ok())
        .collect();
    Json(build_trace_graph(&replays))
}

fn build_trace_graph(replays: &[crate::domain::model::TraceReplay]) -> TraceGraphResponse {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut branches = Vec::new();

    for replay in replays {
        for record in &replay.records {
            let (kind, label) = match &record.kind {
                TraceRecordKind::TaskRootStarted(root) => {
                    ("root".to_string(), root.planner_model.clone())
                }
                TraceRecordKind::TurnStarted(_) => ("turn".to_string(), "turn".to_string()),
                TraceRecordKind::PlannerAction { action, .. } => {
                    ("action".to_string(), truncate(action, 24))
                }
                TraceRecordKind::PlannerBranchDeclared(branch) => {
                    branches.push(TraceGraphBranch {
                        id: branch.branch_id.as_str().to_string(),
                        label: branch.label.clone(),
                        status: branch.status.label().to_string(),
                        parent_branch_id: branch
                            .parent_branch_id
                            .as_ref()
                            .map(|id| id.as_str().to_string()),
                    });
                    ("branch".to_string(), truncate(&branch.label, 24))
                }
                TraceRecordKind::ToolCallRequested(tool) => {
                    ("tool".to_string(), tool.tool_name.clone())
                }
                TraceRecordKind::ToolCallCompleted(tool) => {
                    ("tool_done".to_string(), tool.tool_name.clone())
                }
                TraceRecordKind::SelectionArtifact(sel) => {
                    ("evidence".to_string(), truncate(&sel.summary, 24))
                }
                TraceRecordKind::ModelExchangeArtifact(artifact) => (
                    "forensic".to_string(),
                    truncate(
                        &format!("{} {}", artifact.category.label(), artifact.phase.label()),
                        24,
                    ),
                ),
                TraceRecordKind::LineageEdge(edge) => {
                    ("lineage".to_string(), truncate(&edge.summary, 24))
                }
                TraceRecordKind::SignalSnapshot(signal) => (
                    "signal".to_string(),
                    truncate(&format!("{} {}", signal.kind.label(), signal.level), 24),
                ),
                TraceRecordKind::CompletionCheckpoint(cp) => {
                    ("checkpoint".to_string(), cp.kind.label().to_string())
                }
                TraceRecordKind::ThreadMerged(_) => ("merge".to_string(), "merge".to_string()),
                TraceRecordKind::ThreadCandidateCaptured(_) => {
                    ("thread".to_string(), "candidate".to_string())
                }
                TraceRecordKind::ThreadDecisionSelected(_) => {
                    ("thread".to_string(), "decision".to_string())
                }
            };

            nodes.push(TraceGraphNode {
                id: record.record_id.as_str().to_string(),
                kind,
                label,
                branch_id: record
                    .lineage
                    .branch_id
                    .as_ref()
                    .map(|id| id.as_str().to_string()),
                sequence: record.sequence,
            });

            if let Some(parent_id) = &record.lineage.parent_record_id {
                edges.push(TraceGraphEdge {
                    from: parent_id.as_str().to_string(),
                    to: record.record_id.as_str().to_string(),
                });
            }
        }
    }

    TraceGraphResponse {
        nodes,
        edges,
        branches,
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}...", &s[..n])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AppState, BroadcastForensicSink, ForensicProjectionResponse,
        ForensicTurnProjectionResponse, conversation_forensics, parse_task_id, turn_forensics,
    };
    use crate::application::MechSuitService;
    use crate::domain::model::{
        ArtifactKind, ConversationForensicUpdate, ForensicUpdateSink, TraceCheckpointKind,
        TraceCompletionCheckpoint, TraceLineage, TraceModelExchangeArtifact,
        TraceModelExchangeCategory, TraceModelExchangeLane, TraceModelExchangePhase, TraceRecord,
        TraceRecordId, TraceRecordKind, TraceReplay,
    };
    use crate::domain::ports::{ModelPaths, ModelRegistry, TraceRecorder};
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
    use anyhow::{Result, anyhow};
    use async_trait::async_trait;
    use axum::Json;
    use axum::extract::{Path, State};
    use paddles_conversation::{
        ArtifactEnvelope, ConversationSession, TraceArtifactId, TraceCheckpointId, TurnTraceId,
    };
    use std::path::Path as FsPath;
    use std::sync::Arc;
    use tokio::sync::broadcast;

    #[derive(Default)]
    struct StaticRegistry;

    #[async_trait]
    impl ModelRegistry for StaticRegistry {
        async fn get_model_paths(&self, _model_id: &str) -> Result<ModelPaths> {
            Err(anyhow!("test registry is not used in this suite"))
        }
    }

    fn test_service_with_recorder(
        workspace: &FsPath,
        recorder: Arc<dyn TraceRecorder>,
    ) -> Arc<MechSuitService> {
        let operator_memory = Arc::new(AgentMemory::load(workspace));
        Arc::new(MechSuitService::with_trace_recorder(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            Box::new(|_, _| Err(anyhow!("synthesizer factory not used in this test"))),
            Box::new(|_, _| Err(anyhow!("planner factory not used in this test"))),
            Box::new(|_, _, _, _| Err(anyhow!("gatherer factory not used in this test"))),
            recorder,
        ))
    }

    fn text_artifact(
        id: &str,
        kind: ArtifactKind,
        summary: &str,
        content: &str,
    ) -> ArtifactEnvelope {
        ArtifactEnvelope::text(
            TraceArtifactId::new(id).expect("artifact"),
            kind,
            summary,
            content,
            usize::MAX,
        )
        .with_mime_type("application/json")
    }

    fn seed_forensic_replay(
        recorder: &Arc<InMemoryTraceRecorder>,
        session: &ConversationSession,
    ) -> (TraceReplay, TraceRecordId) {
        let task_id = session.task_id();
        let turn_id = session.allocate_turn_id();
        let first_record_id =
            TraceRecordId::new(format!("{}.record-0001", turn_id.as_str())).expect("record");
        let second_record_id =
            TraceRecordId::new(format!("{}.record-0002", turn_id.as_str())).expect("record");
        let checkpoint_record_id =
            TraceRecordId::new(format!("{}.record-0003", turn_id.as_str())).expect("record");

        recorder
            .record(TraceRecord {
                record_id: first_record_id.clone(),
                sequence: 1,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: None,
                },
                kind: TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                    exchange_id: "exchange-1".to_string(),
                    lane: TraceModelExchangeLane::Planner,
                    category: TraceModelExchangeCategory::PlannerAction,
                    phase: TraceModelExchangePhase::AssembledContext,
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    parent_artifact_id: None,
                    artifact: text_artifact(
                        "artifact-1",
                        ArtifactKind::Prompt,
                        "first planner prompt",
                        "{\"step\":1}",
                    ),
                }),
            })
            .expect("record first artifact");
        recorder
            .record(TraceRecord {
                record_id: second_record_id.clone(),
                sequence: 2,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: Some(first_record_id.clone()),
                },
                kind: TraceRecordKind::ModelExchangeArtifact(TraceModelExchangeArtifact {
                    exchange_id: "exchange-2".to_string(),
                    lane: TraceModelExchangeLane::Planner,
                    category: TraceModelExchangeCategory::PlannerAction,
                    phase: TraceModelExchangePhase::AssembledContext,
                    provider: "openai".to_string(),
                    model: "gpt-5.4".to_string(),
                    parent_artifact_id: Some(TraceArtifactId::new("artifact-1").expect("artifact")),
                    artifact: text_artifact(
                        "artifact-2",
                        ArtifactKind::Prompt,
                        "second planner prompt",
                        "{\"step\":2}",
                    ),
                }),
            })
            .expect("record second artifact");
        recorder
            .record(TraceRecord {
                record_id: checkpoint_record_id,
                sequence: 3,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: Some(second_record_id.clone()),
                },
                kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                    checkpoint_id: TraceCheckpointId::new(format!(
                        "{}.checkpoint",
                        turn_id.as_str()
                    ))
                    .expect("checkpoint"),
                    kind: TraceCheckpointKind::TurnCompleted,
                    summary: "done".to_string(),
                    response: Some(text_artifact(
                        "artifact-3",
                        ArtifactKind::ModelOutput,
                        "final response",
                        "\"ok\"",
                    )),
                    citations: Vec::new(),
                    grounded: true,
                }),
            })
            .expect("record checkpoint");

        (recorder.replay(&task_id).expect("replay"), second_record_id)
    }

    #[tokio::test]
    async fn forensic_routes_project_conversation_and_turn_replay_with_lifecycle_states() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let (replay, latest_record_id) = seed_forensic_replay(&recorder, &session);
        let task_id = replay.task_id.clone();
        let turn_id = replay.records[0].lineage.turn_id.clone();
        let (event_tx, _) = broadcast::channel(8);
        let (transcript_tx, _) = broadcast::channel(8);
        let (forensic_tx, _) = broadcast::channel(8);
        let state = Arc::new(AppState {
            service,
            trace_recorder: recorder,
            event_tx,
            transcript_tx,
            forensic_tx,
        });

        let Json(ForensicProjectionResponse(conversation)) = conversation_forensics(
            State(Arc::clone(&state)),
            Path(task_id.as_str().to_string()),
        )
        .await
        .expect("conversation forensics");
        assert_eq!(conversation.task_id, task_id);
        assert_eq!(conversation.turns.len(), 1);
        assert_eq!(conversation.turns[0].turn_id, turn_id);
        assert!(
            conversation.turns[0]
                .records
                .iter()
                .any(|record| record.record.record_id == latest_record_id)
        );
        assert!(conversation.turns[0].records.iter().any(|record| matches!(
            record.lifecycle,
            crate::domain::model::ForensicLifecycle::Superseded
        )));
        assert!(conversation.turns[0].records.iter().any(|record| matches!(
            record.lifecycle,
            crate::domain::model::ForensicLifecycle::Final
        )));

        let Json(ForensicTurnProjectionResponse(turn)) = turn_forensics(
            State(state),
            Path((task_id.as_str().to_string(), turn_id.as_str().to_string())),
        )
        .await
        .expect("turn forensics");
        assert_eq!(turn.turn_id, turn_id);
        assert_eq!(turn.records.len(), 3);
    }

    #[test]
    fn broadcast_forensic_sink_forwards_live_updates() {
        let (tx, mut rx) = broadcast::channel(8);
        let sink = BroadcastForensicSink { tx };
        let update = ConversationForensicUpdate {
            task_id: parse_task_id("task-000001").expect("task"),
            turn_id: TurnTraceId::new("task-000001.turn-0001").expect("turn"),
            record_id: TraceRecordId::new("task-000001.turn-0001.record-0001").expect("record"),
        };

        sink.emit(update.clone());

        let received = rx.try_recv().expect("received broadcast");
        assert_eq!(received, update);
    }

    #[test]
    fn forensic_inspector_html_exposes_local_force_and_shadow_surfaces() {
        let html = include_str!("index.html");

        assert!(html.contains("id=\"forensic-topology-overview\""));
        assert!(html.contains("id=\"forensic-signal-overview\""));
        assert!(html.contains("id=\"forensic-shadow-overview\""));
        assert!(!html.contains("src=\"https://"));
        assert!(!html.contains("src='https://"));
        assert!(!html.contains("href=\"https://"));
        assert!(!html.contains("href='https://"));
    }

    #[test]
    fn forensic_inspector_html_subscribes_to_replay_backed_live_updates() {
        let html = include_str!("index.html");

        assert!(html.contains("/forensics/events"));
        assert!(html.contains("forensic_update"));
        assert!(html.contains("scheduleForensicRefresh"));
        assert!(html.contains("await refreshForensics(refreshOptions)"));
        assert!(html.contains("refreshTraceGraph();"));
    }

    #[test]
    fn transit_trace_html_supports_wheel_zoom() {
        let html = include_str!("index.html");

        assert!(html.contains("--trace-scale"));
        assert!(html.contains("bindTraceZoom"));
        assert!(html.contains("addEventListener('wheel'"));
        assert!(html.contains("const TRACE_ZOOM_MIN = 0.4"));
    }

    #[test]
    fn transit_trace_html_supports_drag_pan() {
        let html = include_str!("index.html");

        assert!(html.contains("bindTracePan"));
        assert!(html.contains("addEventListener('mousedown'"));
        assert!(html.contains("window.addEventListener('mousemove'"));
        assert!(html.contains("trace-board.is-panning"));
        assert!(html.contains("--trace-pan-x"));
        assert!(html.contains("applyTracePanTransform"));
    }
}
