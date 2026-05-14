---
# system-managed
id: VJZ8QTY4i
status: icebox
created_at: 2026-05-13T21:30:07
updated_at: 2026-05-13T21:36:55
# authored
title: Retire Runtime Lane Language From Public Surfaces
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 3
---

# Retire Runtime Lane Language From Public Surfaces

## Summary

Retire runtime lane language from public surfaces after the internal rename is
complete. Any remaining old terms must be explicit compatibility aliases or
historical artifacts, not active product vocabulary.

## Acceptance Criteria

- [ ] CLI help, TUI/web route copy, docs, and prompt prose no longer present planner, synthesizer, or gatherer as runtime lanes. [SRS-03/AC-01] <!-- verify: automated, SRS-03:start:end -->
- [ ] String scans or targeted tests cover old public phrases such as "planner lane", "synthesizer lane", "gatherer lane", and "runtime lanes". [SRS-03/AC-02] <!-- verify: automated, SRS-03:start:end -->
- [ ] Retained legacy aliases are documented as migration shims and point to action-selection, final-rendering, or retrieval terminology. [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end -->
- [ ] Tests prove turn-loop behavior remains covered for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-05/AC-04] <!-- verify: automated, SRS-05:start:end -->
