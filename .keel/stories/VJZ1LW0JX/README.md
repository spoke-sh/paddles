---
# system-managed
id: VJZ1LW0JX
status: backlog
created_at: 2026-05-13T21:01:59
updated_at: 2026-05-13T21:03:32
# authored
title: Map Lane Concepts To Turn Loop Phases
type: chore
operator-signal:
scope: VJZ0tpZQJ/VJZ14yp0U
index: 3
---

# Map Lane Concepts To Turn Loop Phases

## Summary

Map planner, synthesizer, and gatherer lane concepts across source, tests,
configuration, prompts, and docs, then classify each one as public vocabulary to
retire or an internal turn-loop phase/helper to preserve.

## Acceptance Criteria

- [ ] Inventory lists public lane concepts across CLI/config docs, runtime state, prepared lane structs, tests, prompts, events, and foundational docs. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Mapping identifies the target turn-loop phase or helper for each preserved concept: capability discovery, action selection, retrieval, execution, evidence accumulation, final rendering, or compatibility layer. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] Mapping identifies concepts that should disappear from public operator-facing vocabulary and the tests/docs that must change with them. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
