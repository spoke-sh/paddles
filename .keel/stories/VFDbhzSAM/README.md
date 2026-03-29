---
# system-managed
id: VFDbhzSAM
status: in-progress
created_at: 2026-03-28T19:12:55
updated_at: 2026-03-28T19:15:10
# authored
title: Wire Live Turn Events And Progressive Assistant Rendering
type: feat
operator-signal:
scope: VFDbdzqtU/VFDbfLe0E
index: 3
started_at: 2026-03-28T19:15:10
---

# Wire Live Turn Events And Progressive Assistant Rendering

## Summary

Bridge live paddles turn events and final assistant answers into the TUI so
interactive turns feel alive and visibly structured while preserving grounded
final content.

## Acceptance Criteria

- [ ] Live `TurnEvent` output is rendered inside the TUI transcript as action rows during turn execution. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] Final assistant answers render progressively in the transcript and preserve the final grounded/cited content from paddles. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] Tests or transcript proofs cover live event rendering and progressive assistant output. [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end -->
