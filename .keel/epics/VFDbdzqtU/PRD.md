# Codex-Style Interactive TUI - Product Requirements

## Problem Statement

Paddles still uses a plain stdin/stdout REPL, so interactive use lacks the transcript structure, styling, event visibility, and progressive rendering expected from a Codex-like terminal experience.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make interactive mode feel like a transcript-oriented coding agent instead of a line-based shell loop. | Interactive turns render inside a dedicated TUI transcript with persistent history and no raw `>>` prompt loop | Verified live transcript and story proofs |
| GOAL-02 | Visually distinguish operator, assistant, and action/event output so turn flow is easy to scan. | User turns, assistant turns, and action/event rows render with distinct styling and structure | Verified snapshots/transcripts and renderer tests |
| GOAL-03 | Surface live execution progress inside the TUI without forcing users to read debug logs. | Turn events and final assistant answers appear in the transcript as they happen, with progressive rendering for assistant output | Verified transcript proofs and integration tests |
| GOAL-04 | Preserve current paddles runtime behavior while improving presentation. | `--prompt` one-shot mode remains plain text, and controller/gatherer/tool behavior still passes quality gates | Verified CLI/test proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive Operator | A developer using `paddles` as a local coding assistant in an interactive shell session. | A readable, trustworthy transcript that feels alive during tool use and answer generation. |
| Scripted Operator | A developer using `paddles --prompt` in scripts or one-shot terminal usage. | Stable, non-TUI output that remains easy to pipe and parse. |

## Scope

### In Scope

- [SCOPE-01] Replace the legacy interactive stdin loop with a TUI built around transcript state and terminal lifecycle management.
- [SCOPE-02] Render styled transcript cells for user turns, assistant turns, and action/event rows.
- [SCOPE-03] Surface live paddles turn events in the interactive transcript and render assistant answers progressively once available.
- [SCOPE-04] Preserve plain one-shot CLI behavior and document/prove the new interactive UX.

### Out of Scope

- [SCOPE-05] Replacing the controller, gatherer, or model runtime architecture.
- [SCOPE-06] Shipping a full clone of Codex’s multi-pane TUI or importing Codex internals directly.
- [SCOPE-07] Adding remote-only UI dependencies or browser-based interfaces.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Interactive mode must run through a dedicated terminal UI instead of the legacy raw stdin/stdout loop. | GOAL-01 | must | This is the core user-facing change. |
| FR-02 | The transcript must distinguish user, assistant, and action/event output with intentional styling and structure. | GOAL-01, GOAL-02 | must | Users need speaker and action separation at a glance. |
| FR-03 | Live turn events emitted by the controller must appear in the interactive transcript as execution happens. | GOAL-03 | must | Visibility is the core trust signal for the new UX. |
| FR-04 | Final assistant answers must render progressively inside the TUI while preserving the final grounded/cited content. | GOAL-03 | must | The interaction should feel alive rather than blocked until the whole response completes. |
| FR-05 | One-shot `--prompt` execution must remain plain terminal output outside the TUI path. | GOAL-04 | must | Scripts and simple shell use must not regress. |
| FR-06 | Foundational docs and proof artifacts must describe and demonstrate the new interactive experience. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The UX shift needs durable operator guidance and regression evidence. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The interactive TUI must remain local-first and add no mandatory network dependencies. | GOAL-04 | must | Presentation changes must not alter runtime trust boundaries. |
| NFR-02 | Terminal setup and teardown must restore raw mode and screen state cleanly on exit or error. | GOAL-01 | must | Broken terminal state is unacceptable for a CLI tool. |
| NFR-03 | The TUI architecture should stay small and paddles-owned rather than importing a large opinionated app framework. | GOAL-04 | should | We want Codex-like structure without excessive coupling. |
| NFR-04 | Interactive rendering must remain readable on light and dark terminal backgrounds. | GOAL-02 | should | Visual polish cannot assume one terminal theme. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Interactive runtime | Integration tests plus one-shot CLI checks | Story proofs and commit-hook test runs |
| Visual transcript behavior | Manual transcript proofs and focused renderer tests | Story evidence logs and voyage proof artifact |
| Documentation outcome | Doc review plus transcript examples | Updated foundational docs and proof artifact |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A slimmer paddles-owned TUI can deliver the desired UX without importing Codex wholesale. | We may overbuild or underbuild the UI shell. | Validate during SDD and implementation. |
| Progressive assistant rendering based on finalized output is acceptable until true model token streaming exists. | Users may still want deeper streaming later. | Document current behavior and leave model streaming as follow-on work. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Terminal behavior may differ across shells or CI environments. | Epic owner | Mitigate with clean restore logic and tests |
| The current model backend does not expose true token streaming. | Epic owner | Accepted for this slice; use progressive rendering over final output |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Interactive mode uses the TUI transcript path instead of the legacy loop.
- [ ] User, assistant, and action/event rows are visually distinct and readable.
- [ ] Live turn events and final answers are visible in the TUI without debug logs.
- [ ] One-shot `--prompt` mode remains plain and quality gates stay green.
<!-- END SUCCESS_CRITERIA -->
