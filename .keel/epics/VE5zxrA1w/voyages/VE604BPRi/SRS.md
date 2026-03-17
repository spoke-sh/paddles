# Sift Implementation Transition - SRS

## Summary

Epic: VE5zxrA1w
Goal: Migrate registry and inference to sift crate.

## Scope

### In scope

- [SCOPE-01] Refactoring `ModelRegistry` to use `sift`.
- [SCOPE-02] Refactoring `InferenceEngine` to wrap `sift::GenerativeModel`.
- [SCOPE-03] Using `sift`'s `QwenReranker` or `GemmaAdapter`.

### Out of scope

- [SCOPE-04] Enhancing `sift`.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-26] | Implement `SiftRegistryAdapter` | SCOPE-01 | FR-01 | manual |
| [SRS-27] | Implement `SiftInferenceAdapter` | SCOPE-02 | FR-02 | manual |
| [SRS-28] | Wire `sift` components in `main.rs` | SCOPE-03 | FR-01 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-NFR-11] | No manual `hf-hub` usage in `paddles` | SCOPE-01 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
