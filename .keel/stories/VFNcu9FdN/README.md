---
# system-managed
id: VFNcu9FdN
status: done
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:40:01
# authored
title: Step Timing Reservoir Data Structure
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 1
started_at: 2026-03-30T12:29:31
submitted_at: 2026-03-30T12:39:56
completed_at: 2026-03-30T12:40:01
---

# Step Timing Reservoir Data Structure

## Summary

Introduce a `StepTimingReservoir` type that stores the last N deltas per event-type key. The reservoir is the core data structure — no persistence or UI wiring yet.

Key decisions:
- Event-type keys derived from TurnEvent serde tag names (e.g. `"intent_classified"`, `"tool_called"`)
- Fixed window of 50 entries per key (VecDeque)
- Methods: `record(key, delta)`, `percentile(key, p) -> Option<Duration>`
- Percentile uses nearest-rank on sorted window; returns None when fewer than 5 samples

## Acceptance Criteria

- [x] StepTimingReservoir::record stores deltas per key up to window cap [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Oldest entries evicted when window is full [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] percentile returns None when fewer than 5 samples exist [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] percentile returns correct p50 and p85 for a known dataset [SRS-03/AC-04] <!-- verify: manual, SRS-03:start:end, proof: ac-4.log-->
