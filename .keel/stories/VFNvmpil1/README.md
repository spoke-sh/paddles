---
# system-managed
id: VFNvmpil1
status: icebox
created_at: 2026-03-30T13:35:23
updated_at: 2026-03-30T13:35:23
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

- [ ] InterpretationContext has a precedence_chain field with source, rank, and scope_label [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Interpretation prompt instructs model to state the precedence chain [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] Invalid rank sequences fall back to empty precedence chain [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
- [ ] Single-scope loading produces a single-entry precedence chain with rank 1 [SRS-02/AC-04] <!-- verify: manual, SRS-02:start:end -->
