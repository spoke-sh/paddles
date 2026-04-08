---
# system-managed
id: VGEVvt8SE
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Modularize The Transit Route Surface
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsWxjv
index: 3
---

# Modularize The Transit Route Surface

## Summary

Break the transit route into dedicated toolbar, board, layout, and node-rendering modules so trace-board behavior can evolve without reopening one monolithic route implementation.

## Acceptance Criteria

- [ ] The transit route composes dedicated modules/hooks for toolbar state, board layout, pan/zoom behavior, and node rendering. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Existing transit toggles, zoom/pan behavior, and trace rendering remain covered by route-level tests after the split. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
