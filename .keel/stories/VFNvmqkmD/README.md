---
# system-managed
id: VFNvmqkmD
status: done
updated_at: 2026-03-30T15:15:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
# authored
title: Conflict Detection Between Guidance Sources
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvha5ZW
index: 3
---

# Conflict Detection Between Guidance Sources

## Summary

Extend the interpretation prompt to ask the model to identify conflicts between guidance documents and state resolutions. Add a conflicts field to InterpretationContext. Empty Vec is valid (no conflicts). Each conflict entry must reference at least two sources.

## Acceptance Criteria

- [x] InterpretationContext has a conflicts field with sources, description, and resolution per entry [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [x] Interpretation prompt instructs model to identify conflicts and state resolutions [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end -->
- [x] No conflicts detected produces an empty Vec without error [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
- [x] Each conflict entry references at least two sources [SRS-06/AC-04] <!-- verify: manual, SRS-06:start:end -->
