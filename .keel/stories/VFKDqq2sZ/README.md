---
# system-managed
id: VFKDqq2sZ
status: needs-human-verification
created_at: 2026-03-29T22:22:03
updated_at: 2026-03-29T22:32:17
# authored
title: Trace Graph Endpoint And Hexagonal Railroad Visualization
type: feat
operator-signal:
scope: VFKBFMq8J/VFKDmXte3
index: 1
started_at: 2026-03-29T22:32:17
submitted_at: 2026-03-29T22:32:17
---

# Trace Graph Endpoint And Hexagonal Railroad Visualization

## Summary

Add a GET /sessions/:id/trace/graph endpoint that converts TraceReplay into flat node/edge/branch JSON, and render the result as an SVG railroad diagram embedded in the chat page. Hexagonal nodes are color-coded by TraceRecordKind, branch divergence shows as parallel swimlanes, and merge records converge lanes back. The visualization updates in real time as SSE delivers new TraceRecords.

## Acceptance Criteria

- [x] Trace graph endpoint returns structured node/edge/branch JSON from TraceReplay. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] SVG visualization renders hexagonal nodes in a vertical railroad-style flow. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [x] Node color and label reflect TraceRecordKind (root, action, tool, checkpoint, merge). [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] Branch divergence renders as parallel swimlanes splitting from the mainline. [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [x] Merge records render as lanes converging back. [SRS-05/AC-05] <!-- verify: manual, SRS-05:start:end -->
- [x] Visualization updates in real time as new TraceRecords arrive via SSE. [SRS-06/AC-06] <!-- verify: manual, SRS-06:start:end -->
