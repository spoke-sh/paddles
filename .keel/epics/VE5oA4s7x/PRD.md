# Interactive Prompt TUI - Product Requirements

## Problem Statement

Users currently have to provide a single prompt via CLI arguments. To build capacity for complex agentic workflows, we need an interactive mode that stays open and allows for a turn-based conversation.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Implement interactive prompt loop | `just paddles` opens an input prompt | 100% |
| GOAL-02 | Multi-turn agentic support | Users can send multiple messages in one session | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Main user of Paddles | Fluid interaction with the local AI model |

## Scope

### In Scope

- [SCOPE-01] Basic interactive loop in `main.rs` using `stdin`.
- [SCOPE-02] Shared session state across multiple prompts.
- [SCOPE-03] Integration with existing `CandleProvider`.

### Out of Scope

- [SCOPE-04] Full TUI library integration (e.g. `ratatui`) - sticking to a simple "prompt" style for now.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Enter interactive mode when no prompt is provided. | GOAL-01 | must | Core UX requirement. |
| FR-02 | Maintain conversation history in the session. | GOAL-02 | must | Enables multi-turn tasks. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Prompt for user input with a clear indicator (e.g. `>>`). | GOAL-01 | must | Basic TUI feedback. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Manual test: Run `just paddles`, send 2 prompts, verify they both get model responses.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| A-01 | `wonopcode-core::PromptLoop` handles session persistence correctly. | Required for multi-turn. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | `stdin` blocking behavior | Ensure proper async handling in `main.rs`. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `just paddles` starts a conversation.
- [ ] Multiple prompts can be sent in one session.
<!-- END SUCCESS_CRITERIA -->
