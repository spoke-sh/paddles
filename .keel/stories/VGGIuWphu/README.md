---
# system-managed
id: VGGIuWphu
status: backlog
created_at: 2026-04-08T20:45:56
updated_at: 2026-04-08T20:49:26
# authored
title: Collapse Forensic Selection To Moments And Internals
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqts2y
index: 1
---

# Collapse Forensic Selection To Moments And Internals

## Summary

Replace the current forensic conversation/turn/record selection maze with the same turn + moment + internals model used by the transit machine.

## Acceptance Criteria

- [ ] The forensic route adopts the shared turn + moment + internals selection model instead of route-specific selection semantics. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The default forensic selection path collapses operator choices instead of preserving the current conversation/turn/record mode maze. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] Default forensic navigation no longer requires separate conversation/turn/record mode switching to understand a turn. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
