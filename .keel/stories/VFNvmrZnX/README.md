---
# system-managed
id: VFNvmrZnX
status: done
updated_at: 2026-03-30T15:15:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
# authored
title: Validation Pass For Coverage Gap Detection
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvha5ZW
index: 4
---

# Validation Pass For Coverage Gap Detection

## Summary

After initial interpretation assembly, run a second model call that receives the assembled context + user prompt and asks what areas lack guidance coverage. Returns Vec of {area, suggestion}. Implemented as a standalone function, not yet wired into the main loop.

## Acceptance Criteria

- [x] A standalone function accepts InterpretationContext and user prompt, returns Vec<{area, suggestion}> [SRS-08/AC-01] <!-- verify: manual, SRS-08:start:end -->
- [x] The function makes a model call asking the model to identify gaps [SRS-09/AC-02] <!-- verify: manual, SRS-09:start:end -->
- [x] Model response parsed into structured gap entries [SRS-08/AC-03] <!-- verify: manual, SRS-08:start:end -->
- [x] No gaps detected returns an empty Vec [SRS-08/AC-04] <!-- verify: manual, SRS-08:start:end -->
- [x] Function is callable independently; not wired into the application main loop [SRS-08/AC-05] <!-- verify: manual, SRS-08:start:end -->
