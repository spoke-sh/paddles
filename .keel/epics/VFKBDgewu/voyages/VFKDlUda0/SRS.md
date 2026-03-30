# Browser Chat Page - SRS

## Summary

Epic: VFKBDgewu
Goal: Deliver a self-contained HTML chat interface consuming SSE turn events

## Scope

### In Scope

- [SCOPE-01] HTML/JS chat page served by the paddles axum server via include_str!
- [SCOPE-02] Prompt input that POSTs to /sessions/:id/turns
- [SCOPE-03] SSE EventSource connection rendering TurnEvents as they arrive
- [SCOPE-04] Message bubbles for user prompts and assistant responses
- [SCOPE-05] Collapsible event timeline showing planner actions, tool calls, and gatherer results

### Out of Scope

- [SCOPE-06] Framework-heavy SPA (React, Vue, etc.) - vanilla JS/HTML only
- [SCOPE-07] Trace visualization (separate epic VFKBFMq8J)
- [SCOPE-08] Multi-session management UI

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | GET / serves a self-contained HTML chat page with embedded JS and CSS via include_str!. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Chat page opens an EventSource to the SSE endpoint and renders TurnEvents in real time as they arrive. | SCOPE-03 | FR-02 | manual |
| SRS-03 | Each TurnEvent type (planner action, tool call, gatherer result, synthesis) renders with distinct visual treatment in a collapsible event timeline. | SCOPE-05 | FR-03 | manual |
| SRS-04 | Final assistant response renders as a styled message bubble alongside user prompt bubbles. | SCOPE-04 | FR-04 | manual |
| SRS-05 | Prompt input submits via POST to /sessions/:id/turns and clears after submission. | SCOPE-02 | FR-01 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Chat page must be a single self-contained HTML file with no external dependencies, embedded via include_str!. | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | Page must work in modern browsers without any build tooling (no npm/webpack). | SCOPE-01 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
