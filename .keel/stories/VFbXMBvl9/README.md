---
# system-managed
id: VFbXMBvl9
status: backlog
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T21:29:42
# authored
title: Record Context Lineage And Force Snapshots In Transit
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 2
---

# Record Context Lineage And Force Snapshots In Transit

## Summary

Capture the lineage and force metadata that explains how context was assembled and constrained. Transit should record lineage edges plus force snapshots and source-contribution estimates so the web inspector can explain not only what happened, but why.

## Acceptance Criteria

- [ ] Transit records lineage edges between conversation, turn, model call, planner step, artifacts, and resulting outputs [SRS-02/AC-01] <!-- verify: test, SRS-02:start:end -->
- [ ] Transit records force snapshots for pressure, truncation/compaction, execution/edit pressure, fallback/coercion, and budget effects at the relevant steps [SRS-03/AC-02] <!-- verify: test, SRS-03:start:end -->
- [ ] Transit records contribution estimates by source alongside the applied forces using a documented heuristic/controller-derived model [SRS-03/AC-03] <!-- verify: review, SRS-03:start:end -->
- [ ] Forensic replay can order lineage and force records coherently for a selected turn [SRS-NFR-01/AC-04] <!-- verify: test, SRS-NFR-01:start:end -->
