---
# system-managed
id: VFHIX1MLt
status: done
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T11:58:34
# authored
title: Route Steering Prompts Through Model-Driven Thread Selection
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 3
started_at: 2026-03-29T11:56:57
submitted_at: 2026-03-29T11:58:33
completed_at: 2026-03-29T11:58:34
---

# Route Steering Prompts Through Model-Driven Thread Selection

## Summary

Capture steering prompts during active turns as structured candidates and route
them through the model-driven thread decision loop at safe checkpoints, with
bounded controller validation and honest fail-closed behavior.

## Acceptance Criteria

- [x] Steering prompts received during an active turn are retained as structured thread candidates instead of opaque queue entries. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Active-turn steering prompts are routed through the model-driven thread decision loop at safe checkpoints instead of being silently appended to opaque queue state. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] Invalid model output or recorder failures degrade through bounded local-first fallback behavior instead of silently mutating thread structure. [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
