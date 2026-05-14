# Adopt HTTP-Only Inference Decision - SRS

## Summary

Epic: VJZ034dF2
Goal: Record the HTTP-only inference ADR, compatibility policy, and explicit decision that legacy Sift model-provider config hard-fails with an Ollama HTTP migration hint.

## Scope

### In Scope

- [SCOPE-01] Add an ADR under `.keel/adrs/` that declares paddles model inference HTTP-only for action selection and final rendering.
- [SCOPE-02] Define the legacy Sift model-provider compatibility policy as an explicit hard failure with an `ollama:<model>` migration hint.
- [SCOPE-03] Add tests or document checks that prevent reintroducing paddles-owned model loading as a supported inference path.
- [SCOPE-04] Update owning architecture/configuration docs only for the decision and compatibility policy introduced in this voyage.

### Out of Scope

- [SCOPE-05] Runtime factory rewiring for HTTP-only model clients.
- [SCOPE-06] Deleting Sift inference adapters or dependencies.
- [SCOPE-07] Renaming planner/synthesizer/gatherer internals.
- [SCOPE-08] Removing Sift retrieval/indexing.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The ADR states that action-selection and final-rendering inference must use HTTP model clients and that local models are hosted outside paddles behind HTTP services. | SCOPE-01 | FR-01 | ADR review and doc check |
| SRS-02 | Legacy `sift` model-provider config fails before runtime construction with an actionable migration hint naming `ollama:<model>`. | SCOPE-02 | FR-02 | automated config tests |
| SRS-03 | Architecture and configuration docs point to the ADR and stop presenting in-process model loading as the future-supported inference path. | SCOPE-03, SCOPE-04 | FR-01 | doc check |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Compatibility failures are deterministic, specific to the legacy Sift model-provider path, and do not affect Sift retrieval/indexing. | SCOPE-02 | NFR-02 | automated config and retrieval boundary tests |
| SRS-NFR-02 | The ADR and docs use `ollama:<model>` as the canonical local HTTP inference form without naming a fixed default model. | SCOPE-01, SCOPE-04 | NFR-03 | doc review |
| SRS-NFR-03 | Each implementation story starts with a failing test or doc check before runtime behavior or owning docs are changed. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | story evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
