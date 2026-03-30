---
# system-managed
id: VFNcuCigL
status: done
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:40:01
# authored
title: Wire Reservoir Into TUI Event Loop
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 5
started_at: 2026-03-30T12:37:18
submitted_at: 2026-03-30T12:39:56
completed_at: 2026-03-30T12:40:01
---

# Wire Reservoir Into TUI Event Loop

## Summary

Connect all the pieces: load the reservoir at boot, record deltas as TurnEvents arrive, pass pace classification into transcript rendering, and flush the reservoir after each turn completes.

Integration points:
- `main.rs` or TUI init: load reservoir from cache path
- `handle_message(TurnEvent)`: extract event type key + delta, call `reservoir.record()`
- `render_row_lines`: look up pace for the row's timing delta and pass to styling
- `handle_message(TurnFinished)`: flush reservoir to disk

The event type key comes from the TurnEvent variant name. Add a `event_type_key()` method on TurnEvent that returns the serde tag string.

## Acceptance Criteria

- [x] Reservoir is loaded from cache at TUI startup [SRS-13/AC-01] <!-- verify: manual, SRS-13:start:end, proof: ac-1.log-->
- [x] Each TurnEvent delta is recorded into the reservoir [SRS-12/AC-02] <!-- verify: manual, SRS-12:start:end, proof: ac-2.log-->
- [x] Pace classification is used when rendering timing labels [SRS-10/AC-03] <!-- verify: manual, SRS-10:start:end, proof: ac-3.log-->
- [x] Reservoir is flushed to disk after turn completion [SRS-14/AC-04] <!-- verify: manual, SRS-14:start:end, proof: ac-4.log-->
- [x] Event type key matches the serde tag for each TurnEvent variant [SRS-12/AC-05] <!-- verify: manual, SRS-12:start:end, proof: ac-5.log-->
