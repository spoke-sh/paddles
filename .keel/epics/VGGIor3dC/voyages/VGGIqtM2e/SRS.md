# Build Turn Machine Stage For Transit - SRS

## Summary

Epic: VGGIor3dC
Goal: Replace the current transit board and observatory with a simpler machine-stage that shows how a turn moves through steps, diversions, jams, and outputs.

## Scope

### In Scope

- [SCOPE-01] Build the transit machine stage using the shared machine-moment projection.
- [SCOPE-02] Add temporal navigation and selected-moment detail for the transit route.
- [SCOPE-03] Reduce or retire chrome that competes with the machine narrative.

### Out of Scope

- [SCOPE-04] Rewriting the forensic route.
- [SCOPE-05] Changing the underlying trace recorder semantics.
- [SCOPE-06] Retiring raw/internal trace access across the whole runtime.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The transit route must render a machine stage composed from shared machine moments instead of the existing node-grid-first presentation. | SCOPE-01 | FR-02 | manual |
| SRS-02 | The transit route must support bottom temporal scrubbing and selected-moment detail that explain the current machine part in causal terms. | SCOPE-01 | FR-02 | manual |
| SRS-03 | The transit route must visually distinguish diverters, jams, replans, and outputs so operators can see why the turn changed direction. | SCOPE-02 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The default transit surface must be legible without family toggles, dense summary chrome, or separate observatory reading. | SCOPE-03 | NFR-02 | manual |
| SRS-NFR-02 | Transit route tests must guard the machine-stage contract so later UI changes cannot drift back toward raw-node clutter. | SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
