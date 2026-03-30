---
# system-managed
id: VFNyqDNsI
status: done
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T14:13:12
# authored
title: TUI Rendering For Search Progress Events
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 3
---

# TUI Rendering For Search Progress Events

## Summary

Add format_turn_event_row handling for GathererSearchProgress. Render as an updating event row showing phase and elapsed time. Progress rows replace each other in the live tail rather than accumulating.

## Acceptance Criteria

- [ ] format_turn_event_row renders GathererSearchProgress with phase and elapsed time [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Progress rows update in-place in the live tail rather than accumulating [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] When search completes, progress row is replaced by GathererSummary [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
- [ ] Elapsed time renders using format_duration_compact [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end -->
