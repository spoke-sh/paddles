---
# system-managed
id: VFEDDsIF6
status: backlog
created_at: 2026-03-28T21:41:55
updated_at: 2026-03-28T21:48:08
# authored
title: Feed Interpretation Context Into First Action Selection
type: feat
operator-signal:
scope: VFECyWLL6/VFED2RjSu
index: 2
---

# Feed Interpretation Context Into First Action Selection

## Summary

Move `AGENTS.md`, linked foundational docs, recent turns, and relevant local
state into the first action-selection prompt so the model chooses its initial
bounded action from interpretation context rather than after a controller
shortcut.

## Acceptance Criteria

- [ ] Non-trivial turns assemble interpretation context before first bounded action selection. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The first action-selection prompt or contract demonstrably includes operator memory and linked foundational guidance. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Interpretation-context assembly remains observable in the default user surface. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
