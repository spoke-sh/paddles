# Axum Server With SSE Turn Events - SRS

## Summary

Epic: VFKBCVjpo
Goal: Deliver the axum HTTP server with session endpoints and SSE-streamed TurnEvents

## Scope

### In Scope

- [SCOPE-01] Axum server starting on a configurable localhost port
- [SCOPE-02] Session creation and turn submission endpoints
- [SCOPE-03] SSE streaming of TurnEvents during turn processing
- [SCOPE-04] Health endpoint returning runtime configuration
- [SCOPE-05] Serde Serialize derive on TurnEvent

### Out of Scope

- [SCOPE-06] Web frontend HTML pages
- [SCOPE-07] Trace replay or graph endpoints
- [SCOPE-08] Authentication

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Axum server starts on configurable port when paddles launches with --port flag | SCOPE-01 | FR-01 | manual |
| SRS-02 | GET /health returns 200 with JSON runtime lane metadata | SCOPE-05 | FR-05 | manual |
| SRS-03 | POST /sessions creates a ConversationSession and returns session_id as JSON | SCOPE-02 | FR-02 | manual |
| SRS-04 | POST /sessions/:id/turns accepts JSON prompt body and processes through MechSuitService | SCOPE-03 | FR-03 | manual |
| SRS-05 | GET /sessions/:id/events returns SSE stream of TurnEvents as typed JSON | SCOPE-04 | FR-04 | manual |
| SRS-06 | TurnEvent enum derives Serialize for JSON encoding | SCOPE-05 | FR-06 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | HTTP server is an infrastructure adapter, no application layer changes needed | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | All existing tests pass unchanged | SCOPE-01 | NFR-02 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
