---
# system-managed
id: VGGIuWphu
status: done
created_at: 2026-04-08T20:45:56
updated_at: 2026-04-08T21:29:27
# authored
title: Collapse Forensic Selection To Moments And Internals
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqts2y
index: 1
started_at: 2026-04-08T21:23:31
submitted_at: 2026-04-08T21:29:24
completed_at: 2026-04-08T21:29:27
---

# Collapse Forensic Selection To Moments And Internals

## Summary

Replace the current forensic conversation/turn/record selection maze with the same turn + moment + internals model used by the transit machine.

## Acceptance Criteria

- [x] The forensic route adopts the shared turn + moment + internals selection model instead of route-specific selection semantics. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The default forensic selection path collapses operator choices instead of preserving the current conversation/turn/record mode maze. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Default forensic navigation no longer requires separate conversation/turn/record mode switching to understand a turn. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
