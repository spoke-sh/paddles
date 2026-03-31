---
# system-managed
id: VFP2FEoH6
status: backlog
created_at: 2026-03-30T18:07:22
updated_at: 2026-03-30T18:52:53
# authored
title: Implement Bounded Self-Assessment Engine
type: feat
operator-signal:
scope: VFOmN3n4E/VFOvI9PzB
index: 2
---

# Implement Bounded Self-Assessment Engine

## Summary

Implement the logic that uses the planner to assess context relevance.

## Acceptance Criteria

- [ ] assess_context_relevance implementation [SRS-03/AC-01] <!-- verify: test, SRS-03:start:end -->
- [ ] Respects CompactionBudget.max_steps [SRS-04/AC-01] <!-- verify: test, SRS-04:start:end -->
- [ ] Budget strictly bounded [SRS-NFR-01/AC-01] <!-- verify: test, SRS-NFR-01:start:end -->
