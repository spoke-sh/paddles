---
# system-managed
id: VFJ5wSYep
status: backlog
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T17:47:05
# authored
title: Remove Legacy Direct Routing Heuristics From Remaining Turn Paths
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 1
---

# Remove Legacy Direct Routing Heuristics From Remaining Turn Paths

## Summary

Remove the remaining legacy direct-path reasoning helpers such as string-based
casual/tool/follow-up inference from the primary harness path so bounded
model-selected actions remain the source of reasoning even when the turn does
not immediately recurse through the planner loop.

## Acceptance Criteria

- [ ] Remaining legacy direct-path routing/tool-inference heuristics are removed or demoted so they no longer decide the primary turn path. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] Any retained direct-path fallback stays clearly fail-closed and controller-owned rather than silently reasoning for the model. [SRS-NFR-01/AC-02] <!-- verify: automated, SRS-NFR-01:start:end -->
- [ ] Transcript/tests prove the model-selected path still handles conversational versus workspace turns without the old string classifier. [SRS-01/AC-03] <!-- verify: automated, SRS-01:start:end -->
