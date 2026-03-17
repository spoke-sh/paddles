# Registry Implementation - SRS

## Summary

Epic: VE5ttmBfz
Goal: Connect CandleAdapter to real Hugging Face models.

## Scope

### In scope

- [SCOPE-01] Integration of `hf-hub` crate.
- [SCOPE-02] Implementation of Gemma or Qwen loading logic in `CandleAdapter`.
- [SCOPE-03] Model caching strategy.
- [SCOPE-04] CLI argument for model selection.

### Out of scope

- [SCOPE-05] Support for every model family.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-20] | Implement `ModelRegistry` port for HF Hub | SCOPE-01 | FR-01 | manual |
| [SRS-21] | Extend `CandleAdapter` to load real weights | SCOPE-02 | FR-02 | manual |
| [SRS-22] | Add `--model` CLI argument | SCOPE-04 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-NFR-09] | Use `indicatif` for download progress | SCOPE-01 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
