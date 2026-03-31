---
# system-managed
id: VFURnK5iu
status: backlog
created_at: 2026-03-31T16:20:10
updated_at: 2026-03-31T16:24:51
# authored
title: Add RefinementTrigger And RefinementPolicy Types
type: feat
operator-signal:
scope: VFUNJz9zT/VFURjeU0t
index: 1
---

# Add RefinementTrigger And RefinementPolicy Types

## Summary

Define refinement primitives (`RefinementTrigger` and `RefinementPolicy`) with stable ids, sources, and thresholds that the planner can evaluate during active turns.

## Acceptance Criteria

- [ ] Add domain types for trigger/policy-driven refinement including trigger source, thresholds, and policy metadata consumed by the planner loop. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
