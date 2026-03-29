---
# system-managed
id: VFEDDsfF9
status: backlog
created_at: 2026-03-28T21:41:55
updated_at: 2026-03-28T21:48:08
# authored
title: Replace Heuristic Top-Level Routing With Planner Decisions
type: feat
operator-signal:
scope: VFECyWLL6/VFED2RjSu
index: 3
---

# Replace Heuristic Top-Level Routing With Planner Decisions

## Summary

Retire the current heuristic top-level routing gate for non-trivial turns and
drive the initial route from the validated model-selected action path instead.
This story owns the runtime refactor across controller, planner loop, and safe
execution boundaries.

## Acceptance Criteria

- [ ] Non-trivial turns no longer depend on a separate heuristic classifier to choose their first resource action. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Safe inspect/tool execution remains controller-validated and bounded after the refactor. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [ ] Recursive planner execution and synthesizer handoff operate through the new top-level action contract without regressing grounded answers. [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end -->
- [ ] Model-directed routing remains local-first and fails closed when planner output is invalid or a heavier planner provider is unavailable. [SRS-NFR-01/AC-04] <!-- verify: manual, SRS-NFR-01:start:end -->
