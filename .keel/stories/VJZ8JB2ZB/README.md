---
# system-managed
id: VJZ8JB2ZB
status: done
created_at: 2026-05-13T21:29:39
updated_at: 2026-05-13T21:51:50
# authored
title: Codify Legacy Sift Provider Migration Failure
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8Bws9Z
index: 2
started_at: 2026-05-13T21:45:17
completed_at: 2026-05-13T21:51:50
---

# Codify Legacy Sift Provider Migration Failure

## Summary

Codify the compatibility behavior for old Sift model-provider settings. Legacy
Sift inference config must fail explicitly with an actionable migration hint
instead of silently changing providers.

## Acceptance Criteria

- [x] Tests prove `provider = "sift"` and equivalent planner/final-rendering legacy provider selections fail before runtime construction. [SRS-02/AC-01] <!-- verify: cargo nextest run provider_config_rejects_legacy_sift_model_provider planner_provider_config_rejects_legacy_sift_model_provider prepare_runtime_lanes_rejects_legacy_sift, SRS-02:start:end, proof: ac-1.log-->
- [x] The failure message states that `sift` no longer performs model inference and tells the operator to choose an HTTP provider such as `ollama:<model>`. [SRS-02/AC-02] <!-- verify: cargo nextest run provider_config_rejects_legacy_sift_model_provider planner_provider_config_rejects_legacy_sift_model_provider prepare_runtime_lanes_rejects_legacy_sift, SRS-02:start:end, proof: ac-2.log-->
- [x] Sift retrieval/indexing selections are not rejected by this model-provider compatibility policy. [SRS-02/AC-03] <!-- verify: cargo nextest run sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-02:start:end, proof: ac-3.log-->
