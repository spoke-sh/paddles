# Stream Tool Output And Drop The 1.2k Cap - SRS

## Summary

Epic: VI2sHovAf
Goal: Replace buffered process::Output capture in planner_action_execution.rs with streamed stdout/stderr pipes that fan out to TurnEventSink and the planner request as bytes arrive; remove the trim_for_planner(_, 1_200) cap; raise any planner-bound budget to 32k+ with head+tail truncation; keep raw output uncut in the trace recorder.

## Scope

### In Scope

- [SCOPE-01] Replace `process::Output`-based shell and inspect command capture in `src/application/planner_action_execution.rs` and `src/infrastructure/terminal.rs` with streamed stdout/stderr pipes (`tokio::process::Command` with piped child stdio).
- [SCOPE-02] Forward streamed output chunks to the operator `TurnEventSink` as they arrive so the TUI shows live tool output instead of a frozen step.
- [SCOPE-03] Drop the `trim_for_planner(&rendered, 1_200)` cap from the planner-bound summary; if a budget is needed for context size, raise it to 32k+ and apply head+tail truncation with a marker, only on the planner-bound copy.
- [SCOPE-04] Persist the full, untrimmed output in the trace recorder regardless of what the planner sees.

### Out of Scope

- [SCOPE-05] Streaming for non-shell tools (semantic queries, diff, search) beyond what falls naturally out of the same plumbing change.
- [SCOPE-06] Adding a new approval / permission UX (handled by future plan-mode work).
- [SCOPE-07] Restructuring the planner action enum or planner request schema.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Shell and inspect tool output must stream to the operator UI and the planner request as bytes arrive, with no character cap below 32k on the planner-bound copy and no cap at all on operator-visible output or on the trace record. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | FR-01 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Streaming must not regress sandbox enforcement, governance decisions, or trace recorder durability for tool invocations. | SCOPE-01, SCOPE-04 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
