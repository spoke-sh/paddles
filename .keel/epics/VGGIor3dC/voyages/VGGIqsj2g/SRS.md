# Define Narrative Machine Model And Shared Projection - SRS

## Summary

Epic: VGGIor3dC
Goal: Define the simplified Rube Goldberg machine mental model, moment projection, and interaction contract so transit and forensic views share one causal vocabulary.

## Scope

### In Scope

- [SCOPE-01] Define the machine-moment vocabulary and the operator-facing labels for those moments.
- [SCOPE-02] Define how transit trace nodes and forensic records are grouped into shared narrative machine parts.
- [SCOPE-03] Define the primary selection and navigation contract shared by transit and forensic routes.

### Out of Scope

- [SCOPE-04] Implementing the full transit UI redesign.
- [SCOPE-05] Implementing the full forensic UI redesign.
- [SCOPE-06] Changing recorder/storage behavior beyond what is necessary to project machine moments.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The voyage must define a shared machine-moment taxonomy that maps the runtime trace into operator-meaningful parts such as inputs, diverters, evidence probes, jams, replans, forces, and outputs. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The voyage must define a projection contract that groups trace graph nodes and forensic records into those machine moments while preserving stable links back to raw ids and payload sources. | SCOPE-02 | FR-01 | manual |
| SRS-03 | The voyage must define one shared operator interaction model for transit and forensic routes: selected turn, selected moment, and optional internals mode. | SCOPE-03 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The shared vocabulary must stay explainable in simple operator terms and avoid leaking raw storage concepts into the default surface. | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-02 | The projection contract must be stable enough to support route-level tests before the larger UI rewrites begin. | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
