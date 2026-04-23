# Hosted Cursor Resume And Projection Rebuild Semantics - SRS

## Summary

Epic: VHaTau3dH
Goal: Resume replay-derived session views and projections from hosted cursors and materialization checkpoints without depending on local recorder state or full replay on every restart.

## Scope

### In Scope

- [SCOPE-01] Hosted consumer cursor semantics for session and lifecycle consumers
- [SCOPE-02] Hosted materialization checkpoint/resume semantics for projection rebuilds
- [SCOPE-03] Restart/resume behavior for long-lived hosted service mode
- [SCOPE-04] Deterministic replay and no-loss/no-duplication verification for hosted resume

### Out of Scope

- [SCOPE-05] Defining the public projection payload schema itself
- [SCOPE-06] Local-only embedded recorder recovery behavior beyond explicit fallback modes
- [SCOPE-07] Downstream deployment or consumer behavior outside the Paddles runtime boundary

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Session and lifecycle consumers must persist and resume hosted consumer cursors so that service restart continues from the authoritative hosted position instead of replaying from zero by default. | SCOPE-01 | FR-06 | automated |
| SRS-02 | Projection rebuilders/materializers must persist and resume from hosted materialization checkpoints or equivalent hosted resume primitives. | SCOPE-02 | FR-06 | automated |
| SRS-03 | Restart resume must preserve turn ordering and avoid lost or duplicated work across command consumption, lifecycle publication, and projection updates. | SCOPE-03 | FR-07 | automated |
| SRS-04 | Full replay from authoritative Transit history must remain available as the correctness baseline even when resume uses cursors and checkpoints. | SCOPE-04 | NFR-01 | automated |
| SRS-05 | The runtime must expose enough replay revision metadata in hosted projections and diagnostics to explain which cursor/checkpoint state a resumed service is operating from. | SCOPE-03 | FR-05 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted resume optimizations must not change the canonical replay-derived truth model or require local-only checkpoint state. | SCOPE-01 | NFR-01 | automated |
| SRS-NFR-02 | Resume correctness must be reproducible through deterministic restart scenarios that cover no-loss/no-duplication guarantees. | SCOPE-04 | NFR-05 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
