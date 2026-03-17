# Sift Docking Integration - Product Requirements

## Problem Statement

The current implementation manually handles Hugging Face downloads and Candle inference loops. The `sift` crate already provides these capabilities in its internal adapters. We should leverage `sift` to reduce maintenance overhead and improve robustness.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Delegate Model Acquisition to Sift | `sift::internal::search::adapters` handles downloads | 100% |
| GOAL-02 | Delegate Inference to Sift | `sift::GenerativeModel::generate` performs inference | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Maintaining Paddles | Less code to maintain for core inference |

## Scope

### In Scope

- [SCOPE-01] Refactoring `ModelRegistry` to use `sift`.
- [SCOPE-02] Refactoring `InferenceEngine` to wrap `sift::GenerativeModel`.
- [SCOPE-03] Using `sift`'s `QwenReranker` or `GemmaAdapter` for actual work.

### Out of Scope

- [SCOPE-04] Enhancing `sift` itself.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | System must use `sift` to download model assets. | GOAL-01 | must | Leverages existing reliable code. |
| FR-02 | System must use `sift`'s inference implementation. | GOAL-02 | must | Simplifies the mech suit's core. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain 100% functional parity with `--prompt`. | GOAL-02 | must | Zero-regression refactor. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- CLI Proof: `just paddles` generates text using the `sift`-backed engine.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| `sift::internal` APIs are accessible and stable enough for our use. | We are using a specific git version of sift. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | Sync vs Async mismatch | Wrap sift's sync `generate` in an async stream. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `main.rs` contains no manual HF Hub or Candle loop code.
- [ ] `just paddles` returns valid AI responses.
<!-- END SUCCESS_CRITERIA -->
