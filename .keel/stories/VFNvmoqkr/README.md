---
# system-managed
id: VFNvmoqkr
status: backlog
created_at: 2026-03-30T13:35:23
updated_at: 2026-03-30T14:20:39
# authored
title: Typed Guidance Categories In Interpretation Schema
type: feat
operator-signal:
scope: VFNvH5LxS/VFNvha5ZW
index: 1
---

# Typed Guidance Categories In Interpretation Schema

## Summary

Extend the interpretation prompt to request typed guidance categories. Add a GuidanceCategory enum (Rules, Conventions, Constraints, Procedures, Preferences) to planning.rs. Add a categories field to InterpretationContext. Parse from model response with graceful fallback for unrecognized categories.

## Acceptance Criteria

- [ ] GuidanceCategory enum exists in planning.rs with five variants [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] InterpretationContext has a categories field with category, count, and sources per entry [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] build_interpretation_context_prompt instructs the model to return typed guidance categories [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] Unrecognized category values fall back gracefully without failing the interpretation [SRS-01/AC-04] <!-- verify: manual, SRS-01:start:end -->
