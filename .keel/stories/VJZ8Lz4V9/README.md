---
# system-managed
id: VJZ8Lz4V9
status: done
created_at: 2026-05-13T21:29:49
updated_at: 2026-05-13T22:17:24
# authored
title: Migrate Legacy Runtime Lane Preferences
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 2
started_at: 2026-05-13T22:14:42
completed_at: 2026-05-13T22:17:24
---

# Migrate Legacy Runtime Lane Preferences

## Summary

Keep legacy lane-shaped config readable as migration input while making the new
turn-runtime preference shape the only write target. Legacy Sift model-provider
values still hard-fail rather than remapping silently.

## Acceptance Criteria

- [x] Migration fixture tests prove legacy runtime-lane config is read and normalized into turn-runtime preferences. [SRS-02/AC-01] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_migrate_into_turn_runtime_shape, SRS-02:start:end, proof: ac-1.log-->
- [x] Persistence tests prove new writes use only the turn-runtime preference shape. [SRS-02/AC-02] <!-- verify: cargo nextest run turn_runtime_preference_store_writes_canonical_shape_without_lane_terms turn_runtime_preference_store_round_trips_preferences, SRS-02:start:end, proof: ac-2.log-->
- [x] Legacy lane config containing Sift model-provider values fails with the approved `ollama:<model>` migration hint. [SRS-NFR-01/AC-03] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_reject_sift_model_provider_with_migration_hint, SRS-NFR-01:start:end, proof: ac-3.log-->
