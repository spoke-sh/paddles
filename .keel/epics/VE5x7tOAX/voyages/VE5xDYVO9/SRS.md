# Auth and Default Stabilization - SRS

## Summary

Epic: VE5x7tOAX
Goal: Switch to non-gated default and implement token support.

## Scope

### In scope

- [SCOPE-01] Changing default model to `qwen-1.5b`.
- [SCOPE-02] Adding `--hf-token` CLI argument.
- [SCOPE-03] Supporting `HF_TOKEN` environment variable.

### Out of scope

- [SCOPE-04] Multiple account management.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-23] | Set Qwen-1.5B as default in `main.rs` | SCOPE-01 | FR-01 | manual |
| [SRS-24] | Add `hf_token` field to `BootContext` and `Cli` | SCOPE-02 | FR-02 | manual |
| [SRS-25] | Pass token to `HFHubAdapter` initialization | SCOPE-02 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-NFR-10] | Mask token in all logs and outputs | SCOPE-02 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
