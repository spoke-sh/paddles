---
# system-managed
id: VFOkHIDzL
status: backlog
created_at: 2026-03-30T16:55:57
updated_at: 2026-03-30T17:06:27
# authored
title: TUI In-Place Rendering For Planner Step Progress
type: feat
operator-signal:
scope: VFOiwHCXn/VFOjDg7Zm
index: 3
---

# TUI In-Place Rendering For Planner Step Progress

## Summary

Render PlannerStepProgress events in-place in the TUI, replacing the previous progress row on each new step. Coexist with GathererSearchProgress using independent row tracking. At verbose=0 show "Step N/M: action — query", at verbose=1+ add evidence count.

## Acceptance Criteria

- [ ] TUI renders PlannerStepProgress in-place, replacing previous progress row like GathererSearchProgress [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Planner step progress and search progress coexist via independent row tracking [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] format_turn_event_row renders PlannerStepProgress as "Step N/M: action — query" at verbose=0 [SRS-07/AC-03] <!-- verify: manual, SRS-07:start:end -->
- [ ] At verbose=0, at most one in-place progress line during the entire planner loop [SRS-NFR-02/AC-04] <!-- verify: manual, SRS-NFR-02:start:end -->
