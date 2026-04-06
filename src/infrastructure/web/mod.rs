use crate::application::MechSuitService;
use crate::domain::model::{
    ConversationForensicProjection, ConversationForensicUpdate, ConversationManifoldProjection,
    ConversationProjectionSnapshot, ConversationProjectionUpdate, ConversationTraceGraph,
    ConversationTranscript, ConversationTranscriptUpdate, ForensicLifecycle,
    ForensicRecordProjection, ForensicTurnProjection, ForensicUpdateSink, ManifoldFrame,
    ManifoldTurnProjection, RuntimeEventPresentation, TaskTraceId, TranscriptUpdateSink, TurnEvent,
    TurnEventSink, TurnTraceId, project_runtime_event,
};
use crate::domain::ports::TraceRecorder;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::header::{CACHE_CONTROL, CONTENT_TYPE};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Json, Response};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::{Component, Path as FsPath, PathBuf};
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
    projection_tx: broadcast::Sender<ConversationProjectionStreamEvent>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConversationProjectionStreamEvent {
    ProjectionUpdate(ConversationProjectionUpdate),
    TurnEvent {
        task_id: TaskTraceId,
        event: TurnEvent,
        presentation: RuntimeEventPresentation,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct ProjectionTurnEventResponse {
    event: TurnEvent,
    presentation: RuntimeEventPresentation,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[derive(Serialize)]
struct SessionResponse {
    session_id: String,
}

#[derive(Serialize)]
struct ConversationBootstrapResponse {
    session_id: String,
    projection: ConversationProjectionSnapshot,
    prompt_history: Vec<String>,
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
struct ConversationProjectionResponse(ConversationProjectionSnapshot);

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
struct ManifoldProjectionResponse(ConversationManifoldProjection);

#[derive(Serialize)]
struct ManifoldTurnProjectionResponse(ManifoldTurnProjection);

#[derive(Serialize)]
struct ManifoldUpdateEventResponse {
    update: ConversationForensicUpdate,
    turn: Option<ManifoldTurnProjection>,
    frame: Option<ManifoldFrame>,
}

struct BroadcastEventSink {
    session_id: String,
    task_id: TaskTraceId,
    verbose: u8,
    event_tx: broadcast::Sender<(String, TurnEvent)>,
    projection_tx: broadcast::Sender<ConversationProjectionStreamEvent>,
}

fn should_forward_projection_event(event: &TurnEvent, verbose: u8) -> bool {
    event.is_visible_at_verbosity(verbose)
}

impl TurnEventSink for BroadcastEventSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.event_tx.send((self.session_id.clone(), event.clone()));
        if should_forward_projection_event(&event, self.verbose) {
            let _ = self
                .projection_tx
                .send(ConversationProjectionStreamEvent::TurnEvent {
                    task_id: self.task_id.clone(),
                    presentation: project_runtime_event(&event),
                    event,
                });
        }
    }
}

struct BroadcastProjectionTranscriptSink {
    service: Arc<MechSuitService>,
    tx: broadcast::Sender<ConversationProjectionStreamEvent>,
}

impl TranscriptUpdateSink for BroadcastProjectionTranscriptSink {
    fn emit(&self, update: ConversationTranscriptUpdate) {
        if let Ok(payload) = self.service.projection_update_for_transcript(&update) {
            let _ = self
                .tx
                .send(ConversationProjectionStreamEvent::ProjectionUpdate(payload));
        }
    }
}

struct BroadcastProjectionForensicSink {
    service: Arc<MechSuitService>,
    tx: broadcast::Sender<ConversationProjectionStreamEvent>,
}

impl ForensicUpdateSink for BroadcastProjectionForensicSink {
    fn emit(&self, update: ConversationForensicUpdate) {
        if let Ok(payload) = self.service.projection_update_for_forensic(&update) {
            let _ = self
                .tx
                .send(ConversationProjectionStreamEvent::ProjectionUpdate(payload));
        }
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
    let (projection_tx, _) = broadcast::channel::<ConversationProjectionStreamEvent>(512);
    let shared_session = service.shared_conversation_session();
    let shared_task_id = shared_session.task_id();
    let verbose = service.verbose();
    let observer: Arc<dyn TurnEventSink> = Arc::new(GlobalBroadcastSink {
        session_id: shared_task_id.as_str().to_string(),
        task_id: shared_task_id,
        verbose,
        event_tx: event_tx.clone(),
        projection_tx: projection_tx.clone(),
    });
    service.register_transcript_observer(Arc::new(BroadcastTranscriptSink {
        tx: transcript_tx.clone(),
    }));
    service.register_transcript_observer(Arc::new(BroadcastProjectionTranscriptSink {
        service: Arc::clone(&service),
        tx: projection_tx.clone(),
    }));
    service.register_forensic_observer(Arc::new(BroadcastForensicSink {
        tx: forensic_tx.clone(),
    }));
    service.register_forensic_observer(Arc::new(BroadcastProjectionForensicSink {
        service: Arc::clone(&service),
        tx: projection_tx.clone(),
    }));
    let state = Arc::new(AppState {
        service,
        trace_recorder,
        event_tx,
        transcript_tx,
        forensic_tx,
        projection_tx,
    });

    let app = Router::new()
        .route("/", get(primary_index_page))
        .route("/manifold", get(primary_index_page))
        .route("/transit", get(primary_index_page))
        .route("/assets/{*path}", get(primary_frontend_asset))
        .route("/favicon.svg", get(primary_frontend_favicon))
        .route("/health", get(health))
        .route(
            "/session/shared/bootstrap",
            get(shared_conversation_bootstrap),
        )
        .route("/sessions", post(create_session))
        .route("/sessions/{id}/turns", post(submit_turn))
        .route("/sessions/{id}/projection", get(conversation_projection))
        .route(
            "/sessions/{id}/projection/events",
            get(conversation_projection_event_stream),
        )
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
        .route("/sessions/{id}/manifold", get(conversation_manifold))
        .route(
            "/sessions/{id}/manifold/turns/{turn_id}",
            get(turn_manifold),
        )
        .route(
            "/sessions/{id}/manifold/events",
            get(conversation_manifold_event_stream),
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

pub fn web_server_url(addr: SocketAddr) -> String {
    format!("http://{addr}")
}

/// Broadcasts all events to the SSE channel regardless of session.
/// Used as a global observer on MechSuitService so CLI turns are visible.
struct GlobalBroadcastSink {
    session_id: String,
    task_id: TaskTraceId,
    verbose: u8,
    event_tx: broadcast::Sender<(String, TurnEvent)>,
    projection_tx: broadcast::Sender<ConversationProjectionStreamEvent>,
}

impl TurnEventSink for GlobalBroadcastSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.event_tx.send((self.session_id.clone(), event.clone()));
        if should_forward_projection_event(&event, self.verbose) {
            let _ = self
                .projection_tx
                .send(ConversationProjectionStreamEvent::TurnEvent {
                    task_id: self.task_id.clone(),
                    presentation: project_runtime_event(&event),
                    event,
                });
        }
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

fn primary_frontend_dist_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("apps")
        .join("web")
        .join("dist")
}

fn load_primary_shell_html_from(dist_dir: &FsPath) -> std::io::Result<String> {
    std::fs::read_to_string(dist_dir.join("index.html"))
}

fn primary_shell_fallback_html() -> &'static str {
    include_str!("index.html")
}

fn load_primary_shell_html() -> String {
    load_primary_shell_html_from(&primary_frontend_dist_dir())
        .unwrap_or_else(|_| primary_shell_fallback_html().to_string())
}

fn resolve_frontend_asset_path(path: &str) -> Option<PathBuf> {
    let trimmed = path.trim_start_matches('/');
    let candidate = primary_frontend_dist_dir().join(trimmed);
    let relative = candidate.strip_prefix(primary_frontend_dist_dir()).ok()?;

    if relative.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return None;
    }

    Some(candidate)
}

fn content_type_for_asset(path: &FsPath) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "js" => "text/javascript; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "json" => "application/json; charset=utf-8",
        "html" => "text/html; charset=utf-8",
        _ => "application/octet-stream",
    }
}

async fn primary_index_page() -> Html<String> {
    Html(load_primary_shell_html())
}

async fn primary_frontend_asset(
    Path(path): Path<String>,
) -> Result<Response, axum::http::StatusCode> {
    let Some(asset_path) = resolve_frontend_asset_path(&format!("assets/{path}")) else {
        return Err(axum::http::StatusCode::NOT_FOUND);
    };
    let bytes = std::fs::read(&asset_path).map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    Ok((
        [
            (CONTENT_TYPE, content_type_for_asset(&asset_path)),
            (CACHE_CONTROL, "no-cache"),
        ],
        bytes,
    )
        .into_response())
}

async fn primary_frontend_favicon() -> Result<Response, axum::http::StatusCode> {
    let Some(asset_path) = resolve_frontend_asset_path("favicon.svg") else {
        return Err(axum::http::StatusCode::NOT_FOUND);
    };
    let bytes = std::fs::read(&asset_path).map_err(|_| axum::http::StatusCode::NOT_FOUND)?;

    Ok((
        [
            (CONTENT_TYPE, content_type_for_asset(&asset_path)),
            (CACHE_CONTROL, "no-cache"),
        ],
        bytes,
    )
        .into_response())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn create_session(State(state): State<Arc<AppState>>) -> Json<SessionResponse> {
    let session = state.service.shared_conversation_session();
    let session_id = session.task_id().as_str().to_string();
    Json(SessionResponse { session_id })
}

async fn shared_conversation_bootstrap(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ConversationBootstrapResponse>, axum::http::StatusCode> {
    let session = state.service.shared_conversation_session();
    let task_id = session.task_id();
    let projection = state
        .service
        .replay_conversation_projection(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let prompt_history = state.service.prompt_history().unwrap_or_default();

    Ok(Json(ConversationBootstrapResponse {
        session_id: task_id.as_str().to_string(),
        projection,
        prompt_history,
    }))
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
        task_id,
        verbose: state.service.verbose(),
        event_tx: state.event_tx.clone(),
        projection_tx: state.projection_tx.clone(),
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

async fn conversation_projection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ConversationProjectionResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let projection = state
        .service
        .replay_conversation_projection(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ConversationProjectionResponse(projection)))
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

async fn conversation_manifold(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ManifoldProjectionResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let projection = state
        .service
        .replay_conversation_manifold(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(ManifoldProjectionResponse(projection)))
}

async fn turn_manifold(
    State(state): State<Arc<AppState>>,
    Path((id, turn_id)): Path<(String, String)>,
) -> Result<Json<ManifoldTurnProjectionResponse>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;
    let turn_id = parse_turn_id(&turn_id)?;
    let turn = state
        .service
        .replay_turn_manifold(&task_id, &turn_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(ManifoldTurnProjectionResponse(turn)))
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

async fn conversation_projection_event_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.projection_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(ConversationProjectionStreamEvent::ProjectionUpdate(update))
            if update.task_id.as_str() == id =>
        {
            let json = serde_json::to_string(&update).unwrap_or_default();
            Some(Ok(Event::default().event("projection_update").data(json)))
        }
        Ok(ConversationProjectionStreamEvent::TurnEvent {
            task_id,
            event,
            presentation,
        }) if task_id.as_str() == id => {
            let json = serde_json::to_string(&ProjectionTurnEventResponse {
                event,
                presentation,
            })
            .unwrap_or_default();
            Some(Ok(Event::default().event("turn_event").data(json)))
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

fn manifold_update_payload(
    state: &Arc<AppState>,
    update: ConversationForensicUpdate,
) -> ManifoldUpdateEventResponse {
    let turn = state
        .service
        .replay_turn_manifold(&update.task_id, &update.turn_id)
        .ok()
        .flatten();
    let frame = turn.as_ref().and_then(|turn| {
        turn.frames
            .iter()
            .find(|frame| frame.record_id == update.record_id)
            .cloned()
    });
    ManifoldUpdateEventResponse {
        update,
        turn,
        frame,
    }
}

async fn conversation_manifold_event_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.forensic_tx.subscribe();
    let state = Arc::clone(&state);
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok(update) if update.task_id.as_str() == id => {
            let payload = manifold_update_payload(&state, update);
            let json = serde_json::to_string(&payload).unwrap_or_default();
            Some(Ok(Event::default().event("manifold_update").data(json)))
        }
        _ => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn event_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let rx = state.event_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |result| match result {
        Ok((session_id, event)) if session_id == id => {
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
) -> Result<Json<ConversationTraceGraph>, axum::http::StatusCode> {
    let task_id = parse_task_id(&id)?;

    let graph = state
        .service
        .replay_conversation_trace_graph(&task_id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(graph))
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

async fn trace_graph_all(State(state): State<Arc<AppState>>) -> Json<ConversationTraceGraph> {
    let replays: Vec<_> = state
        .trace_recorder
        .task_ids()
        .iter()
        .filter_map(|id| state.trace_recorder.replay(id).ok())
        .collect();
    Json(ConversationTraceGraph::from_trace_replays(&replays))
}

#[cfg(test)]
mod tests {
    use super::{
        AppState, BroadcastForensicSink, BroadcastProjectionForensicSink,
        BroadcastProjectionTranscriptSink, ConversationBootstrapResponse,
        ConversationProjectionStreamEvent, ForensicProjectionResponse,
        ForensicTurnProjectionResponse, conversation_forensics, parse_task_id, turn_forensics,
    };
    use crate::application::{MechSuitService, RuntimeLaneConfig};
    use crate::domain::model::NullTurnEventSink;
    use crate::domain::model::{
        ArtifactKind, ConversationForensicUpdate, ConversationProjectionUpdateKind,
        ConversationTranscriptUpdate, ForensicUpdateSink, TraceCheckpointKind,
        TraceCompletionCheckpoint, TraceLineage, TraceLineageNodeKind, TraceLineageNodeRef,
        TraceModelExchangeArtifact, TraceModelExchangeCategory, TraceModelExchangeLane,
        TraceModelExchangePhase, TraceRecord, TraceRecordId, TraceRecordKind, TraceReplay,
        TraceSignalContribution, TraceSignalKind, TraceSignalSnapshot, TranscriptUpdateSink,
        TurnEventSink,
    };
    use crate::domain::ports::{
        ModelPaths, ModelRegistry, RecursivePlanner, SynthesizerEngine, TraceRecorder,
    };
    use crate::infrastructure::adapters::agent_memory::AgentMemory;
    use crate::infrastructure::adapters::http_provider::{
        ApiFormat, HttpPlannerAdapter, HttpProviderAdapter,
    };
    use crate::infrastructure::adapters::trace_recorders::InMemoryTraceRecorder;
    use crate::infrastructure::conversation_history::ConversationHistoryStore;
    use crate::infrastructure::providers::ModelProvider;
    use crate::infrastructure::rendering::RenderCapability;
    use anyhow::{Result, anyhow};
    use async_trait::async_trait;
    use axum::Json;
    use axum::body::{Body, to_bytes};
    use axum::extract::{Path, State};
    use axum::http::{HeaderMap, Request, StatusCode, Uri};
    use axum::response::{IntoResponse, Response};
    use axum::routing::any;
    use axum::{Router, body::Bytes};
    use paddles_conversation::{
        ArtifactEnvelope, ConversationSession, TraceArtifactId, TraceCheckpointId, TurnTraceId,
    };
    use serde_json::json;
    use std::collections::VecDeque;
    use std::path::Path as FsPath;
    use std::sync::{Arc, Mutex};
    use tokio::sync::broadcast;
    use tokio::task::JoinHandle;
    use tower::util::ServiceExt;

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

    #[derive(Clone, Debug)]
    struct MockResponse {
        status: StatusCode,
        body: String,
    }

    struct MockServerState {
        responses: Mutex<VecDeque<MockResponse>>,
    }

    struct MockServerHandle {
        base_url: String,
        task: JoinHandle<()>,
    }

    impl Drop for MockServerHandle {
        fn drop(&mut self) {
            self.task.abort();
        }
    }

    async fn mock_provider_handler(
        State(state): State<Arc<MockServerState>>,
        _headers: HeaderMap,
        _uri: Uri,
        _body: Bytes,
    ) -> Response {
        let response = state
            .responses
            .lock()
            .expect("response lock")
            .pop_front()
            .expect("queued response");

        (
            response.status,
            [("content-type", "application/json")],
            response.body,
        )
            .into_response()
    }

    async fn start_mock_provider_server(responses: Vec<MockResponse>) -> MockServerHandle {
        let state = Arc::new(MockServerState {
            responses: Mutex::new(VecDeque::from(responses)),
        });
        let app = Router::new()
            .fallback(any(mock_provider_handler))
            .with_state(Arc::clone(&state));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind mock provider");
        let addr = listener.local_addr().expect("local addr");
        let task = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("run mock provider");
        });

        MockServerHandle {
            base_url: format!("http://{}", addr),
            task,
        }
    }

    fn openai_content_response(content: &str) -> String {
        json!({
            "choices": [
                {
                    "message": {
                        "content": content,
                    }
                }
            ]
        })
        .to_string()
    }

    fn openai_tool_call_response(arguments: &str) -> String {
        json!({
            "choices": [
                {
                    "message": {
                        "content": null,
                        "tool_calls": [
                            {
                                "id": "call_test",
                                "type": "function",
                                "function": {
                                    "name": "select_planner_action",
                                    "arguments": arguments,
                                }
                            }
                        ]
                    }
                }
            ]
        })
        .to_string()
    }

    fn live_http_test_service_with_recorder(
        workspace: &FsPath,
        base_url: String,
        recorder: Arc<dyn TraceRecorder>,
    ) -> Arc<MechSuitService> {
        let operator_memory = Arc::new(AgentMemory::load(workspace));

        let synth_base_url = base_url.clone();
        let synthesizer_factory: Box<crate::application::SynthesizerFactory> = Box::new(
            move |workspace: &FsPath, lane: &crate::application::PreparedModelLane| {
                Ok(Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    ModelProvider::Inception.name(),
                    lane.model_id.clone(),
                    "test-key",
                    synth_base_url.clone(),
                    ApiFormat::OpenAi,
                    RenderCapability::OpenAiJsonSchema,
                )) as Arc<dyn SynthesizerEngine>)
            },
        );

        let planner_base_url = base_url;
        let planner_factory: Box<crate::application::PlannerFactory> = Box::new(
            move |workspace: &FsPath, lane: &crate::application::PreparedModelLane| {
                let engine = Arc::new(HttpProviderAdapter::new(
                    workspace.to_path_buf(),
                    ModelProvider::Inception.name(),
                    lane.model_id.clone(),
                    "test-key",
                    planner_base_url.clone(),
                    ApiFormat::OpenAi,
                    RenderCapability::OpenAiJsonSchema,
                ));
                Ok(Arc::new(HttpPlannerAdapter::new(engine)) as Arc<dyn RecursivePlanner>)
            },
        );

        Arc::new(MechSuitService::with_trace_recorder(
            workspace,
            Arc::new(StaticRegistry),
            operator_memory,
            synthesizer_factory,
            planner_factory,
            Box::new(|_, _, _, _| Ok::<Option<_>, anyhow::Error>(None)),
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

    fn seed_manifold_replay(
        recorder: &Arc<InMemoryTraceRecorder>,
        session: &ConversationSession,
    ) -> (TraceReplay, TraceRecordId) {
        let task_id = session.task_id();
        let turn_id = session.allocate_turn_id();
        let signal_record_id =
            TraceRecordId::new(format!("{}.record-0001", turn_id.as_str())).expect("record");
        let checkpoint_record_id =
            TraceRecordId::new(format!("{}.record-0002", turn_id.as_str())).expect("record");

        recorder
            .record(TraceRecord {
                record_id: signal_record_id.clone(),
                sequence: 1,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: None,
                },
                kind: TraceRecordKind::SignalSnapshot(TraceSignalSnapshot {
                    kind: TraceSignalKind::ActionBias,
                    summary: "action bias accumulated".to_string(),
                    level: "high".to_string(),
                    magnitude_percent: 82,
                    applies_to: Some(TraceLineageNodeRef {
                        kind: TraceLineageNodeKind::Turn,
                        id: turn_id.as_str().to_string(),
                        label: "turn".to_string(),
                    }),
                    contributions: vec![TraceSignalContribution {
                        source: "controller_policy".to_string(),
                        share_percent: 100,
                        rationale: "test contribution".to_string(),
                    }],
                    artifact: text_artifact(
                        "signal-artifact-1",
                        ArtifactKind::Checkpoint,
                        "signal snapshot",
                        "{\"kind\":\"action_bias\"}",
                    ),
                }),
            })
            .expect("record signal snapshot");
        recorder
            .record(TraceRecord {
                record_id: checkpoint_record_id.clone(),
                sequence: 2,
                lineage: TraceLineage {
                    task_id: task_id.clone(),
                    turn_id: turn_id.clone(),
                    branch_id: None,
                    parent_record_id: Some(signal_record_id.clone()),
                },
                kind: TraceRecordKind::CompletionCheckpoint(TraceCompletionCheckpoint {
                    checkpoint_id: TraceCheckpointId::new(format!(
                        "{}.checkpoint",
                        turn_id.as_str()
                    ))
                    .expect("checkpoint"),
                    kind: TraceCheckpointKind::TurnCompleted,
                    summary: "done".to_string(),
                    response: None,
                    citations: Vec::new(),
                    grounded: true,
                }),
            })
            .expect("record checkpoint");

        (recorder.replay(&task_id).expect("replay"), signal_record_id)
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
            projection_tx: broadcast::channel(8).0,
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

    #[tokio::test]
    async fn manifold_routes_project_time_ordered_signal_frames_and_lineage_anchors() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let (replay, signal_record_id) = seed_manifold_replay(&recorder, &session);
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
            projection_tx: broadcast::channel(8).0,
        });

        let Json(super::ManifoldProjectionResponse(conversation)) = super::conversation_manifold(
            State(Arc::clone(&state)),
            Path(task_id.as_str().to_string()),
        )
        .await
        .expect("conversation manifold");
        assert_eq!(conversation.task_id, task_id);
        assert_eq!(conversation.turns.len(), 1);
        assert_eq!(conversation.turns[0].turn_id, turn_id);
        assert_eq!(conversation.turns[0].frames.len(), 2);
        assert_eq!(conversation.turns[0].frames[0].record_id, signal_record_id);
        assert_eq!(
            conversation.turns[0].frames[0].active_signals[0]
                .anchor
                .as_ref()
                .expect("anchor")
                .kind,
            TraceLineageNodeKind::Turn
        );

        let Json(super::ManifoldTurnProjectionResponse(turn)) = super::turn_manifold(
            State(state),
            Path((task_id.as_str().to_string(), turn_id.as_str().to_string())),
        )
        .await
        .expect("turn manifold");
        assert_eq!(turn.turn_id, turn_id);
        assert_eq!(turn.frames.len(), 2);
    }

    #[test]
    fn manifold_update_payload_replays_turn_state_for_live_updates() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let session = service.shared_conversation_session();
        let (replay, signal_record_id) = seed_manifold_replay(&recorder, &session);
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
            projection_tx: broadcast::channel(8).0,
        });

        let payload = super::manifold_update_payload(
            &state,
            ConversationForensicUpdate {
                task_id,
                turn_id,
                record_id: signal_record_id,
            },
        );

        assert!(payload.turn.is_some());
        assert!(payload.frame.is_some());
        assert_eq!(
            payload.frame.expect("frame").active_signals[0].kind,
            TraceSignalKind::ActionBias
        );
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

    #[tokio::test(flavor = "multi_thread")]
    async fn broadcast_projection_sinks_rebuild_snapshots_from_authoritative_replay() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse the local harness before answering.\n",
        )
        .expect("write AGENTS.md");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let server = start_mock_provider_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: openai_content_response(
                    "I should inspect the local workspace before answering.",
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_tool_call_response(
                    r#"{"action":"inspect","command":"pwd","rationale":"inspect the local workspace before answering"}"#,
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_tool_call_response(
                    r#"{"action":"answer","rationale":"the local evidence is sufficient"}"#,
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_content_response(
                    r#"{"render_types":["paragraph"],"blocks":[{"type":"paragraph","text":"Mock provider completed the turn after local inspection."}]}"#,
                ),
            },
        ])
        .await;
        let service = live_http_test_service_with_recorder(
            workspace.path(),
            server.base_url.clone(),
            recorder.clone(),
        );
        let runtime_lanes = RuntimeLaneConfig::new("mercury-2".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Inception)
            .with_planner_provider(Some(ModelProvider::Inception));
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare lanes");

        let session = service.shared_conversation_session();
        service
            .process_prompt_in_session_with_sink(
                "CI is failing. Can you debug it on this machine?",
                session.clone(),
                Arc::new(NullTurnEventSink),
            )
            .await
            .expect("process prompt");

        let replay = recorder.replay(&session.task_id()).expect("replay");
        let last_record = replay.records.last().expect("record");
        let task_id = session.task_id();
        let forensic_update = ConversationForensicUpdate {
            task_id: task_id.clone(),
            turn_id: last_record.lineage.turn_id.clone(),
            record_id: last_record.record_id.clone(),
        };
        let transcript_update = ConversationTranscriptUpdate {
            task_id: task_id.clone(),
        };

        let (tx, mut rx) = broadcast::channel(8);
        BroadcastProjectionTranscriptSink {
            service: Arc::clone(&service),
            tx: tx.clone(),
        }
        .emit(transcript_update.clone());
        let ConversationProjectionStreamEvent::ProjectionUpdate(transcript_payload) =
            rx.try_recv().expect("transcript payload")
        else {
            panic!("expected projection update payload");
        };
        assert_eq!(
            transcript_payload.kind,
            ConversationProjectionUpdateKind::Transcript
        );
        assert_eq!(transcript_payload.task_id, task_id);
        assert_eq!(
            transcript_payload.transcript_update,
            Some(transcript_update)
        );
        assert_eq!(transcript_payload.snapshot.transcript.entries.len(), 2);
        assert!(
            transcript_payload.snapshot.transcript.entries[1]
                .render
                .as_ref()
                .is_some_and(|render| !render.blocks.is_empty())
        );
        assert!(!transcript_payload.snapshot.trace_graph.nodes.is_empty());

        BroadcastProjectionForensicSink { service, tx }.emit(forensic_update.clone());
        let ConversationProjectionStreamEvent::ProjectionUpdate(forensic_payload) =
            rx.try_recv().expect("forensic payload")
        else {
            panic!("expected projection update payload");
        };
        assert_eq!(
            forensic_payload.kind,
            ConversationProjectionUpdateKind::Forensic
        );
        assert_eq!(forensic_payload.task_id, task_id);
        assert_eq!(forensic_payload.forensic_update, Some(forensic_update));
        assert_eq!(forensic_payload.snapshot.forensics.turns.len(), 1);
        assert_eq!(forensic_payload.snapshot.manifold.turns.len(), 1);
        assert!(!forensic_payload.snapshot.trace_graph.nodes.is_empty());
    }

    #[test]
    fn broadcast_event_sink_tags_turn_events_with_the_session_projection_identity() {
        let task_id = parse_task_id("task-000001").expect("task");
        let (event_tx, mut event_rx) = broadcast::channel(8);
        let (projection_tx, mut projection_rx) = broadcast::channel(8);
        let sink = super::BroadcastEventSink {
            session_id: task_id.as_str().to_string(),
            task_id: task_id.clone(),
            verbose: 0,
            event_tx,
            projection_tx,
        };

        let event = crate::domain::model::TurnEvent::ToolCalled {
            call_id: "call-1".to_string(),
            tool_name: "shell".to_string(),
            invocation: "pwd".to_string(),
        };

        sink.emit(event.clone());

        let (session_id, received_event) = event_rx.try_recv().expect("event payload");
        assert_eq!(session_id, task_id.as_str());
        assert_eq!(received_event, event);

        let ConversationProjectionStreamEvent::TurnEvent {
            task_id: received_task_id,
            event: received_projection_event,
            presentation,
        } = projection_rx.try_recv().expect("projection event payload")
        else {
            panic!("expected turn event payload");
        };
        assert_eq!(received_task_id, task_id);
        assert_eq!(received_projection_event, event);
        assert_eq!(presentation.badge, "tool");
        assert_eq!(presentation.title, "• Ran shell");
    }

    #[test]
    fn broadcast_event_sink_filters_projection_rows_by_resolved_verbosity() {
        let task_id = parse_task_id("task-000001").expect("task");
        let (event_tx, mut event_rx) = broadcast::channel(8);
        let (projection_tx, mut projection_rx) = broadcast::channel(8);
        let sink = super::BroadcastEventSink {
            session_id: task_id.as_str().to_string(),
            task_id,
            verbose: 0,
            event_tx,
            projection_tx,
        };

        let event = crate::domain::model::TurnEvent::IntentClassified {
            intent: crate::domain::model::TurnIntent::Casual,
        };

        sink.emit(event.clone());

        let (_, received_event) = event_rx.try_recv().expect("event payload");
        assert_eq!(received_event, event);
        assert!(projection_rx.try_recv().is_err());
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
    fn web_html_exposes_manifold_route_shell_and_path_sync() {
        let html = include_str!("index.html");

        assert!(html.contains("id=\"manifold-view\""));
        assert!(html.contains("id=\"manifold-shell\""));
        assert!(html.contains("data-trace-view=\"manifold\""));
        assert!(html.contains("window.location.pathname"));
        assert!(html.contains("history.pushState"));
        assert!(html.contains("renderManifoldShell"));
    }

    #[test]
    fn manifold_route_html_uses_bounded_local_scrollers() {
        let html = include_str!("index.html");

        assert!(html.contains("html,\nbody {\n  height: 100%;\n  overflow: hidden;"));
        assert!(html.contains(".manifold-shell"));
        assert!(html.contains(
            ".manifold-canvas {\n  flex: 1;\n  min-height: 0;\n  padding: 20px;\n  overflow: auto;"
        ));
        assert!(html.contains(".manifold-panel-list {\n  margin-top: 14px;\n  display: grid;\n  gap: 10px;\n  min-height: 0;\n  overflow: auto;"));
    }

    #[test]
    fn manifold_route_html_renders_topology_primitives_and_conduits() {
        let html = include_str!("index.html");

        assert!(html.contains("frame.primitives"));
        assert!(html.contains("frame.conduits"));
        assert!(html.contains("manifold-node"));
        assert!(html.contains("manifold-conduit"));
    }

    #[test]
    fn manifold_route_html_supports_time_scrub_controls() {
        let html = include_str!("index.html");

        assert!(html.contains("id=\"manifold-play-toggle\""));
        assert!(html.contains("id=\"manifold-replay-button\""));
        assert!(html.contains("id=\"manifold-time-scrubber\""));
        assert!(html.contains("data-manifold-turn-id"));
        assert!(html.contains("function advanceManifoldPlayback"));
        assert!(html.contains("requestAnimationFrame"));
    }

    #[test]
    fn manifold_route_html_uses_compact_playback_banner_once_frames_exist() {
        let html = include_str!("index.html");

        assert!(html.contains(".manifold-playback-banner {"));
        assert!(html.contains("<div class=\"manifold-playback-banner\">"));
        assert!(!html.contains("<div class=\"manifold-empty-state\"><strong>Temporal manifold playback is active.</strong>"));
    }

    #[test]
    fn manifold_route_html_encodes_temporal_signal_phases() {
        let html = include_str!("index.html");

        assert!(html.contains("function primitivePhase"));
        assert!(html.contains("accumulating"));
        assert!(html.contains("superseded"));
        assert!(html.contains("bleed_off"));
        assert!(html.contains("manifold-node__fill"));
    }

    #[test]
    fn manifold_route_html_streams_live_updates_and_reconciles_from_replay() {
        let html = include_str!("index.html");

        assert!(html.contains("let manifoldEventSource = null;"));
        assert!(html.contains("/manifold/events"));
        assert!(html.contains("manifold_update"));
        assert!(html.contains("function applyManifoldUpdate"));
        assert!(html.contains("function scheduleManifoldReplayRefresh"));
        assert!(html.contains("await refreshManifold()"));
    }

    #[test]
    fn manifold_route_html_surfaces_lifecycle_states_during_live_turns() {
        let html = include_str!("index.html");

        assert!(html.contains("function manifoldLifecycleBadge"));
        assert!(html.contains("data-lifecycle=\""));
        assert!(html.contains("manifold-node__badge"));
        assert!(html.contains("manifold-panel-status"));
    }

    #[test]
    fn manifold_route_html_links_selected_sources_back_to_forensics() {
        let html = include_str!("index.html");

        assert!(html.contains("let selectedManifoldSourceRecordId = null;"));
        assert!(html.contains("function openManifoldSourceInInspector"));
        assert!(html.contains("data-source-record-id"));
        assert!(html.contains("data-open-forensic-record-id"));
        assert!(html.contains("setTraceView('inspector')"));
    }

    #[test]
    fn transit_trace_html_uses_primary_route_paths_only() {
        let html = include_str!("index.html");

        assert!(html.contains("function traceRouteFamily(pathname)"));
        assert!(!html.contains("/legacy"));
        assert!(!html.contains("return routePath === '/' ? '/legacy' : `/legacy${routePath}`;"));
    }

    #[tokio::test]
    async fn web_router_serves_dedicated_manifold_and_transit_routes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let (app, _observer) = super::router(service, recorder);

        for route in ["/", "/manifold", "/transit"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(route)
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::OK, "route {route}");
        }
    }

    #[tokio::test]
    async fn primary_runtime_routes_serve_the_primary_frontend_shell() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let (app, _observer) = super::router(service, recorder);

        for route in ["/", "/manifold", "/transit"] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(route)
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            let body = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body");
            let html = String::from_utf8(body.to_vec()).expect("utf8 html");

            assert!(
                html.contains("<div id=\"root\"></div>"),
                "route {route} should serve the primary React shell"
            );
            assert!(
                !html.contains("id=\"prompt\""),
                "route {route} should not fall back to the embedded runtime shell"
            );
            assert!(
                !html.contains("/legacy"),
                "route {route} should not expose legacy route families"
            );
        }
    }

    #[tokio::test]
    async fn web_router_does_not_expose_legacy_or_app_alias_routes() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let (app, _observer) = super::router(service, recorder);

        for route in [
            "/legacy",
            "/legacy/manifold",
            "/legacy/transit",
            "/app",
            "/app/transit",
            "/app/manifold",
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(route)
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "route {route}");
        }
    }

    #[test]
    fn primary_shell_loader_prefers_react_dist_when_present() {
        let workspace = tempfile::tempdir().expect("workspace");
        let dist = workspace.path().join("dist");
        std::fs::create_dir_all(&dist).expect("create dist");
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><html><body><div id=\"root\">react</div></body></html>",
        )
        .expect("write dist index");

        let html = super::load_primary_shell_html_from(&dist).expect("load primary shell");

        assert!(html.contains("<div id=\"root\">react</div>"));
        assert!(!html.contains("id=\"prompt\""));
    }

    #[test]
    fn primary_shell_loader_falls_back_to_embedded_primary_shell_when_dist_is_missing() {
        let workspace = tempfile::tempdir().expect("workspace");

        let html = super::load_primary_shell_html_from(workspace.path())
            .unwrap_or_else(|_| super::primary_shell_fallback_html().to_string());

        assert_eq!(html, include_str!("index.html"));
        assert!(html.contains("id=\"prompt\""));
        assert!(html.contains("let transcriptEventSource = null;"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn session_routes_project_live_shared_session_turns_from_mock_provider() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::write(
            workspace.path().join("AGENTS.md"),
            "# Operator Memory\nUse the local harness before answering.\n",
        )
        .expect("write AGENTS.md");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let server = start_mock_provider_server(vec![
            MockResponse {
                status: StatusCode::OK,
                body: openai_content_response(
                    "I should inspect the local workspace before answering.",
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_tool_call_response(
                    r#"{"action":"inspect","command":"pwd","rationale":"inspect the local workspace before answering"}"#,
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_tool_call_response(
                    r#"{"action":"answer","rationale":"the local evidence is sufficient"}"#,
                ),
            },
            MockResponse {
                status: StatusCode::OK,
                body: openai_content_response(
                    r#"{"render_types":["paragraph"],"blocks":[{"type":"paragraph","text":"Mock provider completed the turn after local inspection."}]}"#,
                ),
            },
        ])
        .await;
        let service = live_http_test_service_with_recorder(
            workspace.path(),
            server.base_url.clone(),
            recorder.clone(),
        );
        let runtime_lanes = RuntimeLaneConfig::new("mercury-2".to_string(), None)
            .with_synthesizer_provider(ModelProvider::Inception)
            .with_planner_provider(Some(ModelProvider::Inception));
        service
            .prepare_runtime_lanes(&runtime_lanes)
            .await
            .expect("prepare lanes");

        let session = service.shared_conversation_session();
        let task_id = session.task_id().as_str().to_string();
        service
            .process_prompt_in_session_with_sink(
                "CI is failing. Can you debug it on this machine?",
                session,
                Arc::new(NullTurnEventSink),
            )
            .await
            .expect("process prompt");

        let (event_tx, _) = broadcast::channel(8);
        let (transcript_tx, _) = broadcast::channel(8);
        let (forensic_tx, _) = broadcast::channel(8);
        let state = Arc::new(AppState {
            service,
            trace_recorder: recorder,
            event_tx,
            transcript_tx,
            forensic_tx,
            projection_tx: broadcast::channel(8).0,
        });

        let Json(super::ConversationProjectionResponse(projection)) =
            super::conversation_projection(State(Arc::clone(&state)), Path(task_id.clone()))
                .await
                .expect("conversation projection");
        assert_eq!(projection.task_id.as_str(), task_id);
        assert_eq!(projection.transcript.entries.len(), 2);
        assert_eq!(projection.forensics.turns.len(), 1);
        assert_eq!(projection.manifold.turns.len(), 1);
        assert!(!projection.trace_graph.nodes.is_empty());

        let Json(super::TranscriptResponse(transcript)) =
            super::conversation_transcript(State(Arc::clone(&state)), Path(task_id.clone()))
                .await
                .expect("conversation transcript");
        assert_eq!(transcript.entries.len(), 2);
        assert!(!transcript.entries[1].content.trim().is_empty());
        assert!(transcript.entries[1].render.is_some());

        let Json(graph) = super::trace_graph(State(Arc::clone(&state)), Path(task_id.clone()))
            .await
            .expect("trace graph");
        assert!(!graph.nodes.is_empty());
        assert!(graph.nodes.iter().any(|node| node.kind == "root"));
        assert!(graph.nodes.iter().any(|node| node.kind == "action"));
        assert!(graph.nodes.iter().any(|node| node.kind == "signal"));

        let Json(super::ManifoldProjectionResponse(manifold)) =
            super::conversation_manifold(State(state), Path(task_id))
                .await
                .expect("conversation manifold");
        assert_eq!(manifold.turns.len(), 1);
        assert!(
            manifold.turns[0]
                .frames
                .iter()
                .any(|frame| !frame.active_signals.is_empty())
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn shared_bootstrap_route_returns_shared_session_projection() {
        let workspace = tempfile::tempdir().expect("workspace");
        let recorder = Arc::new(InMemoryTraceRecorder::default());
        let service = test_service_with_recorder(workspace.path(), recorder.clone());
        let history_store = Arc::new(ConversationHistoryStore::with_path(
            workspace.path().join("state/conversation-history.toml"),
        ));
        history_store
            .record_prompt("first prompt")
            .expect("record prompt history");
        history_store
            .record_prompt("second prompt")
            .expect("record prompt history");
        service.set_conversation_history_store(Arc::clone(&history_store));
        let state = Arc::new(AppState {
            service: Arc::clone(&service),
            trace_recorder: recorder,
            event_tx: broadcast::channel(8).0,
            transcript_tx: broadcast::channel(8).0,
            forensic_tx: broadcast::channel(8).0,
            projection_tx: broadcast::channel(8).0,
        });

        let shared = service.shared_conversation_session();
        let task_id = shared.task_id();

        let Json(ConversationBootstrapResponse {
            session_id,
            projection,
            prompt_history,
        }) = super::shared_conversation_bootstrap(State(state))
            .await
            .expect("shared bootstrap");

        assert_eq!(session_id, task_id.as_str());
        assert_eq!(projection.task_id, task_id);
        assert_eq!(projection.transcript.entries.len(), 0);
        assert_eq!(projection.forensics.turns.len(), 0);
        assert_eq!(projection.manifold.turns.len(), 0);
        assert!(projection.trace_graph.nodes.is_empty());
        assert_eq!(
            prompt_history,
            vec!["first prompt".to_string(), "second prompt".to_string()]
        );
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

    #[test]
    fn transit_trace_html_supports_significance_toggles() {
        let html = include_str!("index.html");

        assert!(html.contains("id=\"trace-transit-toolbar\""));
        assert!(html.contains("data-trace-scope=\"significant\""));
        assert!(html.contains("data-trace-scope=\"full\""));
        assert!(html.contains("data-trace-family=\"model_io\""));
        assert!(html.contains("data-trace-family=\"signals\""));
        assert!(html.contains("function traceNodeVisible"));
        assert!(html.contains("function syncTransitTraceControls"));
    }

    #[test]
    fn transit_trace_html_fetches_session_scoped_graphs() {
        let html = include_str!("index.html");

        assert!(html.contains("'/sessions/' + sessionId + '/trace/graph'"));
        assert!(!html.contains("fetch('/trace/graph')"));
    }

    #[test]
    fn transit_trace_html_adapts_detail_density_to_zoom() {
        let html = include_str!("index.html");

        assert!(html.contains("function traceDetailLevelForZoom"));
        assert!(html.contains("function traceLayoutForZoom"));
        assert!(html.contains("data-detail-level"));
        assert!(html.contains("--trace-column-gap"));
        assert!(html.contains("--trace-row-gap"));
    }

    #[test]
    fn transit_trace_html_uses_monospace_typography() {
        let html = include_str!("index.html");

        assert!(html.contains("--trace-mono-family"));
        assert!(html.contains(".trace-transit-toggle"));
        assert!(html.contains(".trace-node__label"));
        assert!(html.contains(".trace-node__detail-title"));
        assert!(html.contains("font-family: var(--trace-mono-family);"));
    }
}
