# Remove In-Process Sift Inference Code - SRS

## Summary

Epic: VJZ034dF2
Goal: Delete paddles-owned Sift model loading and in-process inference adapters after HTTP-only runtime construction is proven, without removing Sift retrieval/indexing.

## Scope

### In Scope

- [SCOPE-01] Delete or retire Sift action-selection and final-rendering inference adapters.
- [SCOPE-02] Remove Sift model-preparation registry behavior used only for inference.
- [SCOPE-03] Remove Candle/Qwen/tokenizer or other inference-only dependencies that become unused.
- [SCOPE-04] Remove docs, build notes, and examples that describe paddles-owned local model loading.
- [SCOPE-05] Keep Sift retrieval/indexing code when it remains independent of inference.

### Out of Scope

- [SCOPE-06] Removing Sift retrieval/indexing.
- [SCOPE-07] Replacing HTTP provider adapters.
- [SCOPE-08] Renaming unrelated turn-loop types.
- [SCOPE-09] Changing model-provider semantics beyond the already-approved hard failure.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Sift inference adapters and inference runtime branches are deleted or made unreachable after HTTP-only construction is proven. | SCOPE-01, SCOPE-02 | FR-06 | compile and targeted tests |
| SRS-02 | Inference-only dependencies are removed from Cargo/build configuration when no remaining code uses them. | SCOPE-03 | FR-06 | cargo checks and dependency review |
| SRS-03 | Documentation no longer instructs users to download, prepare, or load local inference models inside paddles. | SCOPE-04 | FR-06 | doc checks |
| SRS-04 | Sift retrieval/indexing remains covered by tests and is not deleted in this voyage. | SCOPE-05 | FR-04 | retrieval tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The deletion preserves all HTTP provider integration tests for action selection, final rendering, retry, structured answer, and credential behavior. | SCOPE-01, SCOPE-03 | NFR-04 | integration tests |
| SRS-NFR-02 | Removed code paths leave clear compatibility errors rather than panics or missing-match fallthroughs. | SCOPE-01, SCOPE-02 | NFR-02 | automated error tests |
| SRS-NFR-03 | Each implementation story starts with a failing test or doc check before runtime behavior or owning docs are changed. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | story evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
