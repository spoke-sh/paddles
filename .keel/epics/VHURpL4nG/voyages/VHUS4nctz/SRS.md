# Canonical Render Truth And Projection Convergence - SRS

## Summary

Epic: VHURpL4nG
Goal: Persist typed authored responses end-to-end and make live/replay stream projections converge on one canonical render/projection contract.

## Scope

### In Scope

- [SCOPE-01] Persist typed `AuthoredResponse` and `RenderDocument` data in the durable completion/checkpoint path
- [SCOPE-02] Replay transcript/render state directly from persisted typed response data instead of reparsing flattened plain text
- [SCOPE-03] Canonical projection update contracts for live stream and replay convergence
- [SCOPE-04] Versioning or reducer semantics that let surfaces detect stale stream state and reconcile deterministically
- [SCOPE-05] Automated convergence tests comparing live emitted render/projection state with replayed state for the same turn

### Out of Scope

- [SCOPE-06] Refactoring recursive control ownership beyond the render/projection seam
- [SCOPE-07] Major UI redesigns for TUI or web presentation
- [SCOPE-08] New remote transport or browser services for event delivery

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Completion records must persist typed authored response data sufficient to reconstruct the final answer render blocks without reparsing assistant plain text. | SCOPE-01 | FR-01 | test |
| SRS-02 | Conversation transcript replay must hydrate assistant render state from the persisted typed response path and preserve response-mode, citation, and grounding metadata. | SCOPE-02 | FR-01 | test |
| SRS-03 | Live projection updates must flow through one canonical projection contract that can rebuild transcript/render state deterministically for a task. | SCOPE-03 | FR-02 | test |
| SRS-04 | Projection consumers must be able to detect stale state and reconcile from ordered reducer or version semantics rather than UI-local render repair heuristics. | SCOPE-03, SCOPE-04 | FR-02 | test |
| SRS-05 | The repository must include automated tests that compare live turn render/projection output with replayed transcript state for the same completed turn. | SCOPE-05 | FR-02 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Structured render persistence must preserve response-mode, citation, and grounding metadata through live and replayed paths. | SCOPE-01, SCOPE-02, SCOPE-05 | NFR-03 | test |
| SRS-NFR-02 | The voyage must retire duplicate render reconstruction paths once the canonical typed response contract is in place. | SCOPE-02, SCOPE-03 | NFR-04 | review |
| SRS-NFR-03 | The render/projection convergence changes must add no new external runtime dependency. | SCOPE-01, SCOPE-03, SCOPE-04 | NFR-01 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
