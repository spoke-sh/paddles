---
# system-managed
id: VFNvmpil1
status: done
updated_at: 2026-03-30T15:15:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
# authored
title: Precedence Chain Extraction From Document Hierarchy
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvha5ZW
index: 2
---

# Precedence Chain Extraction From Document Hierarchy

## Summary

Extend the interpretation prompt to ask the model for the precedence chain given system -> user -> workspace document loading order. Add a precedence_chain field to InterpretationContext with source, rank, and scope_label per entry. Validate ranks are sequential; fall back to empty on invalid sequences.

## Acceptance Criteria

- [x] InterpretationContext has a precedence_chain field with source, rank, and scope_label [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [x] Interpretation prompt instructs model to state the precedence chain [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [x] Invalid rank sequences fall back to empty precedence chain [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end -->
- [x] Single-scope loading produces a single-entry precedence chain with rank 1 [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
