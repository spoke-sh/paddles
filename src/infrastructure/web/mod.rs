use crate::application::MechSuitService;
use crate::domain::model::{
    ConversationTranscript, ConversationTranscriptUpdate, TaskTraceId, TraceRecordKind,
    TranscriptUpdateSink, TurnEvent, TurnEventSink,
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
    let observer: Arc<dyn TurnEventSink> = Arc::new(GlobalBroadcastSink {
        tx: event_tx.clone(),
    });
    service.register_transcript_observer(Arc::new(BroadcastTranscriptSink {
        tx: transcript_tx.clone(),
    }));
    let state = Arc::new(AppState {
        service,
        trace_recorder,
        event_tx,
        transcript_tx,
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
