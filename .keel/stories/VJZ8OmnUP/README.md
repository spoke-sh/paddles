---
# system-managed
id: VJZ8OmnUP
status: done
created_at: 2026-05-13T21:30:00
updated_at: 2026-05-13T22:57:35
# authored
title: Rename Internal Lane Types To Turn Runtime Concepts
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 1
started_at: 2026-05-13T22:50:26
completed_at: 2026-05-13T22:57:35
---

# Rename Internal Lane Types To Turn Runtime Concepts

## Summary

Rename internal Rust runtime lane types to turn runtime concepts. This should
change active architecture names, not just user-visible labels.

## Acceptance Criteria

- [x] Internal types such as `RuntimeLaneConfig`, `PreparedRuntimeLanes`, `PreparedModelLane`, and `PreparedGathererLane` are replaced with turn-runtime/model-client/retrieval concepts or documented migration shims. [SRS-01/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "RuntimeLaneConfig|PreparedRuntimeLanes|PreparedModelLane|PreparedGathererLane|RuntimeLaneRole|prepare_runtime_lanes|prepared_runtime_lanes|default_response_lane|default_response_role|build_lane|from_runtime_lanes" src', SRS-01:start:end, proof: ac-1.log-->
- [x] Tests and module names use the new turn-runtime vocabulary where they describe active runtime architecture. [SRS-01/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "runtime_lane_config|remote_http_lane|prepared_turn_runtime_keep_synthesizer|prepare runtime lanes|Runtime lanes now target|Activating runtime lanes|requested runtime lanes|runtime_lanes|prepared_lanes|runtime_lane_summary" src/application/mod.rs src/application/harness_capability_posture.rs src/infrastructure/adapters/http_provider.rs src/infrastructure/cli/interactive_tui.rs src/main.rs src/infrastructure/web/mod.rs', SRS-01:start:end, proof: ac-2.log-->
- [x] Behavior stays covered by existing runtime construction and turn-loop tests after the rename. [SRS-NFR-01/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo nextest run prepare_turn_runtime --lib && cargo test --lib turn_runtime && cargo check --all-targets', SRS-NFR-01:start:end, proof: ac-3.log-->
