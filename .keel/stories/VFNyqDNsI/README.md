---
# system-managed
id: VFNyqDNsI
status: done
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T16:41:30
# authored
title: TUI Rendering For Search Progress Events
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 3
started_at: 2026-03-30T16:39:14
submitted_at: 2026-03-30T16:41:25
completed_at: 2026-03-30T16:41:30
---

# TUI Rendering For Search Progress Events

## Summary

Add format_turn_event_row handling for GathererSearchProgress. Render as an updating event row showing phase and elapsed time. Progress rows replace each other in the live tail rather than accumulating.

## Acceptance Criteria

- [x] format_turn_event_row renders GathererSearchProgress with phase and elapsed time [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Progress rows update in-place in the live tail rather than accumulating [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] When search completes, progress row is replaced by GathererSummary [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] Elapsed time renders using format_duration_compact [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end, proof: ac-4.log-->
