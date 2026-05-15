# Unify First Action Entry Point - SRS

## Summary

Epic: VJeQx1O20
Goal: Normal turn execution enters execute_agent_loop before any model-selected action; the first loop iteration handles direct answers, stops, and workspace actions.

## Scope

### In Scope

- [SCOPE-01] Route normal turn execution into `execute_agent_loop` before any model-selected turn action is accepted.
- [SCOPE-02] Replace the pre-loop initial action decision with the first loop iteration.
- [SCOPE-03] Preserve direct-answer and stop behavior as loop outcomes.
- [SCOPE-04] Remove `PromptExecutionPlan` and `PromptExecutionPath` from the normal runtime path.

### Out of Scope

- [SCOPE-05] Provider wire schema cleanup that is not required for this entry-point migration.
- [SCOPE-06] Replacing execution governance or final response rendering.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `process_prompt_in_session_with_mode_request_and_sink` must call `execute_agent_loop` before any model-selected workspace/direct-answer/stop action is executed or accepted for normal turns. | SCOPE-01 | FR-01 | focused test |
| SRS-02 | The first agent-loop iteration must be able to produce direct answer, stop, and workspace actions without a separate `select_initial_action` call. | SCOPE-02, SCOPE-03 | FR-01 | focused test |
| SRS-03 | Existing direct-answer and final-rendering behavior must remain available through `AgentLoopOutcome`. | SCOPE-03 | FR-05 | focused test |
| SRS-04 | `PromptExecutionPlan` and `PromptExecutionPath` must be deleted or reduced to non-runtime compatibility scaffolding. | SCOPE-04 | FR-02 | static search |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The migration must preserve trace events that explain selected, blocked, retried, and stopped actions. | SCOPE-01, SCOPE-03 | NFR-02 | focused test |
| SRS-NFR-02 | Full library tests must pass after the entry-point migration. | SCOPE-01 | NFR-01 | `cargo test --lib` |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
