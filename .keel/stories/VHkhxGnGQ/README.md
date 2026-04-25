---
# system-managed
id: VHkhxGnGQ
status: done
created_at: 2026-04-24T16:02:28
updated_at: 2026-04-24T19:06:02
# authored
title: Surface Runtime Posture In Operator Projections
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 1
started_at: 2026-04-24T19:02:28
completed_at: 2026-04-24T19:06:02
---

# Surface Runtime Posture In Operator Projections

## Summary

Surface capability posture, governance decisions, diagnostics, worker evidence, and eval outcomes through existing operator projections.

## Acceptance Criteria

- [x] CLI, TUI, or web projections expose new runtime posture events without inventing controller-authored plans. [SRS-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Projection snapshots include governance, diagnostics, provenance, worker, and eval outcome fields where present. [SRS-NFR-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->
