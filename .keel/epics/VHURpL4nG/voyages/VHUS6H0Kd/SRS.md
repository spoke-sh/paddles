# Chamber Services And Read-Model Boundaries - SRS

## Summary

Epic: VHURpL4nG
Goal: Decompose orchestration into chamber-aligned application services and move projection and presentation concerns out of the domain model.

## Scope

### In Scope

- [SCOPE-01] Extract chamber-aligned application services or modules for turn orchestration responsibilities now concentrated in `MechSuitService`
- [SCOPE-02] Move conversation transcript/forensic/manifold/trace projections into an application-owned read-model boundary
- [SCOPE-03] Move runtime event presentation formatting/projectors out of `domain/model`
- [SCOPE-04] Preserve current recorder, replay, and surface outputs while ownership moves

### Out of Scope

- [SCOPE-05] New product features unrelated to architectural boundary cleanup
- [SCOPE-06] Replacing the underlying trace recorder or transport stack
- [SCOPE-07] Broad performance work unrelated to service and read-model ownership

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Turn orchestration responsibilities must be decomposed into chamber-aligned application seams so rendering/projection changes do not require editing a monolithic service. | SCOPE-01 | FR-06 | review |
| SRS-02 | The remaining top-level application service must compose chamber services rather than directly owning all recursive-control, projection, and observer behavior. | SCOPE-01 | FR-06 | review |
| SRS-03 | Conversation transcript/forensic/manifold/trace graph projections must move out of `domain/model` into an application-owned read-model boundary. | SCOPE-02 | FR-05 | review |
| SRS-04 | Runtime event presentation/projector logic must move out of `domain/model` while preserving equivalent surface-facing data for TUI and web adapters. | SCOPE-03, SCOPE-04 | FR-05 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Recorder, replay, and surface outputs must remain stable enough that the refactor can be validated with existing projection behavior and targeted contract tests. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | test |
| SRS-NFR-02 | Domain event types and invariants must remain presentation-free after the boundary move. | SCOPE-03, SCOPE-04 | NFR-04 | review |
| SRS-NFR-03 | The boundary cleanup must not add any new remote service dependency or hosted read-model store. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
