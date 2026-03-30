---
# system-managed
id: VFNyqELsS
status: icebox
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T13:47:31
# authored
title: Upstream Sift Progress Callback Requirements
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 4
---

# Upstream Sift Progress Callback Requirements

## Summary

Document the upstream requirements for sift to expose a progress callback mechanism. This is a bearing/ADR that specifies what paddles needs from sift's search_autonomous API to provide granular progress: indexing phase with file count/total, planner step-by-step emissions, and estimated completion time. Delivered as a keel bearing or ADR artifact.

## Acceptance Criteria

- [ ] Requirements document specifies the callback shape paddles needs (trait, channel, or closure) [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Document defines the progress phases sift should report (indexing, embedding, planning, retrieving) [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] Document specifies the data each phase should carry (file count, total, step index, etc.) [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end -->
- [ ] Document identifies the integration seam (parameter on search_autonomous, builder config, etc.) [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end -->
- [ ] Delivered as a keel bearing or ADR artifact in .keel/ [SRS-05/AC-05] <!-- verify: manual, SRS-05:start:end -->
