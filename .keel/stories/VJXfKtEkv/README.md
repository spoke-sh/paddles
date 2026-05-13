---
# system-managed
id: VJXfKtEkv
status: done
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:38:49
# authored
title: Author Shared Planner Action Schema Renderer
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hYYX
index: 1
started_at: 2026-05-13T15:33:29
completed_at: 2026-05-13T15:38:49
---

# Author Shared Planner Action Schema Renderer

## Summary

Create the shared authored planner action schema contract and renderer. The
renderer provides canonical prompt-facing action schema blocks for initial
planner decisions, recursive next-action decisions, retry prompts, and
redecision prompts.

## Acceptance Criteria

- [x] A shared planner action schema contract defines action names, JSON examples, required fields, and shared action-selection rules. [SRS-01/AC-01] <!-- verify: cargo test planner_action_schema --lib, SRS-01:start:end, proof: ac-1.log-->
- [x] The renderer can produce canonical schema blocks for initial and recursive prompt variants. [SRS-02/AC-02] <!-- verify: cargo test planner_action_schema --lib, SRS-02:start:end, proof: ac-2.log-->
- [x] Semantic workspace actions and `external_capability` are represented by the renderer contract. [SRS-01/AC-03] <!-- verify: cargo test planner_action_schema --lib, SRS-01:start:end, proof: ac-3.log-->
- [x] The renderer output leaves room for the existing `PlannerExecutionContract` capability manifest to be rendered separately. [SRS-02/AC-04] <!-- verify: cargo test schema_renderer_leaves_execution_contract_separate --lib, SRS-02:start:end, proof: ac-4.log-->
