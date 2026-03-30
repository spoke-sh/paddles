---
# system-managed
id: VFNyqDNsI
status: icebox
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T13:47:31
# authored
title: TUI Rendering For Search Progress Events
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 3
---

# TUI Rendering For Search Progress Events

## Summary

Add format_turn_event_row handling for GathererSearchProgress. Render as an updating event row showing phase and elapsed time (e.g. "Indexing workspace 12s", "Searching graph 28s"). These rows replace each other in the live tail rather than accumulating — only the latest progress event for the current search is visible.

## Acceptance Criteria

- [ ] format_turn_event_row renders GathererSearchProgress with phase and elapsed time [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Progress rows update in-place in the live tail rather than accumulating [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] When search completes, the progress row is replaced by the GathererSummary row [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] The elapsed time renders in compact format using format_duration_compact [SRS-03/AC-04] <!-- verify: manual, SRS-03:start:end -->
