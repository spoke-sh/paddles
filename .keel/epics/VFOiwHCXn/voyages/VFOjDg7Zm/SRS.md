# Planner Reasoning Events And TUI Rendering - SRS

## Summary

Epic: VFOiwHCXn
Goal: Emit verbosity-tiered planner reasoning events during the recursive loop and render them in the TUI with in-place progress at verbose=0 and reasoning detail at verbose=1+

## Scope

### In Scope

- [SCOPE-01] New TurnEvent for planner step progress (verbose=0 tier, in-place updates)
- [SCOPE-02] Enriched PlannerActionSelected rendering with human-readable reasoning at verbose=1
- [SCOPE-03] Branch and refine decision reasoning events at verbose=1
- [SCOPE-04] Budget consumption indicators (steps used/limit, evidence count) at verbose=1
- [SCOPE-05] Full planner graph state rendering at verbose=2

### Out of Scope

- [SCOPE-06] Cancellation of in-progress planner loops
- [SCOPE-07] Changing the planner's actual decision-making logic
- [SCOPE-08] Progress percentage estimation

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | TurnEvent::PlannerStepProgress variant with step_number, step_limit, action, and query fields | SCOPE-01 | FR-01 | manual |
| SRS-02 | PlannerStepProgress has min_verbosity 0 (always visible) | SCOPE-01 | FR-01 | manual |
| SRS-03 | event_type_key returns "planner_step_progress" | SCOPE-01 | FR-01 | manual |
| SRS-04 | PlannerStepProgress emitted at the start of each planner loop iteration in execute_recursive_planner_loop | SCOPE-01 | FR-01 | manual |
| SRS-05 | TUI renders PlannerStepProgress in-place (replacing previous progress row) like GathererSearchProgress | SCOPE-01 | FR-02 | manual |
| SRS-06 | Planner step progress and search progress coexist — search progress replaces within the step progress row or alongside it | SCOPE-01 | FR-02 | manual |
| SRS-07 | format_turn_event_row renders PlannerStepProgress as "Step N/M: action — query" at verbose=0 | SCOPE-01 | FR-02 | manual |
| SRS-08 | At verbose=1, PlannerActionSelected renders with collapsed rationale and the specific query or command target | SCOPE-02 | FR-03 | manual |
| SRS-09 | At verbose=1, after each gather/refine, emit a compact evidence outcome showing items found and top source | SCOPE-03 | FR-04 | manual |
| SRS-10 | At verbose=1, branch and refine actions include a one-line explanation of why the planner chose that action | SCOPE-03 | FR-05 | manual |
| SRS-11 | PlannerStepProgress includes evidence_count and step budget info for verbose=1+ rendering | SCOPE-04 | FR-07 | manual |
| SRS-12 | At verbose=2, PlannerSummary includes graph node count, edge count, and active branch retained artifact count | SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Progress events add no measurable latency to the planner step cycle | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | At verbose=0, at most one in-place progress line during the entire planner loop | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-03 | At verbose=1, each step renders in 2-3 lines maximum | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
