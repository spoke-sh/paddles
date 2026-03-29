---
# system-managed
id: VFCzXgoAk
status: done
created_at: 2026-03-28T16:41:19
updated_at: 2026-03-28T17:22:27
# authored
title: Expose Planner Telemetry And Compare Retrieval Modes
type: feat
operator-signal:
scope: VFCzL9KKd/VFCzWHL1Y
index: 4
started_at: 2026-03-28T17:16:26
submitted_at: 2026-03-28T17:22:25
completed_at: 2026-03-28T17:22:27
---

# Expose Planner Telemetry And Compare Retrieval Modes

## Summary

Expose planner telemetry to operators and add proof or evaluation coverage that
compares static context assembly against autonomous retrieval planning on
representative repository-investigation prompts.

## Acceptance Criteria

- [x] Verbose or debug output surfaces planner strategy, planner trace or step summary, stop reason, retained artifacts, and fallback causes for autonomous-gatherer turns. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] The repository includes proof or evaluation artifacts comparing static context assembly and autonomous retrieval planning on representative retrieval-heavy prompts. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] Foundational docs and configuration guidance describe when autonomous planning should be selected, how it falls back, and why heuristic planning is the default local strategy. [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end, proof: ac-3.log-->
