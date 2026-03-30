---
# system-managed
id: VFJ5wUdgP
status: done
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T18:27:54
# authored
title: Replace Heuristic Planner Fallback With Constrained Model Re-Decision
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 5
started_at: 2026-03-29T18:24:45
completed_at: 2026-03-29T18:27:54
---

# Replace Heuristic Planner Fallback With Constrained Model Re-Decision

## Summary

Replace heuristic initial/planner fallback selection with additional constrained
model re-decision passes so invalid action replies do not immediately trigger
controller reasoning substitutes.

## Acceptance Criteria

- [x] Invalid initial-action replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-01] <!-- verify: cargo test -q invalid_initial_action_replies_use_constrained_redecision_before_succeeding, SRS-03:start:end, proof: ac-1.log-->
- [x] Invalid recursive planner replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-02] <!-- verify: cargo test -q invalid_planner_replies_use_constrained_redecision_before_succeeding, SRS-03:start:end, proof: ac-2.log-->
- [x] Any residual controller fallback is minimal, explicit, and fail-closed rather than a ranked reasoning engine. [SRS-NFR-01/AC-03] <!-- verify: cargo test -q invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid && cargo test -q invalid_planner_replies_fail_closed_after_redecision_is_still_invalid, SRS-NFR-01:start:end, proof: ac-3.log-->
