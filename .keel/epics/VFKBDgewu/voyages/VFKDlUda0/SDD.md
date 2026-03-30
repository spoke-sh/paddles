# Browser Chat Page - Software Design Description

> Deliver a self-contained HTML chat interface consuming SSE turn events

**SRS:** [SRS.md](SRS.md)

## Overview

A self-contained HTML page is compiled into the paddles binary via `include_str!`
and served by an axum GET handler. The page uses vanilla JavaScript with an
`EventSource` to connect to the existing SSE endpoint for a session. Incoming
`TurnEvent` messages are parsed and rendered in two views: message bubbles for
user prompts and assistant responses, and a collapsible event timeline that shows
planner actions, tool calls, and gatherer results as they arrive. No external
dependencies, frameworks, or build steps are required -- the entire UI is a
single HTML file with inline CSS and JS.

## Context & Boundaries

- In scope:
  - axum handler serving the HTML page via include_str!
  - vanilla JS EventSource consuming SSE TurnEvents
  - message bubble rendering for user and assistant messages
  - collapsible event timeline for planner actions, tool calls, gatherer results
  - prompt input POSTing to /sessions/:id/turns
- Out of scope:
  - any JS framework (React, Vue, etc.)
  - trace DAG visualization (separate epic)
  - multi-session management

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌───────────┐  ┌──────────┐  ┌──────┐ │
│  │ HTML page │  │ EventSrc │  │ POST │ │
│  │ (incl_str)│  │ (SSE)    │  │ input│ │
│  └───────────┘  └──────────┘  └──────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [axum server]   [SSE endpoint]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| axum server | internal runtime | Serves the HTML page and handles POST /sessions/:id/turns | current Rust runtime |
| SSE endpoint | internal runtime | Streams TurnEvents to the EventSource client | existing /sessions/:id/events |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Single HTML file via include_str! | Compile the page into the binary | Zero deployment dependencies, no static file serving needed |
| Vanilla JS with EventSource | No framework, native browser SSE API | Minimal complexity, no build tooling, wide browser support |
| Collapsible event timeline | Expandable sections per TurnEvent type | Keeps the chat view clean while preserving full event detail |

## Architecture

The voyage adds a single axum route handler and one HTML asset:

1. `GET /` handler returns the HTML page from `include_str!("chat.html")`.
2. The HTML page contains inline JS that:
   - Opens an `EventSource` connection to `/sessions/:id/events`.
   - Parses each SSE `data:` line as a TurnEvent JSON payload.
   - Appends rendered elements to the DOM based on event type.
3. A prompt form POSTs to `/sessions/:id/turns` with the user message.

## Components

- `chat.html`
  Purpose: self-contained HTML/CSS/JS chat interface.
  Contains message bubble rendering, event timeline with collapsible sections,
  and prompt input form.

- `GET /` handler
  Purpose: serves the chat page from the compiled binary.

## Interfaces

- `GET /` -- returns `text/html` with the embedded chat page.
- `POST /sessions/:id/turns` -- accepts user prompt (consumed by existing turn handler).
- `GET /sessions/:id/events` -- existing SSE endpoint consumed by EventSource.

## Data Flow

1. Browser loads `GET /` and receives the chat HTML page.
2. User enters a prompt; JS POSTs it to `/sessions/:id/turns`.
3. JS opens `EventSource` on `/sessions/:id/events`.
4. Each incoming SSE message is parsed as a TurnEvent.
5. TurnEvent type determines rendering: message bubble or timeline entry.
6. Final synthesis event renders as the assistant message bubble.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| SSE connection drops | EventSource onerror callback | Display reconnection status in UI | EventSource auto-reconnects per spec |
| POST /turns fails | fetch response status != 2xx | Display error message below prompt input | User can retry submission |
| Malformed TurnEvent JSON | JSON.parse throws | Log to console, skip rendering that event | Continue processing subsequent events |
