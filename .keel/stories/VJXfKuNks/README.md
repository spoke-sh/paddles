---
# system-managed
id: VJXfKuNks
status: done
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:59:36
# authored
title: Prove Planner Lane Schema Parity
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hlYW
index: 2
started_at: 2026-05-13T15:57:12
completed_at: 2026-05-13T15:59:36
---

# Prove Planner Lane Schema Parity

## Summary

Add mocked-turn tests that extract the canonical schema block from Sift and
HTTP planner prompts and compare the blocks exactly.

## Acceptance Criteria

- [x] Mocked Sift and HTTP initial planner turns receive the same canonical schema block. [SRS-04/AC-01] <!-- verify: cargo test mocked_initial_planner_lanes_receive_same_canonical_schema_block --lib, SRS-04:start:end, proof: ac-1.log-->
- [x] Mocked Sift and HTTP recursive planner turns receive the same canonical schema block. [SRS-05/AC-02] <!-- verify: cargo test mocked_recursive_planner_lanes_receive_same_canonical_schema_block --lib, SRS-05:start:end, proof: ac-2.log-->
- [x] Test failures identify the drifting lane and prompt variant. [SRS-NFR-02/AC-03] <!-- verify: cargo test schema_block_assertions_name_drifting_lane_and_prompt_variant --lib, SRS-NFR-02:start:end, proof: ac-3.log-->
