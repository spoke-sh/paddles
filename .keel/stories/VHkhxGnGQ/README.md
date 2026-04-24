---
# system-managed
id: VHkhxGnGQ
status: icebox
created_at: 2026-04-24T16:02:28
updated_at: 2026-04-24T16:06:42
# authored
title: Surface Runtime Posture In Operator Projections
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 1
---

# Surface Runtime Posture In Operator Projections

## Summary

Surface capability posture, governance decisions, diagnostics, worker evidence, and eval outcomes through existing operator projections.

## Acceptance Criteria

- [ ] CLI, TUI, or web projections expose new runtime posture events without inventing controller-authored plans. [SRS-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-01:start:end -->
- [ ] Projection snapshots include governance, diagnostics, provenance, worker, and eval outcome fields where present. [SRS-NFR-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-NFR-01:start:end -->
