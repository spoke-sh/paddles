# Establish A Replayable Turn And Thread Control Substrate - SRS

## Summary

Epic: VGb1c1AAK
Goal: Establish one replayable turn/thread control substrate with same-turn steering, durable lifecycle transitions, and shared live runtime items for all surfaces.

## Scope

### In Scope

- [SCOPE-01] Define typed turn and thread control operations such as start, steer, interrupt, fork/resume, and rollback or archive equivalents.
- [SCOPE-02] Route same-turn steering and interruption through replayable control records instead of queued prompts alone.
- [SCOPE-03] Emit typed plan, diff, command, file-change, and control-state runtime items during active turns.
- [SCOPE-04] Expose one control surface vocabulary that TUI, web, and API layers can consume consistently.
- [SCOPE-05] Document the resulting control-plane behavior, degradation rules, and verification posture.

### Out of Scope

- [SCOPE-06] A hosted multi-tenant app-server or remote orchestration service.
- [SCOPE-07] Cross-machine collaboration or synchronization beyond one local Paddles runtime.
- [SCOPE-08] Full multi-agent delegation semantics; that belongs to the multi-agent mission.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define typed turn and thread control operations as first-class runtime contracts instead of relying on surface-specific prompt conventions. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Same-turn steering and interruption must become replayable control events with bounded fallback when the requested control action cannot apply. | SCOPE-02 | FR-02 | manual |
| SRS-03 | Fork, resume, and rollback or archive style transitions must preserve durable thread lineage and replayability. | SCOPE-01, SCOPE-02 | FR-03 | manual |
| SRS-04 | Active turns must emit typed runtime items for plan updates, diff updates, command summaries, file changes, and control-state transitions. | SCOPE-03 | FR-04 | manual |
| SRS-05 | TUI, web, and API surfaces must be able to render one shared control and runtime item vocabulary without inventing divergent semantics. | SCOPE-03, SCOPE-04 | FR-05 | manual |
| SRS-06 | Invalid or stale control requests must degrade honestly with explicit status instead of mutating hidden thread state. | SCOPE-02, SCOPE-04 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The control plane must build on the existing recorder, replay, and thread-lineage substrate instead of inventing a parallel state store. | SCOPE-01, SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Control transitions and live runtime items must remain readable enough for default transcript and UI projections. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-02 | manual |
| SRS-NFR-03 | Control semantics must stay deterministic enough for focused tests and replay proofs. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | manual |
| SRS-NFR-04 | The control plane must preserve the local-first recursive execution model. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
