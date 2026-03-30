---
# system-managed
id: VFNyqCTqy
status: icebox
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T13:47:31
# authored
title: Search Progress TurnEvent And Elapsed Timer
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 2
---

# Search Progress TurnEvent And Elapsed Timer

## Summary

Add TurnEvent::GathererSearchProgress with phase (indexing/searching/planning), elapsed_seconds, and optional detail string. Emit from the gather_context async wrapper as heartbeats arrive from the progress channel. The application layer forwards these to the TUI event sink.

## Acceptance Criteria

- [ ] TurnEvent::GathererSearchProgress variant exists with phase, elapsed_seconds, and detail fields [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Events emitted from gather_context as channel heartbeats arrive [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] event_type_key returns "gatherer_search_progress" for the new variant [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
- [ ] min_verbosity is 0 (always visible — this is the point of the feature) [SRS-02/AC-04] <!-- verify: manual, SRS-02:start:end -->
