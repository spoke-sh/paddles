# Stream Uncut Tool Output - Product Requirements

## Problem Statement

Shell, inspect, and other tool outputs are captured to process::Output and trimmed to 1,200 chars by trim_for_planner in src/application/planner_action_execution.rs, which silently slices real cargo build, pytest, grep, and git log output before the planner sees it and makes long commands look frozen in the TUI. Stream output to operator and planner as bytes arrive; raise any planner-bound budget to 32k+ with head+tail truncation; keep raw output uncut in the trace recorder.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Resolve the problem described above for the primary user. | A measurable outcome is defined for this problem | Target agreed during planning |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | The person or team most affected by the problem above. | A clearer path to the outcome this epic should improve. |

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

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Deliver the primary user workflow for this epic end-to-end. | GOAL-01 | must | Establishes the minimum functional capability needed to achieve the epic goal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01 | must | Keeps operations stable and makes regressions detectable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
