# Axum Server With SSE Turn Events - Software Design Description

> Deliver the axum HTTP server with session endpoints and SSE-streamed TurnEvents

**SRS:** [SRS.md](SRS.md)

## Overview

An axum HTTP server runs as a tokio task alongside the CLI/TUI frontend. It is an infrastructure adapter that shares the existing MechSuitService instance. Sessions and turn processing use the same application layer methods as the CLI. TurnEvents broadcast to connected SSE clients through a channel-backed TurnEventSink implementation.

## Architecture

The server lives in `src/infrastructure/web/` as a peer to `src/infrastructure/cli/`. It implements `TurnEventSink` via a broadcast channel to fan out events to SSE connections.

```
main.rs
  ├── spawns axum server on --port (default 3000)
  └── runs CLI/TUI frontend as before

infrastructure/web/
  ├── mod.rs (server setup, router)
  └── handlers.rs (endpoint handlers)

Shared: Arc<MechSuitService>
```

## Components

### WebServer
- Binds to `0.0.0.0:{port}`, serves the axum Router
- Holds `Arc<MechSuitService>` as shared state
- Manages a session registry (`HashMap<String, ConversationSession>`)

### BroadcastEventSink
- Implements `TurnEventSink` using `tokio::sync::broadcast`
- Each SSE connection subscribes to the broadcast receiver
- Events serialize as JSON via the new Serialize derive on TurnEvent

## Interfaces

### GET /health
Returns: `{ "status": "ok", "lanes": { "planner": "...", "synthesizer": "...", "gatherer": "..." } }`

### POST /sessions
Returns: `{ "session_id": "task-000001" }`

### POST /sessions/:id/turns
Body: `{ "prompt": "..." }`
Returns: `{ "response": "..." }`

### GET /sessions/:id/events
Returns: SSE stream with `event: turn_event` and `data: { "type": "...", ... }`

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| axum | crate | HTTP framework | 0.8 |
| tokio broadcast | stdlib | SSE fan-out | tokio 1.x |
| serde | derive | TurnEvent JSON serialization | 1.0 |
| tower-http | crate | CORS middleware | 0.6 |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| HTTP framework | axum | Already uses tokio and tower, zero-friction integration |
| Event streaming | SSE via axum::response::sse | Unidirectional, HTTP-native, browser EventSource compatible |
| Session storage | In-memory HashMap behind Arc+Mutex | Local-first, single-process, matches existing ConversationSession model |

## Data Flow

1. Client POSTs to /sessions, gets session_id
2. Client opens SSE connection to /sessions/:id/events
3. Client POSTs to /sessions/:id/turns with prompt
4. Server calls MechSuitService::process_prompt_in_session_with_sink with a BroadcastEventSink
5. BroadcastEventSink emits TurnEvents to all SSE subscribers
6. Turn completes, response returned in POST body

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unknown session ID | HashMap lookup miss | 404 Not Found | Client creates new session |
| Runtime not initialized | MechSuitService returns error | 503 Service Unavailable | Client retries after boot |
| SSE client disconnects | Broadcast receiver dropped | Server cleans up silently | No action needed |
