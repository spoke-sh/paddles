---
# system-managed
id: VFNvouNsS
status: done
updated_at: 2026-03-30T15:20:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
# authored
title: Coverage Confidence Field And Refinement Events
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvhauZg
index: 3
---

# Coverage Confidence Field And Refinement Events

## Summary

Add CoverageConfidence enum (High/Medium/Low) to InterpretationContext. Set based on refinement outcome: no gaps=High, gaps filled=Medium, unfilled gaps=Low. Add TurnEvent::InterpretationValidated and TurnEvent::InterpretationRefined variants with appropriate min_verbosity levels.

## Acceptance Criteria

- [x] InterpretationContext has a coverage_confidence field with CoverageConfidence enum [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [x] coverage_confidence set to High when no gaps detected [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end -->
- [x] coverage_confidence set to Medium when gaps filled by refinement [SRS-07/AC-03] <!-- verify: manual, SRS-07:start:end -->
- [x] coverage_confidence set to Low when unfilled gaps remain [SRS-07/AC-04] <!-- verify: manual, SRS-07:start:end -->
- [x] TurnEvent::InterpretationValidated emitted after validation pass [SRS-08/AC-05] <!-- verify: manual, SRS-08:start:end -->
- [x] TurnEvent::InterpretationRefined emitted after refinement cycle [SRS-09/AC-06] <!-- verify: manual, SRS-09:start:end -->
- [x] Both new events have appropriate min_verbosity levels [SRS-08/AC-07] <!-- verify: manual, SRS-08:start:end -->
