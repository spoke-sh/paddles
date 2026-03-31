---
# system-managed
id: VFP2FDmFm
status: done
created_at: 2026-03-30T18:07:22
updated_at: 2026-03-30T19:48:07
# authored
title: Define CompactionRequest And CompactionPlan Domain Types
type: feat
operator-signal:
scope: VFOmN3n4E/VFOvI9PzB
index: 1
started_at: 2026-03-30T19:10:00
completed_at: 2026-03-30T19:48:07
---

# Define CompactionRequest And CompactionPlan Domain Types

## Summary

Define the core domain types for the self-assessing compaction system.

## Acceptance Criteria

- [x] CompactionRequest with target_scope, relevance_query, and budget [SRS-01/AC-01] <!-- verify: cargo test -- domain::model::compaction::tests, SRS-01:start:end, proof: tests_passed.log -->
- [x] CompactionPlan with decisions (Keep, Compact, Discard) [SRS-02/AC-01] <!-- verify: cargo test -- domain::model::compaction::tests, SRS-02:start:end, proof: tests_passed.log -->
