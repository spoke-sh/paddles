---
# system-managed
id: VJZ8LOzJi
status: done
created_at: 2026-05-13T21:29:47
updated_at: 2026-05-13T22:13:44
# authored
title: Introduce Turn Runtime Preference Schema
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 1
started_at: 2026-05-13T22:08:55
completed_at: 2026-05-13T22:13:44
---

# Introduce Turn Runtime Preference Schema

## Summary

Introduce the canonical turn-runtime preference schema. New code should describe
model clients and turn phases directly instead of persisting planner,
synthesizer, gatherer, or runtime-lane settings.

## Acceptance Criteria

- [x] Tests define the new preference shape using action-selection, final-rendering, retrieval, model-client, and turn-runtime terminology. [SRS-01/AC-01] <!-- verify: cargo nextest run turn_runtime_preferences_capture_model_clients_and_retrieval turn_runtime_preferences_record_shared_model_clients_without_lane_names, SRS-01:start:end, proof: ac-1.log-->
- [x] New preference writes do not emit planner, synthesizer, gatherer, or lane-shaped field names. [SRS-01/AC-02] <!-- verify: cargo nextest run turn_runtime_preference_store_writes_canonical_shape_without_lane_terms turn_runtime_preference_store_round_trips_preferences, SRS-01:start:end, proof: ac-2.log-->
- [x] Runtime construction consumes normalized turn-runtime preferences. [SRS-01/AC-03] <!-- verify: cargo nextest run load_layers_runtime_preferences_after_authored_config runtime_preferences_override_workspace_config_for_lane_fields openai_gpt_5_4_thinking_mode_selection_queues_runtime_update_from_prompt_box, SRS-01:start:end, proof: ac-3.log-->
