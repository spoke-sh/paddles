---
# system-managed
id: VFV0xoWcl
status: done
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T19:06:19
# authored
title: Rename And Rewire Gatherer Configuration Away From Autonomous Planning
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 3
started_at: 2026-03-31T18:59:19
completed_at: 2026-03-31T19:06:19
---

# Rename And Rewire Gatherer Configuration Away From Autonomous Planning

## Summary

Align configuration, provider naming, and runtime wiring with the new architecture so paddles operators see sift as a retrieval backend rather than a second planner.

## Acceptance Criteria

- [x] Gatherer configuration and provider selection no longer imply that paddles delegates recursive planning to `sift-autonomous`. [SRS-05/AC-01] <!-- verify: cargo test -q runtime_lane_config_defaults_to_synthesizer_responses && cargo test -q sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-05:start:end, proof: ac-1.log-->
- [x] Runtime labels and summaries describe sift as a retrieval backend in logs, traces, or UI copy where applicable. [SRS-05/AC-02] <!-- verify: cargo test -q sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-05:start:end, proof: ac-2.log-->
- [x] Any required compatibility aliasing or migration behavior is explicit rather than silently preserving misleading autonomous terminology. [SRS-05/AC-03] <!-- verify: cargo test -q normalizes_legacy_gatherer_provider_alias, SRS-05:start:end, proof: ac-3.log-->
