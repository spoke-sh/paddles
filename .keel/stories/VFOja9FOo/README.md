---
# system-managed
id: VFOja9FOo
status: done
created_at: 2026-03-30T16:53:11
updated_at: 2026-03-30T17:14:34
# authored
title: Add PlannerStepProgress TurnEvent Variant
type: feat
operator-signal:
scope: VFOiwHCXn/VFOjDg7Zm
index: 1
started_at: 2026-03-30T17:11:03
submitted_at: 2026-03-30T17:14:29
completed_at: 2026-03-30T17:14:34
---

# Add PlannerStepProgress TurnEvent Variant

## Summary

Add TurnEvent::PlannerStepProgress with step_number, step_limit, action, query, and evidence_count fields. This is the verbose=0 event that lets users see which planner step is executing and what it's targeting, without waiting for the full step to complete.

## Acceptance Criteria

- [x] TurnEvent::PlannerStepProgress variant exists with step_number, step_limit, action, query, and evidence_count fields [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] min_verbosity is 0 (always visible) [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] event_type_key returns "planner_step_progress" [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
- [x] render_turn_event in application/mod.rs handles the new variant [SRS-01/AC-04] <!-- verify: manual, SRS-01:start:end, proof: ac-4.log-->
