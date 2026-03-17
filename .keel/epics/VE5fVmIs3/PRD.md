# Agentic Loop Wiring - Product Requirements

## Problem Statement

The current `paddles` implementation uses placeholders for the agentic loop instead of the real `PromptLoop` from `wonopcode-core`. This prevents the "mech suit" from actually performing agentic tasks.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Wire the real `PromptLoop` | `paddles --prompt` executes via `PromptLoop` | 100% |
| GOAL-02 | Robust session handling | Each prompt runs in a valid `Session` | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Human user of Paddles | Actual agentic task execution |

## Scope

### In Scope

- [SCOPE-01] Instantiating `PromptLoop` with all required dependencies.
- [SCOPE-02] Orchestrating the `run` loop in `main.rs`.
- [SCOPE-03] Handling `PromptResult` and displaying the final text.

### Out of Scope

- [SCOPE-04] Advanced streaming output (future epic).
- [SCOPE-05] Complex tool registration (future epic).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `main.rs` must correctly construct a `PromptLoop`. | GOAL-01 | must | Required for execution. |
| FR-02 | `main.rs` must execute the loop and handle the result. | GOAL-01 | must | Required for user output. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Loop initialization must be traced. | GOAL-01 | must | For observability. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- CLI proof: `paddles --prompt "hello"` returns a real AI response.

## Assumptions

- `wonopcode-core` APIs are sufficiently public for this integration.

## Open Questions & Risks

- Risk: Missing dependencies for `PromptLoop` construction (e.g. `LanguageModel` provider).

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles --prompt "write a comment in src/lib.rs"` actually executes.
<!-- END SUCCESS_CRITERIA -->
