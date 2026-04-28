# Promote Plan Mode And Add Slash Command Surface - Software Design Description

> Promote Collaboration::PlanOnly (or equivalent) to a first-class plan mode in the TUI/CLI mirroring Claude Code and Codex; add /help, /plan, /compact, /cost, /agents, /mcp slash commands alongside the existing /login, /model, /resume.

**SRS:** [SRS.md](SRS.md)

## Overview

The TUI ships only `/login`, `/model`, `/resume` (`src/infrastructure/cli/interactive_tui.rs:151-190`) — far short of the slash-command surface operators arrive expecting from Claude Code, Codex, Gemini CLI, and OpenCode. `Collaboration::PlanOnly` exists in the domain model but never reaches the operator as a first-class plan mode: the orchestrator runs whatever the planner returns inline. Governance denials, when they fire, are summarized back into the planner's next request rather than surfaced as a TUI prompt, so users see "command failed" instead of "permission required" and re-run the same prompt expecting different results.

This voyage closes the operator-surface gap to the incumbents in three movements:

1. **Plan mode end-to-end.** Promote `Collaboration::PlanOnly` (final variant chosen during implementation) so the operator can enter plan mode, see the proposed action / edit summary, and approve or decline before any tool runs.
2. **Slash command surface parity.** Add `/help`, `/plan`, `/compact`, `/cost`, `/agents`, `/mcp` alongside the existing `/login`, `/model`, `/resume`. Commands for capability that hasn't shipped (`/agents`, `/mcp`) print an honest "not yet wired" notice instead of being absent.
3. **Visible governance denials.** When `ExecutionGovernanceOutcome::Blocked` returns, surface it to the operator (TUI prompt or inline notice) with the reason and re-issue option, rather than only summarizing into the next planner request.

Real subagent execution and real MCP integration are explicitly out of scope; this voyage carves out the operator surface so the underlying capability can land later without further UX churn.

## Components

- `src/infrastructure/cli/interactive_tui.rs` — extend slash-command parser, render plan-mode review panel, render governance-denial notices.
- `src/application/turn_orchestration.rs` (post-rename: `turn`) — defer execution under plan mode until operator approval; thread approval back into the loop.
- `src/application/planner_action_execution.rs` — emit a `TurnEvent` variant for exposed governance denials.
- `src/infrastructure/web/mod.rs` — mirror the slash-command surface where applicable; deferable to a follow-up story if it adds friction.
- Verification: TUI integration tests covering `/plan` approve/decline, denial-notice surfacing, and absence of tool runs prior to approval.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

<!-- Component relationships, layers, modules -->

## Components

<!-- For each major component: purpose, interface, behavior -->

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
