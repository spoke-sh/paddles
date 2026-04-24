---
# system-managed
id: VHkhvmjg7
status: icebox
created_at: 2026-04-24T16:02:22
updated_at: 2026-04-24T16:06:36
# authored
title: Record Snapshots And Rollback Anchors
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgNakSc
index: 2
---

# Record Snapshots And Rollback Anchors

## Summary

Record snapshot and rollback anchors around workspace-affecting actions so sessions can be replayed or recovered.

## Acceptance Criteria

- [ ] Workspace-affecting actions can record snapshot metadata and rollback anchors. [SRS-02/AC-01] <!-- verify: cargo test session_snapshots -- --nocapture, SRS-02:start:end -->
- [ ] Missing or incomplete snapshots are represented explicitly during replay. [SRS-NFR-02/AC-01] <!-- verify: cargo test session_snapshot_replay_validation -- --nocapture, SRS-NFR-02:start:end -->
