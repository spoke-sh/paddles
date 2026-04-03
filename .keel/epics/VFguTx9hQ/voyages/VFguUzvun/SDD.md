# Unified Projection Store And Product-Route Sync - Software Design Description

> Replace duplicated web bootstrap and multi-endpoint refresh logic with a single shared conversation projection contract, then rebuild the React runtime and product-route E2E around that contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the mixed-ownership runtime boundary where TanStack hosts a monolithic imperative shell that still owns session bootstrap, fetch orchestration, and live updates. In its place, the application layer exposes a canonical conversation projection snapshot/update contract, the web adapter serves that contract through one bootstrap endpoint and one session-scoped event stream, and the primary React/TanStack runtime owns chat, transit, and manifold routes directly through a shared projection store.

## Context & Boundaries

### In Scope

- canonical conversation projection contracts for transcript, forensic, manifold, and trace state
- one web bootstrap endpoint and one session-scoped live projection stream
- one shared React-side projection store/hook
- React/TanStack route ownership for chat, transit, and manifold with parity to the current UI
- product-route browser E2E for external turn injection and reload continuity
- verification/governor alignment and documentation

### Out of Scope

- visual redesign or route taxonomy changes
- hosted state services or remote coordination layers
- TUI redesign work
- optional performance work beyond correctness and verification

```
┌──────────────────────────────────────────────────────────────────────┐
│                              This Voyage                             │
│                                                                      │
│ shared conversation session                                           │
│          │                                                           │
│          v                                                           │
│ application read models ──> canonical projection snapshot/update     │
│          │                                   │                        │
│          │                                   v                        │
│          │                       web bootstrap + live stream          │
│          │                                   │                        │
│          └───────────────> React projection store                    │
│                                              │                        │
│                         ┌────────────────────┼────────────────────┐   │
│                         v                    v                    v   │
│                       chat route         transit route        manifold │
│                                                                      │
│ product-route E2E injects external turns against the shared session  │
└──────────────────────────────────────────────────────────────────────┘
         ↑                                                    ↑
       TUI / CLI                                      browser governor
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Shared conversation session + transcript/forensic/manifold/trace read models | internal | Source of truth for interactive state | current |
| Axum web adapter | internal | Serves bootstrap, projection events, and built frontend assets | current |
| TanStack Router React runtime | internal | Owns primary web routes and projection store consumers | current |
| Playwright browser harness | internal/local dependency | Verifies product-route cross-surface behavior | current |
| `just` + `nix develop` workflow | internal | Hermetic verification path for humans, CI, and governor | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Projection ownership | Introduce one canonical conversation projection contract | Eliminates per-panel reconstruction and establishes one web-facing source of truth |
| Live transport | One session-scoped projection event stream | Reduces duplicated SSE wiring and reconnect logic |
| React boundary | TanStack runtime owns routes and state directly; no raw HTML bridge | Removes the mixed-ownership runtime layer that keeps drifting |
| UI parity posture | Preserve current look/feel/route behavior while moving ownership | The mission is a simplification slice, not a redesign |
| Test posture | Product-route browser E2E with external turn injection | This is the real contract that has been failing in production behavior |
| Verification path | `just test` and governor both run the same full browser suite | Prevents drift between local confidence and commit protection |

## Architecture

1. The application layer builds a canonical conversation projection from the shared session’s transcript, forensic, manifold, and trace read models.
2. The web adapter serves a unified bootstrap snapshot and one session-scoped live update stream for that projection.
3. The React/TanStack runtime hydrates a shared projection store from the bootstrap snapshot and merges live updates from the stream.
4. Chat, transit, and manifold routes render from that store and share route/state semantics instead of each owning its own fetch logic.
5. Product-route browser E2E verifies that external turns against the shared session propagate into the open browser without reload.

## Components

`ConversationProjectionBuilder`
: Application-layer abstraction that packages transcript, forensic, manifold, and trace graph state into one canonical snapshot/update model.

`ProjectionBootstrapEndpoint`
: Web adapter route that returns the initial projection snapshot for the active shared conversation session.

`ProjectionEventStream`
: Session-scoped live stream emitting typed projection updates that the browser can merge without panel-local fan-out wiring.

`ProjectionStore`
: Shared React-side store/hook that owns session identity, bootstrap hydration, live updates, and replay recovery decisions for all routes.

`ConversationRoute`
: React/TanStack chat route that renders transcript and submission UX from the projection store.

`TransitRoute`
: React/TanStack transit trace route that renders the trace graph from the same projection store and store-owned route state.

`ManifoldRoute`
: React/TanStack manifold route that renders the steering-signal manifold from the same projection store and timeline state.

`CrossSurfaceProductRouteE2E`
: Browser harness that keeps a product page open, injects turns externally, and verifies transcript/transit/manifold continuity.

## Interfaces

Candidate internal interfaces:

- `replay_conversation_projection(task_id) -> ConversationProjectionSnapshot`
- `project_conversation_update(task_id, update) -> ConversationProjectionUpdate`
- `subscribe_conversation_projection(task_id) -> stream`

Candidate web contracts:

- `GET /sessions/{id}/projection`
- `GET /sessions/{id}/projection/events`
- `POST /sessions/{id}/turns`

Candidate payload fields:

- `session_id`
- `transcript`
- `forensics`
- `manifold`
- `trace_graph`
- `update_kind`
- `lifecycle`
- `turn_id`
- `record_id`

## Data Flow

1. A turn enters through TUI, web, or another interactive surface attached to the shared conversation session.
2. The application updates transcript, forensic, manifold, and trace read models for that session.
3. `ConversationProjectionBuilder` materializes a snapshot or update from those read models.
4. The web adapter serves the initial snapshot through the bootstrap endpoint and emits subsequent updates on the session stream.
5. The React projection store hydrates from the snapshot, merges updates, and notifies route consumers.
6. Chat, transit, and manifold rerender from the same store without each performing separate fetch/replay repair logic.
7. Product-route E2E opens a page, injects a turn externally, and confirms the store-driven UI updates live and after reload.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Projection snapshot omits a panel’s source data | Integration tests or route render failures show missing transcript/transit/manifold state | Fail the slice and extend the canonical projection contract | Rebuild the snapshot from the shared read models before continuing the cutover |
| Live updates are missed or arrive out of order | Store detects a version/signature gap or route state becomes stale | Trigger replay/bootstrap refresh from the canonical projection endpoint | Recover from authoritative replay instead of panel-local heuristic repair |
| React route diverges visually or behaviorally from the current UI | Visual/manual parity review or browser tests fail route-specific assertions | Treat as a parity regression and block cutover completion | Adjust the React implementation until parity is restored |
| Browser E2E passes locally but not under the governor path | Workflow contract tests or hook runs diverge from `just test` | Normalize the repo-owned command path and remove duplicate harness assumptions | Run the same suite through `just test` under `nix develop` everywhere |
