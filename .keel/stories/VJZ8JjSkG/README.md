---
# system-managed
id: VJZ8JjSkG
status: done
created_at: 2026-05-13T21:29:41
updated_at: 2026-05-13T22:01:01
# authored
title: Resolve Action Selection Through HTTP Model Clients
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 1
started_at: 2026-05-13T21:57:23
completed_at: 2026-05-13T22:01:01
---

# Resolve Action Selection Through HTTP Model Clients

## Summary

Move action-selection model construction to the HTTP model-client boundary.
This story removes Sift model-path preparation from the action-selection path
while preserving provider capability negotiation.

## Acceptance Criteria

- [x] A failing test is added first proving action-selection runtime construction never receives local `ModelPaths`. [SRS-01/AC-01] <!-- verify: cargo nextest run action_selection_http_client_rejects_local_model_paths prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths, SRS-01:start:end, proof: ac-1.log-->
- [x] Action-selection clients are built through HTTP provider configuration and capability negotiation. [SRS-01/AC-02] <!-- verify: cargo nextest run action_selection_client_builds_from_http_provider_configuration, SRS-01:start:end, proof: ac-2.log-->
- [x] Legacy Sift action-selection provider config fails with the approved `ollama:<model>` migration hint. [SRS-01/AC-03] <!-- verify: cargo nextest run action_selection_client_rejects_legacy_sift_provider_with_migration_hint prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction, SRS-01:start:end, proof: ac-3.log-->
