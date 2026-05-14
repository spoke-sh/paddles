---
# system-managed
id: VJZ8N7avc
status: done
created_at: 2026-05-13T21:29:54
updated_at: 2026-05-13T22:31:29
# authored
title: Delete Sift Agent And Planner Inference Adapters
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 1
started_at: 2026-05-13T22:27:19
completed_at: 2026-05-13T22:31:29
---

# Delete Sift Agent And Planner Inference Adapters

## Summary

Delete the Sift action-selection and final-rendering inference adapters after
HTTP-only runtime construction is proven. Any remaining Sift code must be
retrieval-specific or compatibility parsing that fails before runtime.

## Acceptance Criteria

- [x] Compile failures or targeted tests first identify every remaining active reference to Sift inference adapters. [SRS-01/AC-01] <!-- verify: sh -lc 'test ! -e src/infrastructure/adapters/sift_agent.rs && test ! -e src/infrastructure/adapters/sift_planner.rs && ! rg -n "sift_agent|SiftAgentAdapter|SiftPlannerAdapter|pub mod sift_planner|pub mod sift_agent" src tests', SRS-01:start:end -->
- [x] Sift action-selection and final-rendering inference adapters are deleted or made unreachable from runtime construction. [SRS-01/AC-02] <!-- verify: cargo clippy --all-targets -- -D warnings, SRS-01:start:end -->
- [x] Legacy Sift model-provider inputs still fail with the approved migration hint rather than panicking or falling through. [SRS-NFR-02/AC-03] <!-- verify: cargo nextest run action_selection_client_rejects_legacy_sift_provider_with_migration_hint final_rendering_client_rejects_legacy_sift_provider_with_migration_hint prepare_runtime_lanes_rejects_legacy_sift_synthesizer_before_construction prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction sift_direct_boundary_can_be_prepared_without_local_model_paths direct_gatherer_returns_direct_retrieval_metadata_and_evidence, SRS-NFR-02:start:end -->
