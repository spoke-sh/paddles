# Integrate Resolver Into Edit Convergence - SRS

## Summary

Epic: VGDNcabks
Goal: Thread deterministic entity resolution through the planner loop so edit-oriented turns validate targets, converge sooner, and explain misses instead of hallucinating paths or stalling in repeated search/read steps.

## Scope

### In Scope

- [SCOPE-01] Planner-loop integration that consults deterministic resolution before broad search or edit-state actions on known-edit turns.
- [SCOPE-02] Steering and fallback gates that treat resolver outcomes as first-class convergence pressure.
- [SCOPE-03] Runtime/trace diagnostics for resolved, ambiguous, and missing targets.

### Out of Scope

- [SCOPE-04] Rebuilding the resolver backbone itself.
- [SCOPE-05] Full semantic code intelligence beyond the deterministic lookup modes delivered in voyage one.
- [SCOPE-06] Non-edit turns that do not need file-target convergence.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Known-edit and edit-pressured planner turns invoke deterministic resolution before repeating broad search, inspect, or malformed patch attempts once likely target evidence exists. | SCOPE-01 | FR-03 | cargo nextest |
| SRS-02 | Planner/controller state promotes resolved authored targets into read/diff/edit actions and records deterministic ambiguity or miss outcomes when no safe target exists. | SCOPE-01, SCOPE-03 | FR-03 | cargo nextest |
| SRS-03 | Runtime events and traces expose enough resolver outcome detail for operators to understand why a target resolved, remained ambiguous, or missed. | SCOPE-03 | FR-04 | cargo nextest, web tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Resolver integration must fail closed: unresolved or non-authored targets cannot silently proceed into workspace mutation. | SCOPE-02 | NFR-01 | cargo nextest |
| SRS-NFR-02 | Resolver-backed convergence must reduce repeated search/inspect churn without hiding planner rationale or making miss states opaque. | SCOPE-01, SCOPE-03 | NFR-03 | cargo nextest, manual trace proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
