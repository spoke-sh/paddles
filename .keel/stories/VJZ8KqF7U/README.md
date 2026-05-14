---
# system-managed
id: VJZ8KqF7U
status: done
created_at: 2026-05-13T21:29:45
updated_at: 2026-05-13T22:07:47
# authored
title: Preserve Sift Retrieval Outside Inference Cleanup
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 3
started_at: 2026-05-13T22:05:33
submitted_at: 2026-05-13T22:07:44
completed_at: 2026-05-13T22:07:47
---

# Preserve Sift Retrieval Outside Inference Cleanup

## Summary

Protect Sift retrieval/indexing from the inference cleanup. The story should
prove retrieval remains separately selectable and does not depend on removed
model-provider behavior.

## Acceptance Criteria

- [x] Tests prove legacy Sift model-provider branches fail before runtime construction using the ADR compatibility policy. [SRS-03/AC-01] <!-- verify: cargo nextest run prepare_runtime_lanes_rejects_legacy_sift_synthesizer_before_construction prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction, SRS-03:start:end, proof: ac-1.log-->
- [x] Tests prove Sift retrieval/indexing can be prepared without Sift model-provider inference paths. [SRS-04/AC-02] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-04:start:end, proof: ac-2.log-->
- [x] Retrieval provider selection remains independent from action-selection and final-rendering model-client selection. [SRS-04/AC-03] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference prepare_runtime_lanes_resolves_local_gatherer_paths_independent_of_http_inference, SRS-04:start:end, proof: ac-3.log-->
- [x] Any inference cleanup that would require deleting retrieval/indexing is stopped and split into a later mission decision. [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end, proof: ac-4.log-->
