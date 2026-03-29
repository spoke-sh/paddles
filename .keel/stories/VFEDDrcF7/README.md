---
# system-managed
id: VFEDDrcF7
status: backlog
created_at: 2026-03-28T21:41:55
updated_at: 2026-03-28T21:48:08
# authored
title: Define Model-Directed Next-Action Contract
type: feat
operator-signal:
scope: VFECyWLL6/VFED2RjSu
index: 1
---

# Define Model-Directed Next-Action Contract

## Summary

Define the constrained top-level action schema that replaces heuristic routing
for non-trivial turns. This story owns the contract shape, validation rules,
and the planner/synth boundary needed for model-directed first action
selection.

## Acceptance Criteria

- [ ] A top-level action contract exists for first action selection and can express direct answer or synthesize, search, read, inspect, refine, branch, and stop. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] The contract defines the validation envelope the controller must enforce, including safe inspect/tool boundaries and fail-closed behavior, without yet owning the runtime refactor. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] The contract is positioned as a general-purpose harness boundary rather than a Keel-specific routing feature. [SRS-NFR-03/AC-03] <!-- verify: manual, SRS-NFR-03:start:end -->
