# Single Recursive Control Plane - SRS

## Summary

Epic: VHURpL4nG
Goal: Collapse nested adapter tool loops into the application harness and route workspace actions through explicit execution boundaries rather than synthesis ports.

## Scope

### In Scope

- [SCOPE-01] Application-owned execution boundary for planner-selected workspace actions
- [SCOPE-02] Removing workspace mutation methods from synthesizer authoring ports
- [SCOPE-03] Retiring adapter-owned repository tool loops for Sift, HTTP, and equivalent model integrations
- [SCOPE-04] Keeping budgets, retries, governance, and evidence recording under one recursive harness path

### Out of Scope

- [SCOPE-05] Reworking final-answer render persistence beyond the control-plane seam
- [SCOPE-06] Large-scale application service extraction unrelated to loop ownership
- [SCOPE-07] New model-provider features unrelated to removing duplicate repository tool loops

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Planner-selected workspace actions must execute through an explicit application-owned executor boundary rather than through `SynthesizerEngine`. | SCOPE-01 | FR-03 | test |
| SRS-02 | The synthesizer authoring port must only author responses and supply synthesis-context helpers; it must not mutate workspace state directly. | SCOPE-02 | FR-03 | review |
| SRS-03 | Model adapters must not run independent recursive repository tool loops once the application harness owns the recursive control plane. | SCOPE-03 | FR-04 | test |
| SRS-04 | Budgets, retries, execution-governance decisions, and evidence recording for repository actions must be emitted from the single application-owned recursive loop. | SCOPE-01, SCOPE-03, SCOPE-04 | FR-04 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The new executor path must preserve existing execution-governance visibility and local-first constraints. | SCOPE-01, SCOPE-04 | NFR-02 | review |
| SRS-NFR-02 | The refactor must remove duplicate ownership of retry/stop logic rather than layering a second control surface beside the first. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | review |
| SRS-NFR-03 | The new control-plane path must remain testable with deterministic turn-loop tests that assert one owner for repository actions. | SCOPE-01, SCOPE-03, SCOPE-04 | NFR-03 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
