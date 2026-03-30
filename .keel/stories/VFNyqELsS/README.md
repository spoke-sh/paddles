---
# system-managed
id: VFNyqELsS
status: backlog
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T14:13:12
# authored
title: Upstream Sift Progress Callback Requirements
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 4
---

# Upstream Sift Progress Callback Requirements

## Summary

Document the upstream requirements for sift to expose a progress callback mechanism. Delivered as a keel bearing or ADR. Specifies callback shape, progress phases, phase data, and integration seam.

## Acceptance Criteria

- [ ] Document specifies the callback shape paddles needs (trait, channel, or closure) [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [ ] Document defines progress phases sift should report (indexing, embedding, planning, retrieving) [SRS-08/AC-02] <!-- verify: manual, SRS-08:start:end -->
- [ ] Document specifies data each phase carries (file count, total, step index) [SRS-08/AC-03] <!-- verify: manual, SRS-08:start:end -->
- [ ] Document identifies the integration seam on search_autonomous [SRS-08/AC-04] <!-- verify: manual, SRS-08:start:end -->
- [ ] Delivered as a keel bearing or ADR artifact [SRS-07/AC-05] <!-- verify: manual, SRS-07:start:end -->
