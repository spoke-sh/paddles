---
# system-managed
id: VFP2FFxGM
status: done
created_at: 2026-03-30T18:07:22
updated_at: 2026-03-30T20:18:10
# authored
title: Implement Artifact Compaction With Locators
type: feat
operator-signal:
scope: VFOmN3n4E/VFOvI9PzB
index: 3
started_at: 2026-03-30T19:40:00
completed_at: 2026-03-30T20:18:10
---

# Implement Artifact Compaction With Locators

## Summary

Implement the actual compaction of artifacts, ensuring they carry locators.

## Acceptance Criteria

- [x] Compacted summaries wrapped in ArtifactEnvelope with ContextLocator [SRS-05/AC-01] <!-- verify: cargo test -- application::tests::compaction_engine_executes_plan_and_preserves_locators, SRS-05:start:end, proof: test_output.log -->
