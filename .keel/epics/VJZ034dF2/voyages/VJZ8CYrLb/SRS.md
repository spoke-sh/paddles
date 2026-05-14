# Route Runtime Inference Through HTTP Model Clients - SRS

## Summary

Epic: VJZ034dF2
Goal: Make action-selection and final-rendering model inference resolve exclusively through HTTP model clients while preserving Sift retrieval independence.

## Scope

### In Scope

- [SCOPE-01] Route action-selection model construction through HTTP provider configuration only.
- [SCOPE-02] Route final-rendering model construction through HTTP provider configuration only.
- [SCOPE-03] Remove `ModelPaths` and Sift model-preparation requirements from action-selection and final-rendering runtime preparation.
- [SCOPE-04] Keep Sift retrieval/indexing independently selectable and tested.
- [SCOPE-05] Update docs and tests that describe the runtime inference boundary changed by this voyage.

### Out of Scope

- [SCOPE-06] Deleting Sift inference adapter files and inference-only dependencies.
- [SCOPE-07] Replacing runtime preference file names.
- [SCOPE-08] Renaming all planner/synthesizer/gatherer internal ports.
- [SCOPE-09] Removing Sift retrieval/indexing.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Action-selection runtime construction resolves a model client through the HTTP provider boundary and never requires local `ModelPaths`. | SCOPE-01, SCOPE-03 | FR-03 | automated runtime construction tests |
| SRS-02 | Final-rendering runtime construction resolves a model client through the HTTP provider boundary and never requires local `ModelPaths`. | SCOPE-02, SCOPE-03 | FR-03 | automated runtime construction tests |
| SRS-03 | Legacy Sift model-provider branches fail before runtime construction using the policy from the ADR voyage. | SCOPE-01, SCOPE-02 | FR-02 | automated config tests |
| SRS-04 | Sift retrieval/indexing preparation remains independent from model-client inference and retains its existing behavior. | SCOPE-04 | FR-04 | automated retrieval boundary tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | HTTP provider integration coverage for retries, structured final answers, provider-specific schemas, and credential handling remains green. | SCOPE-01, SCOPE-02 | NFR-04 | existing integration tests |
| SRS-NFR-02 | Runtime errors for unsupported legacy provider combinations remain actionable and occur before model runtime construction. | SCOPE-01, SCOPE-02 | NFR-02 | automated error tests |
| SRS-NFR-03 | Each implementation story starts with a failing test or doc check before runtime behavior or owning docs are changed. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | story evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
