---
# system-managed
id: VFNcuCigL
status: backlog
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:24:40
# authored
title: Wire Reservoir Into TUI Event Loop
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 5
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

- [ ] Reservoir is loaded from cache at TUI startup [SRS-13/AC-01] <!-- verify: manual, SRS-13:start:end -->
- [ ] Each TurnEvent delta is recorded into the reservoir [SRS-12/AC-02] <!-- verify: test, SRS-12:start:end -->
- [ ] Pace classification is used when rendering timing labels [SRS-10/AC-03] <!-- verify: manual, SRS-10:start:end -->
- [ ] Reservoir is flushed to disk after turn completion [SRS-14/AC-04] <!-- verify: manual, SRS-14:start:end -->
- [ ] Event type key matches the serde tag for each TurnEvent variant [SRS-12/AC-05] <!-- verify: test, SRS-12:start:end -->
