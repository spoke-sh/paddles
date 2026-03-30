---
# system-managed
id: VFJ5wUdgP
status: backlog
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T17:47:05
# authored
title: Replace Heuristic Planner Fallback With Constrained Model Re-Decision
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 5
---

# Replace Heuristic Planner Fallback With Constrained Model Re-Decision

## Summary

Replace heuristic initial/planner fallback selection with additional constrained
model re-decision passes so invalid action replies do not immediately trigger
controller reasoning substitutes.

## Acceptance Criteria

- [ ] Invalid initial-action replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-01] <!-- verify: automated, SRS-03:start:end -->
- [ ] Invalid recursive planner replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-02] <!-- verify: automated, SRS-03:start:end -->
- [ ] Any residual controller fallback is minimal, explicit, and fail-closed rather than a ranked reasoning engine. [SRS-NFR-01/AC-03] <!-- verify: automated, SRS-NFR-01:start:end -->
