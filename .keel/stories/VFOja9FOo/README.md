---
# system-managed
id: VFOja9FOo
status: backlog
created_at: 2026-03-30T16:53:11
updated_at: 2026-03-30T17:06:27
# authored
title: Add PlannerStepProgress TurnEvent Variant
type: feat
operator-signal:
scope: VFOiwHCXn/VFOjDg7Zm
index: 1
---

# Add PlannerStepProgress TurnEvent Variant

## Summary

Add TurnEvent::PlannerStepProgress with step_number, step_limit, action, query, and evidence_count fields. This is the verbose=0 event that lets users see which planner step is executing and what it's targeting, without waiting for the full step to complete.

## Acceptance Criteria

- [ ] TurnEvent::PlannerStepProgress variant exists with step_number, step_limit, action, query, and evidence_count fields [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] min_verbosity is 0 (always visible) [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] event_type_key returns "planner_step_progress" [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] render_turn_event in application/mod.rs handles the new variant [SRS-01/AC-04] <!-- verify: manual, SRS-01:start:end -->
