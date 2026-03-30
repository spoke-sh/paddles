# Web Chat Interface - Product Requirements

## Problem Statement

The only interactive interfaces are terminal-based. A browser chat UI needs to consume SSE turn events and render the conversation with the same fidelity as the TUI, reflecting real-time planner actions, tool calls, and synthesis.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | A browser-based chat interface renders conversations from paddles SSE events | User can submit a prompt and see the full recursive turn play out in the browser | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles locally | Interact with paddles through a richer visual interface than the terminal |

## Scope

### In Scope

- [SCOPE-01] HTML/JS chat page served by the paddles axum server
- [SCOPE-02] Prompt input that POSTs to /sessions/:id/turns
- [SCOPE-03] SSE EventSource connection rendering TurnEvents as they arrive
- [SCOPE-04] Message bubbles for user prompts and assistant responses
- [SCOPE-05] Collapsible event timeline showing planner actions, tool calls, and gatherer results

### Out of Scope

- [SCOPE-06] Framework-heavy SPA (React, Vue, etc.) - vanilla JS/HTML is sufficient
- [SCOPE-07] Trace visualization (separate epic)
- [SCOPE-08] Multi-session management UI

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | GET / serves a self-contained HTML chat page with embedded JS and CSS | GOAL-01 | must | Zero-dependency browser client |
| FR-02 | Chat page connects to SSE endpoint and renders TurnEvents in real time | GOAL-01 | must | Core real-time feedback loop |
| FR-03 | Each TurnEvent type renders with appropriate visual treatment | GOAL-01 | must | Operator sees planner actions, tool calls, synthesis distinctly |
| FR-04 | Final assistant response renders as a styled message bubble | GOAL-01 | must | Conversational UX |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Chat page is a single self-contained HTML file with no external dependencies | GOAL-01 | must | Simplicity and portability |
| NFR-02 | Page works in modern browsers without build tooling | GOAL-01 | must | No npm/webpack/etc required |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Chat renders | Manual: open browser, submit prompt, observe event stream | Screenshot or operator confirmation |
| Event fidelity | Compare browser event timeline with CLI output for same prompt | Events match |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Modern browsers support EventSource API for SSE | Would need WebSocket fallback | EventSource is supported in all modern browsers |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the chat page auto-create a session or require explicit creation | operator | Resolved: auto-create on page load |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Browser chat page renders a full recursive turn with visible planner actions and tool calls
- [ ] Assistant response appears as a styled message after synthesis completes
<!-- END SUCCESS_CRITERIA -->
