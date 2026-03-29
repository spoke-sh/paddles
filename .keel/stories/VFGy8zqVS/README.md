---
# system-managed
id: VFGy8zqVS
status: done
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:40:49
# authored
title: Route Recursive Search Through Graph-Capable Sift Gatherers
type: feat
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 3
started_at: 2026-03-29T09:39:59
submitted_at: 2026-03-29T09:40:45
completed_at: 2026-03-29T09:40:49
---

# Route Recursive Search Through Graph-Capable Sift Gatherers

## Summary

Use the new graph-capable gatherer path from the existing model-directed
recursive harness so search/refine work can benefit from bounded graph-mode
retrieval while preserving local-first fallback behavior and avoiding new
repository-specific top-level intents.

## Acceptance Criteria

- [x] Recursive search/refine work can request graph-capable gatherer behavior through the generic planner/gatherer path. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [x] Graph-mode gatherers remain local-first, bounded, and fail closed when graph planning is invalid or unavailable. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
- [x] Recursive planner/synthesizer handoff continues to operate with graph-capable gathered evidence instead of flattening the path back into an opaque linear summary. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] The default operator surface renders concise graph planner summaries, branch/frontier state, and graph stop reasons when graph-mode retrieval is active. [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end -->
- [x] The graph-capable route remains compatible with a future embedded recorder and does not assume a networked trace server. [SRS-NFR-04/AC-05] <!-- verify: manual, SRS-NFR-04:start:end -->
