---
# system-managed
id: VFNvotYrG
status: icebox
created_at: 2026-03-30T13:35:31
updated_at: 2026-03-30T13:35:31
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

- [ ] Validation pass invoked after derive_interpretation_context in application/mod.rs [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Gaps detected triggers re-expansion and re-assembly [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] Total refinement model calls capped at 2 [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
- [ ] TurnEvents emitted for each refinement stage [SRS-02/AC-04] <!-- verify: manual, SRS-02:start:end -->
- [ ] Failure falls back to original single-pass context [SRS-02/AC-05] <!-- verify: manual, SRS-02:start:end -->
