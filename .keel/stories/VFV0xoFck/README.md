---
# system-managed
id: VFV0xoFck
status: in-progress
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T18:55:02
# authored
title: Surface Concrete Sift Retrieval Stages In Progress Events
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 2
started_at: 2026-03-31T18:55:02
---

# Surface Concrete Sift Retrieval Stages In Progress Events

## Summary

Expose what direct sift retrieval is doing while it runs so long searches explain their current stage, delay source, and remaining uncertainty instead of looking frozen.

## Acceptance Criteria

- [ ] Gatherer progress events distinguish retrieval execution stages such as initialization, indexing, retrieval, ranking, and completion or fallback. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [ ] User-facing progress does not present internal autonomous planner labels like `Terminate` as the primary status for direct retrieval turns. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [ ] Long-running direct retrieval continues to emit periodic progress updates instead of leaving the UI stagnant. [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
- [ ] Trace output and summaries remain specific enough to explain why retrieval is slow or why ETA is unknown. [SRS-NFR-03/AC-04] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-4.log-->
