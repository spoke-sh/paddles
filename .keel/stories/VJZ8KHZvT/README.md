---
# system-managed
id: VJZ8KHZvT
status: done
created_at: 2026-05-13T21:29:43
updated_at: 2026-05-13T22:03:53
# authored
title: Resolve Final Rendering Through HTTP Model Clients
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 2
started_at: 2026-05-13T22:02:02
completed_at: 2026-05-13T22:03:53
---

# Resolve Final Rendering Through HTTP Model Clients

## Summary

Move final-rendering model construction to the HTTP model-client boundary. The
turn loop should receive a final-rendering client without paddles preparing a
local inference model.

## Acceptance Criteria

- [x] A failing test is added first proving final-rendering runtime construction never receives local `ModelPaths`. [SRS-02/AC-01] <!-- verify: cargo nextest run final_rendering_http_client_rejects_local_model_paths prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths, SRS-02:start:end, proof: ac-1.log-->
- [x] Final-rendering clients are built through HTTP provider configuration and capability negotiation. [SRS-02/AC-02] <!-- verify: cargo nextest run final_rendering_client_builds_from_http_provider_configuration final_rendering_client_rejects_legacy_sift_provider_with_migration_hint, SRS-02:start:end, proof: ac-2.log-->
- [x] HTTP provider tests for structured final answers, retries, and provider-specific schema behavior remain green. [SRS-02/AC-03] <!-- verify: cargo nextest run openai_provider_normalizes_structured_final_answers send_with_retry_retries_on_429_then_succeeds gemini_provider_executes_a_full_turn_against_a_mock_server anthropic_provider_executes_a_full_turn_against_a_mock_server moonshot_provider_executes_full_turn_without_planner_tool_choice, SRS-02:start:end, proof: ac-3.log-->
