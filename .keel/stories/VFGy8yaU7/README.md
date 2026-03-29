---
# system-managed
id: VFGy8yaU7
status: backlog
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:05:52
# authored
title: Upgrade Sift And Expose Graph Mode In Gatherer Config
type: feat
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 1
---

# Upgrade Sift And Expose Graph Mode In Gatherer Config

## Summary

Advance the pinned `sift` dependency to the latest upstream revision that
includes bounded graph-mode autonomous search, then extend the gatherer-facing
planning config so `paddles` can request bounded `linear` versus `graph`
retrieval without introducing repository-specific routing types.

## Acceptance Criteria

- [ ] `Cargo.lock` advances to the latest upstream `sift` revision that includes graph-mode autonomous search and branching/episode APIs. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The gatherer/planning config can express bounded `linear` versus `graph` autonomous retrieval along with optional planner profile selection. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] The config surface remains generic across repositories and evidence domains rather than introducing a board-specific or Keel-specific route selector. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
