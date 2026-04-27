# Extract Planner Executor Boundaries - SRS

## Summary

Epic: VI1zeXMOr
Goal: Move the recursive planner executor loop, planner action execution helpers, and external capability execution helpers out of the planner orchestration module while preserving behavior.

## Scope

### In Scope

- [SCOPE-01] Extract the recursive planner executor loop from `src/application/mod.rs` into the recursive control chamber.
- [SCOPE-02] Extract planner action execution helpers from `src/application/mod.rs` into a dedicated application module.
- [SCOPE-03] Extract external-capability execution helpers from `src/application/mod.rs` into a dedicated application module.
- [SCOPE-04] Add focused regression tests and run repository verification.

### Out of Scope

- [SCOPE-05] Changing planner decision policy, executor governance policy, or external-capability availability semantics.
- [SCOPE-06] Rewriting provider adapters such as HTTP or Sift planners.
- [SCOPE-07] Introducing new external tools, services, or network dependencies.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The recursive planner executor loop lives in the recursive control chamber, not as a direct `MechSuitService` method in `src/application/mod.rs`. | SCOPE-01 | FR-01 | test |
| SRS-02 | Planner action execution helpers for query/evidence-source mapping and governed terminal command execution live outside `src/application/mod.rs`. | SCOPE-02 | FR-02 | test |
| SRS-03 | External-capability invocation formatting, governed invocation, result summarization, and evidence projection live outside `src/application/mod.rs`. | SCOPE-03 | FR-03 | test |
| SRS-04 | Existing runtime service APIs and planner action behavior remain unchanged after extraction. | SCOPE-04 | FR-04 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Execution governance decisions and evidence records continue to be emitted for terminal, workspace, and external-capability actions. | SCOPE-02 | NFR-01 | test |
| SRS-NFR-02 | The final proof includes `cargo test` and `keel doctor`. | SCOPE-04 | NFR-02 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
