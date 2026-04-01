---
# system-managed
id: VFYcrMOia
status: backlog
created_at: 2026-04-01T09:28:48
updated_at: 2026-04-01T09:29:35
# authored
title: Retire Progress-Driven Transcript Repair Paths
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 5
---

# Retire Progress-Driven Transcript Repair Paths

## Summary

Finish the migration by removing the current replay-after-progress and cross-surface transcript repair heuristics once TUI and web both consume the canonical conversation plane. This is where the architecture becomes clean instead of merely functional.

## Acceptance Criteria

- [ ] Transcript hydration no longer depends on `synthesis_ready` or similar progress events [SRS-06/AC-01] <!-- verify: review, SRS-06:start:end -->
- [ ] Surface-specific transcript repair paths are removed or retired once the canonical conversation plane is authoritative [SRS-07/AC-02] <!-- verify: review, SRS-07:start:end -->
- [ ] Cross-surface transcript updates appear without manual page reload, TUI restart, or operator-triggered replay commands [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] The migration introduces no new external service or browser build dependency [SRS-NFR-04/AC-04] <!-- verify: review, SRS-NFR-04:start:end -->
