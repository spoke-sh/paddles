---
# system-managed
id: VJXfKtWku
status: done
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:43:07
# authored
title: Add Planner Schema Enum Parity Tests
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hYYX
index: 2
started_at: 2026-05-13T15:41:29
completed_at: 2026-05-13T15:43:07
---

# Add Planner Schema Enum Parity Tests

## Summary

Add tests that compare the authored planner action schema against the Rust
action enums and fail clearly when an action is missing or extra.

## Acceptance Criteria

- [x] Tests prove schema coverage for `InitialAction` and `PlannerAction` terminal/control actions. [SRS-05/AC-01] <!-- verify: cargo test planner_action_schema --lib, SRS-05:start:end, proof: ac-1.log-->
- [x] Tests prove schema coverage for `WorkspaceAction`, including semantic variants and `ExternalCapability`. [SRS-03/AC-02] <!-- verify: cargo test planner_action_schema --lib, SRS-03:start:end, proof: ac-2.log-->
- [x] Tests or review proof confirm turn-specific availability remains in `PlannerExecutionContract`, not in the schema renderer. [SRS-04/AC-03] <!-- verify: cargo test planner_action_schema --lib, SRS-04:start:end, proof: ac-3.log-->
- [x] Test failures identify missing and extra schema actions clearly. [SRS-NFR-02/AC-03] <!-- verify: cargo test planner_action_schema --lib, SRS-NFR-02:start:end, proof: ac-4.log-->
