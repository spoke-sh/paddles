---
# system-managed
id: VGbPaBmCK
status: backlog
created_at: 2026-04-12T11:24:10
updated_at: 2026-04-12T11:25:34
# authored
title: Wire Same-Turn Steering Through Replayable Control Flow
type: feat
operator-signal:
scope: VGb1c1AAK/VGbPWnUh2
index: 2
---

# Wire Same-Turn Steering Through Replayable Control Flow

## Summary

Wire same-turn steering, interruption, and lineage-aware thread lifecycle
transitions through replayable control flow so the recursive harness can be
steered intentionally without falling back to opaque queued prompts or hidden
thread mutation.

## Acceptance Criteria

- [ ] Same-turn steering and interruption flow through replayable control records with bounded fallback when a requested action cannot apply. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Fork, resume, and rollback or archive style transitions preserve durable thread lineage and replayability. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Accepted steering and interruption requests stay attached to the active turn lifecycle instead of degrading back into queued follow-up prompts. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
- [ ] The steering path reports explicit bounded fallback when a requested control action is unsafe or unsupported in the current execution window. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
