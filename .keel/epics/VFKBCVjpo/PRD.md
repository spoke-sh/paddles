# HTTP Server Foundation - Product Requirements

## Problem Statement

Paddles has no HTTP interface. An axum server needs to run alongside the CLI, sharing the MechSuitService instance, with endpoints for session lifecycle, turn submission, and SSE event streaming.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | An axum HTTP server starts when paddles starts and serves the API alongside the CLI/TUI | Server binds to localhost and responds to health checks | First voyage |
| GOAL-02 | Clients can create sessions, submit turns, and receive streamed TurnEvents via SSE | End-to-end curl test: POST turn, receive SSE events, get final response | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Web client | Browser or HTTP client consuming the paddles API | Submit prompts and observe the recursive planning process in real time |
| Operator | Developer running paddles locally | See the harness working through both terminal and browser simultaneously |

## Scope

### In Scope

- [SCOPE-01] Axum server starting on a configurable localhost port alongside the existing CLI
- [SCOPE-02] POST /sessions endpoint creating ConversationSession instances
- [SCOPE-03] POST /sessions/:id/turns endpoint submitting prompts through MechSuitService
- [SCOPE-04] GET /sessions/:id/events SSE endpoint streaming TurnEvents as typed JSON
- [SCOPE-05] GET /health endpoint returning runtime lane configuration
- [SCOPE-06] Serde serialization for TurnEvent variants

### Out of Scope

- [SCOPE-07] Web frontend (separate epic)
- [SCOPE-08] Trace replay endpoints (separate epic)
- [SCOPE-09] Authentication or multi-user support
- [SCOPE-10] HTTPS/TLS termination

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Axum server starts on a configurable port when paddles launches | GOAL-01 | must | Foundation for all HTTP access |
| FR-02 | POST /sessions creates a ConversationSession and returns its task_id | GOAL-02 | must | Session lifecycle management |
| FR-03 | POST /sessions/:id/turns accepts a prompt and processes it through MechSuitService | GOAL-02 | must | Core turn submission |
| FR-04 | GET /sessions/:id/events streams TurnEvents as SSE with typed JSON payloads | GOAL-02 | must | Real-time visibility into the recursive loop |
| FR-05 | GET /health returns 200 with runtime lane metadata | GOAL-01 | must | Liveness and configuration introspection |
| FR-06 | TurnEvent enum derives Serialize for SSE payload encoding | GOAL-02 | must | Events must be JSON-serializable |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | HTTP server is an infrastructure adapter implementing TurnEventSink, not a new application layer | GOAL-01 | must | Respects hexagonal architecture |
| NFR-02 | All 90 existing tests continue to pass with the server addition | GOAL-01 | must | No regression to CLI/TUI paths |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Server starts | Integration test or manual curl to /health | 200 response with lane config |
| Turn processing | curl POST to /sessions then /turns, observe SSE stream | Events stream matches CLI output |
| Architecture | Code review: no application layer imports infrastructure | Import graph verification |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| axum + tower are compatible with existing tokio runtime | Would need alternative HTTP framework | axum 0.8 runs on tokio, which paddles already uses |
| Single MechSuitService instance can serve both CLI and HTTP | Would need process separation | Local model inference is sequential regardless of interface |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Port configuration: CLI flag vs config file vs default | operator | Resolved: --port flag with default 3000 |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] curl POST /sessions/:id/turns returns SSE stream of TurnEvents followed by the final response
- [ ] Server runs alongside TUI/CLI without interference
<!-- END SUCCESS_CRITERIA -->
