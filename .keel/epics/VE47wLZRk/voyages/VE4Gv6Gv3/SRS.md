# Real Chord Integration - SRS

## Summary

Epic: VE47wLZRk
Goal: Actually wire the real wonopcode core engine into paddles.

## Scope

### In scope

- [SCOPE-06] Integration of `wonopcode-core` `Instance` and `PromptLoop`.
- [SCOPE-07] Successful compilation with real dependencies.
- [SCOPE-08] Execution of a real agentic prompt via the CLI.

### Out of scope

- [SCOPE-09] Advanced TUI features from `wonopcode`.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-03 | CLI must instantiate `wonopcode_core::Instance` | SCOPE-06 | FR-01 | manual |
| SRS-04 | CLI must execute `PromptLoop::run` | SCOPE-08 | FR-01 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-02 | System must build with real OpenSSL and dependencies | SCOPE-07 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
