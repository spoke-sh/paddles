---
# system-managed
id: VFDvEQ7iu
status: done
created_at: 2026-03-28T20:30:28
updated_at: 2026-03-28T21:20:41
# authored
title: Make AGENTS-Driven Interpretation First-Class
type: feat
operator-signal:
scope: VFDv1i61H/VFDv3gE5m
index: 1
started_at: 2026-03-28T21:14:12
submitted_at: 2026-03-28T21:20:40
completed_at: 2026-03-28T21:20:41
---

# Make AGENTS-Driven Interpretation First-Class

## Summary

Make operator memory first-class in turn interpretation so the planner sees
`AGENTS.md` and linked foundational guidance before it chooses the next action.

## Acceptance Criteria

- [x] Interpretation-time context assembly includes operator memory and relevant foundational guidance instead of injecting that memory only into late answer prompts. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Planner-visible context can reference linked foundational docs without turning them into hardcoded domain-specific intents. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
