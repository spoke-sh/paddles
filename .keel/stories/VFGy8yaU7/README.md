---
# system-managed
id: VFGy8yaU7
status: done
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:40:49
# authored
title: Upgrade Sift And Expose Graph Mode In Gatherer Config
type: feat
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 1
started_at: 2026-03-29T09:31:28
submitted_at: 2026-03-29T09:40:45
completed_at: 2026-03-29T09:40:49
---

# Upgrade Sift And Expose Graph Mode In Gatherer Config

## Summary

Advance the pinned `sift` dependency to the latest upstream revision that
includes bounded graph-mode autonomous search, then extend the gatherer-facing
planning config so `paddles` can request bounded `linear` versus `graph`
retrieval without introducing repository-specific routing types.

## Acceptance Criteria

- [x] `Cargo.lock` advances to the latest upstream `sift` revision that includes graph-mode autonomous search and branching/episode APIs. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] The gatherer/planning config can express bounded `linear` versus `graph` autonomous retrieval along with optional planner profile selection. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [x] The config surface remains generic across repositories and evidence domains rather than introducing a board-specific or Keel-specific route selector. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
