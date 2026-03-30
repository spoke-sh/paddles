---
# system-managed
id: VFNvmqkmD
status: icebox
created_at: 2026-03-30T13:35:23
updated_at: 2026-03-30T13:35:23
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

- [ ] InterpretationContext has a conflicts field with sources, description, and resolution per entry [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Interpretation prompt instructs model to identify conflicts and state resolutions [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] No conflicts detected produces an empty Vec without error [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] Each conflict entry references at least two sources [SRS-03/AC-04] <!-- verify: manual, SRS-03:start:end -->
