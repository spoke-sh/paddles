---
# system-managed
id: VFDvEQfis
status: done
created_at: 2026-03-28T20:30:28
updated_at: 2026-03-28T21:20:42
# authored
title: Replace Static Turn Classification With Planner Action Selection
type: feat
operator-signal:
scope: VFDv1i61H/VFDv3gE5m
index: 2
started_at: 2026-03-28T21:14:15
submitted_at: 2026-03-28T21:20:40
completed_at: 2026-03-28T21:20:42
---

# Replace Static Turn Classification With Planner Action Selection

## Summary

Replace static turn-type routing as the main reasoning mechanism with a planner
action-selection contract that decides the next bounded resource use.

## Acceptance Criteria

- [x] The runtime exposes a planner action contract that can express at least search, read, inspect, refine, branch, and stop decisions. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Non-trivial turns use planner action selection instead of relying solely on coarse static intent buckets. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
