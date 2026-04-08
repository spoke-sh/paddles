---
# system-managed
id: VGDNlXw4N
status: in-progress
created_at: 2026-04-08T08:46:25
updated_at: 2026-04-08T08:50:30
# authored
title: Define Deterministic Entity Resolver Contracts
type: feat
operator-signal:
scope: VGDNcabks/VGDNgMbMW
index: 1
started_at: 2026-04-08T08:50:30
---

# Define Deterministic Entity Resolver Contracts

## Summary

Define the domain and planner-facing contract for deterministic entity/path resolution so later implementation stories can resolve authored workspace targets without inventing ad hoc result shapes.

## Acceptance Criteria

- [ ] A typed resolver request/result contract exists for deterministic entity/path lookup, including explicit resolved, ambiguous, and missing outcomes. [SRS-01/AC-01] <!-- verify: cargo nextest run entity_resolver_contracts --no-tests pass, SRS-01:start:end -->
- [ ] Planner/controller integration seams can carry resolver outcomes without collapsing them into free-form strings. [SRS-01/AC-02] <!-- verify: cargo nextest run planner_can_carry_resolver_outcomes --no-tests pass, SRS-01:start:end -->
