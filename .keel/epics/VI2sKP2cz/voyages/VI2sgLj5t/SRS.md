# Promote Plan Mode And Add Slash Command Surface - SRS

## Summary

Epic: VI2sKP2cz
Goal: Promote Collaboration::PlanOnly (or equivalent) to a first-class plan mode in the TUI/CLI mirroring Claude Code and Codex; add /help, /plan, /compact, /cost, /agents, /mcp slash commands alongside the existing /login, /model, /resume.

## Scope

### In Scope

- [SCOPE-01] Promote `Collaboration::PlanOnly` (or equivalent) to a first-class `plan` mode in the TUI and CLI: the operator can enter plan mode, see the proposed action, and approve/decline before any tool runs.
- [SCOPE-02] Add `/plan` slash command to enter plan mode and `/help` to enumerate available commands.
- [SCOPE-03] Add `/compact`, `/cost`, `/agents`, `/mcp` slash commands as discoverable surfaces, even if some are stubs that report "not yet wired" for capability that hasn't shipped.
- [SCOPE-04] Surface governance denials as visible TUI prompts (or inline notices) instead of silently summarizing them back to the planner.

### Out of Scope

- [SCOPE-05] Implementing real MCP server discovery / invocation (the `/mcp` command may report unavailability).
- [SCOPE-06] Implementing real subagent execution (the `/agents` command may report unavailability).
- [SCOPE-07] Cost model accounting beyond what the existing trace recorder already exposes.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The TUI exposes `plan` mode end-to-end (entry, proposed action review, approve/decline) and recognizes `/help`, `/plan`, `/compact`, `/cost`, `/agents`, `/mcp` slash commands alongside the existing `/login`, `/model`, `/resume`. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-01 | automated + manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | New slash commands and plan mode must not change non-interactive CLI behavior, service-mode HTTP surfaces, or governance enforcement defaults. | SCOPE-01, SCOPE-04 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
