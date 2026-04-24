# Install Codex-Grade Execution Policy - SRS

## Summary

Epic: VHkfpJJc4
Goal: Add an expressive command and tool policy engine beneath Paddles governance so shell, edit, patch, and external actions can be allowed, prompted, denied, or retried with typed evidence.

## Scope

### In Scope

- [SCOPE-02] Add a Codex-grade execution policy layer under Paddles governance, including prefix decisions, denial evidence, and operator-visible posture.

### Out of Scope

- [SCOPE-10] Replacing OS-level sandboxing in this voyage.
- [SCOPE-10] Hidden escalation around the configured approval policy.
- [SCOPE-11] A policy language that requires network access.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define a typed execution policy model and evaluator for command/tool decisions. | SCOPE-02 | FR-02 | test: policy evaluator fixtures |
| SRS-02 | Route shell, edit, patch, and external capability decisions through the evaluator before execution. | SCOPE-02 | FR-02 | test: gate integration tests |
| SRS-03 | Emit denial, prompt-required, allowed, and on-failure evidence through existing governance events. | SCOPE-02 | FR-02 | test: governance projection tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Policy defaults are conservative and compatible with existing local-first profiles. | SCOPE-02 | NFR-01 | test: default profile fixtures |
| SRS-NFR-02 | Policy evaluation is deterministic and does not depend on model output. | SCOPE-02 | NFR-03 | test: pure evaluator tests |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
