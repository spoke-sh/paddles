---
# system-managed
id: VFH4Fi0Dk
status: icebox
created_at: 2026-03-29T09:25:06
updated_at: 2026-03-29T09:25:06
# authored
title: Implement Embedded Transit Recorder Adapter And Replay Proof
type: feat
operator-signal:
scope: VFH4BXH4F/VFH4CCJ4d
index: 5
---

# Implement Embedded Transit Recorder Adapter And Replay Proof

## Summary

Implement the first durable recorder adapter through embedded `transit-core`
and prove that representative `paddles` traces can be recorded and replayed
locally without requiring a networked `transit` server.

## Acceptance Criteria

- [ ] An embedded `transit-core` recorder adapter maps the `paddles` trace contract into local roots, appends, branches, and checkpoints. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Replay proof artifacts demonstrate that representative `paddles` traces can be recorded and read back locally. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] Foundational docs explain the recorder boundary, transit alignment, embedded/server distinction, and current limitations honestly. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
- [ ] The implementation does not require a networked `transit` server for normal local recording. [SRS-NFR-01/AC-04] <!-- verify: manual, SRS-NFR-01:start:end -->
