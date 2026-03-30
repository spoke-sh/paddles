---
# system-managed
id: VFNvotYrG
status: done
updated_at: 2026-03-30T15:20:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
# authored
title: Wire Refinement Loop Into Application Layer
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvhauZg
index: 2
---

# Wire Refinement Loop Into Application Layer

## Summary

In application/mod.rs, after derive_interpretation_context, call the validation pass. If gaps found, trigger re-expansion + re-assembly. Cap at 2 total refinement model calls. Emit TurnEvents at each stage. Fall back to single-pass result on any failure.

## Acceptance Criteria

- [x] Validation pass invoked after derive_interpretation_context in application/mod.rs [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [x] Gaps detected triggers re-expansion and re-assembly [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [x] Total refinement model calls capped at 2 [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end -->
- [x] TurnEvents emitted for each refinement stage [SRS-06/AC-04] <!-- verify: manual, SRS-06:start:end -->
- [x] Failure falls back to original single-pass context [SRS-04/AC-05] <!-- verify: manual, SRS-04:start:end -->
