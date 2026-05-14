---
# system-managed
id: VJZ8N7avc
status: icebox
created_at: 2026-05-13T21:29:54
updated_at: 2026-05-13T21:36:44
# authored
title: Delete Sift Agent And Planner Inference Adapters
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 1
---

# Delete Sift Agent And Planner Inference Adapters

## Summary

Delete the Sift action-selection and final-rendering inference adapters after
HTTP-only runtime construction is proven. Any remaining Sift code must be
retrieval-specific or compatibility parsing that fails before runtime.

## Acceptance Criteria

- [ ] Compile failures or targeted tests first identify every remaining active reference to Sift inference adapters. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] Sift action-selection and final-rendering inference adapters are deleted or made unreachable from runtime construction. [SRS-01/AC-02] <!-- verify: automated, SRS-01:start:end -->
- [ ] Legacy Sift model-provider inputs still fail with the approved migration hint rather than panicking or falling through. [SRS-NFR-02/AC-03] <!-- verify: automated, SRS-NFR-02:start:end -->
