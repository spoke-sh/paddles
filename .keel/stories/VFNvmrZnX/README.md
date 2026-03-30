---
# system-managed
id: VFNvmrZnX
status: icebox
created_at: 2026-03-30T13:35:23
updated_at: 2026-03-30T13:35:23
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

- [ ] A standalone function accepts InterpretationContext and user prompt, returns Vec<{area, suggestion}> [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] The function makes a model call asking the model to identify gaps [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [ ] Model response parsed into structured gap entries [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end -->
- [ ] No gaps detected returns an empty Vec [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [ ] Function is callable independently; not wired into the application main loop [SRS-04/AC-05] <!-- verify: manual, SRS-04:start:end -->
