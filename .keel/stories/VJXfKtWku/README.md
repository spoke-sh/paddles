---
# system-managed
id: VJXfKtWku
status: backlog
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:29:36
# authored
title: Add Planner Schema Enum Parity Tests
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hYYX
index: 2
---

# Add Planner Schema Enum Parity Tests

## Summary

Add tests that compare the authored planner action schema against the Rust
action enums and fail clearly when an action is missing or extra.

## Acceptance Criteria

- [ ] Tests prove schema coverage for `InitialAction` and `PlannerAction` terminal/control actions. [SRS-05/AC-01] <!-- verify: test, SRS-05:start:end -->
- [ ] Tests prove schema coverage for `WorkspaceAction`, including semantic variants and `ExternalCapability`. [SRS-03/AC-02] <!-- verify: test, SRS-03:start:end -->
- [ ] Tests or review proof confirm turn-specific availability remains in `PlannerExecutionContract`, not in the schema renderer. [SRS-04/AC-03] <!-- verify: test, SRS-04:start:end -->
- [ ] Test failures identify missing and extra schema actions clearly. [SRS-NFR-02/AC-03] <!-- verify: test, SRS-NFR-02:start:end -->
