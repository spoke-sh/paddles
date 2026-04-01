---
# system-managed
id: VFYc53IkA
status: backlog
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T09:28:09
# authored
title: Separate Transcript Updates From Turn Progress Events
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 4
---

# Separate Transcript Updates From Turn Progress Events

## Summary

Introduce a dedicated transcript update path so prompt/response visibility no longer depends on `TurnEvent` timing. Progress events stay available for trace/activity rendering, but transcript changes move onto their own signal path and reconciliation flow.

## Acceptance Criteria

- [ ] Transcript update delivery is emitted independently of `TurnEvent` progress completion [SRS-03/AC-01] <!-- verify: test, SRS-03:start:end -->
- [ ] The dedicated transcript update path can notify surfaces without inferring transcript state from progress-event timing [SRS-03/AC-02] <!-- verify: test, SRS-03:start:end -->
