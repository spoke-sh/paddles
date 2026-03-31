# Evidence-Threshold Context Refinement - SRS

## Summary

Epic: VFUNJz9zT
Goal: GOAL-01: Evidence-threshold refinement

## Scope

### In Scope

- [SCOPE-01] Define explicit refinement trigger and policy artifacts used by the planner during an active turn.
- [SCOPE-02] Implement mid-loop context refinement that can update interpretation context while the loop is running.
- [SCOPE-03] Record refinement actions as structured turn-level events in the trace stream.
- [SCOPE-04] Add cooldown and oscillation-prevention behavior so repeated refinements stay bounded and stable.

### Out of Scope

- [SCOPE-05] Reworking offline replay, long-term evidence storage, or non-interpreter policy engines.
- [SCOPE-06] Expanding the turn model to include policy authoring workflows outside this mission.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The system defines `RefinementTrigger` and `RefinementPolicy` types with stable ids and explicit metadata used by planner/runtime checks. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The planner can perform a mid-loop interpretation context refinement when a trigger becomes active. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The system emits a `RefinementApplied` trace event whenever a refinement is applied. | SCOPE-03 | FR-03 | manual |
| SRS-04 | The system applies a cooldown window and oscillation guard to prevent rapid back-and-forth refinements. | SCOPE-04 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Mid-loop refinement must fail closed when policy or trigger inputs are incomplete, with no change to existing planner execution state. | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Trace event emission must not block turn completion and should degrade to warning telemetry if tracing is unavailable. | SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
