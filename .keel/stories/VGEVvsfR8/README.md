---
# system-managed
id: VGEVvsfR8
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Modularize The Inspector Route Surface
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsWxjv
index: 2
---

# Modularize The Inspector Route Surface

## Summary

Break the inspector route into dedicated modules for overview, navigation, record selection, and detail presentation so route-local behavior no longer lives in one large route body.

## Acceptance Criteria

- [ ] The inspector route composes dedicated modules/hooks for overview, navigation, records, and detail panes instead of one monolithic implementation block. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Existing inspector selection, focus, and detail behavior remain covered by route-level regressions after the split. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
