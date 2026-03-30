use crate::application::MechSuitService;
use crate::domain::model::{TurnEvent, TurnEventSink};
use axum::Router;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, Json};
use axum::routing::{get, post};
use paddles_conversation::ConversationSession;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{Mutex, broadcast};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::cors::CorsLayer;

struct AppState {
    service: Arc<MechSuitService>,
    sessions: Mutex<HashMap<String, SessionEntry>>,
    event_tx: broadcast::Sender<(String, TurnEvent)>,
}

struct SessionEntry {
    session: ConversationSession,
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

struct BroadcastEventSink {
    session_id: String,
    tx: broadcast::Sender<(String, TurnEvent)>,
}

impl TurnEventSink for BroadcastEventSink {
    fn emit(&self, event: TurnEvent) {
        let _ = self.tx.send((self.session_id.clone(), event));
    }
}

pub fn router(service: Arc<MechSuitService>) -> Router {
    let (event_tx, _) = broadcast::channel::<(String, TurnEvent)>(256);
    let state = Arc::new(AppState {
        service,
        sessions: Mutex::new(HashMap::new()),
        event_tx,
    });

    Router::new()
        .route("/", get(index_page))
        .route("/health", get(health))
        .route("/sessions", post(create_session))
        .route("/sessions/{id}/turns", post(submit_turn))
        .route("/sessions/{id}/events", get(event_stream))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn index_page() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn create_session(State(state): State<Arc<AppState>>) -> Json<SessionResponse> {
    let session = state.service.create_conversation_session();
    let session_id = session.task_id().as_str().to_string();
    state
        .sessions
        .lock()
        .await
        .insert(session_id.clone(), SessionEntry { session });
    Json(SessionResponse { session_id })
}

async fn submit_turn(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<TurnRequest>,
) -> Result<Json<TurnResponse>, axum::http::StatusCode> {
    let session = {
        let sessions = state.sessions.lock().await;
        sessions
            .get(&id)
            .map(|entry| entry.session.clone())
            .ok_or(axum::http::StatusCode::NOT_FOUND)?
    };

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
