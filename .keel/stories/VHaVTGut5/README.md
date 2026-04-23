---
# system-managed
id: VHaVTGut5
status: backlog
created_at: 2026-04-22T22:10:10
updated_at: 2026-04-22T22:12:25
# authored
title: Add Hosted Materialization Checkpoints For Projection Rebuilds
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrQZr
index: 2
---

# Add Hosted Materialization Checkpoints For Projection Rebuilds

## Summary

Add hosted materialization checkpoint/resume support for replay-derived
projections so projection rebuilds can restart efficiently without drifting from
authoritative Transit history.

## Acceptance Criteria

- [ ] Projection reducers persist and resume hosted materialization checkpoints or equivalent hosted resume tokens. [SRS-02/AC-01] <!-- verify: cargo test hosted_projection_materialization_checkpoints_ -- --nocapture, SRS-02:start:end -->
- [ ] Hosted projection materializers can resume from persisted checkpoint state without requiring a local-only checkpoint store. [SRS-02/AC-02] <!-- verify: cargo test hosted_projection_materializers_resume_without_local_checkpoint_store -- --nocapture, SRS-02:start:end -->
