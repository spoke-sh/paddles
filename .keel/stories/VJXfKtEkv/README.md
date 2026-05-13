---
# system-managed
id: VJXfKtEkv
status: backlog
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:29:36
# authored
title: Author Shared Planner Action Schema Renderer
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hYYX
index: 1
---

# Author Shared Planner Action Schema Renderer

## Summary

Create the shared authored planner action schema contract and renderer. The
renderer provides canonical prompt-facing action schema blocks for initial
planner decisions, recursive next-action decisions, retry prompts, and
redecision prompts.

## Acceptance Criteria

- [ ] A shared planner action schema contract defines action names, JSON examples, required fields, and shared action-selection rules. [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] The renderer can produce canonical schema blocks for initial and recursive prompt variants. [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
- [ ] Semantic workspace actions and `external_capability` are represented by the renderer contract. [SRS-01/AC-03] <!-- verify: test, SRS-01:start:end -->
- [ ] The renderer output leaves room for the existing `PlannerExecutionContract` capability manifest to be rendered separately. [SRS-02/AC-04] <!-- verify: review, SRS-02:start:end -->
