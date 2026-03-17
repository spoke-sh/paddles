# Core Loop Implementation - SRS

## Summary

Epic: VE5fVmIs3
Goal: Implement the real PromptLoop wiring in main.rs.

## Scope

### In scope

- [SCOPE-01] Instantiating `PromptLoop` with all required dependencies.
- [SCOPE-02] Orchestrating the `run` loop in `main.rs`.
- [SCOPE-03] Handling `PromptResult` and displaying the final text.

### Out of scope

- [SCOPE-04] Advanced streaming output (future epic).
- [SCOPE-05] Complex tool registration (future epic).

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-10 | Correct construction of `PromptLoop` instance | SCOPE-01 | FR-01 | manual |
| SRS-11 | Invocation of `loop.run()` with user prompt | SCOPE-02 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-05 | Trace logs for loop lifecycle | SCOPE-01 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
