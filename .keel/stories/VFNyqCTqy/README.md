---
# system-managed
id: VFNyqCTqy
status: done
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T16:41:30
# authored
title: Search Progress TurnEvent And Elapsed Timer
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 2
started_at: 2026-03-30T16:39:14
submitted_at: 2026-03-30T16:41:25
completed_at: 2026-03-30T16:41:30
---

# Search Progress TurnEvent And Elapsed Timer

## Summary

Add TurnEvent::GathererSearchProgress with phase, elapsed_seconds, and optional detail string. Emit from the gather_context async wrapper as heartbeats arrive from the progress channel. The application layer forwards these to the TUI event sink.

## Acceptance Criteria

- [x] TurnEvent::GathererSearchProgress variant exists with phase, elapsed_seconds, and detail fields [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Events emitted from gather_context as channel heartbeats arrive [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] event_type_key returns "gatherer_search_progress" [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] min_verbosity is 0 (always visible) [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end, proof: ac-4.log-->
