---
# system-managed
id: VFesYkmXJ
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:21:22
# authored
title: Expose Steering Signal Manifold Replay And Live Projection
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 1
started_at: 2026-04-02T11:15:46
completed_at: 2026-04-02T11:21:22
---

# Expose Steering Signal Manifold Replay And Live Projection

## Summary

Expose a transit-backed manifold projection to the web layer so the browser can rebuild and stream steering signal state without inventing its own model. This slice defines the replay/live payloads for time-ordered signal frames, lineage anchors, lifecycle markers, and source references.

## Acceptance Criteria

- [x] Replay payloads expose time-ordered steering signal frames, influence snapshots, lineage anchors, and lifecycle state for the manifold route [SRS-01/AC-01] <!-- verify: cargo test -q manifold_, SRS-01:start:end -->
- [x] Replay payloads carry enough structured state that later UI slices do not need to invent chamber or conduit state client-side [SRS-01/AC-02] <!-- verify: cargo test -q manifold_, SRS-01:start:end -->
- [x] Projection contracts make manifold replay the authoritative rebuild path for the route instead of browser-local derived state [SRS-NFR-01/AC-03] <!-- verify: cargo test -q forensic_routes_project_conversation_and_turn_replay_with_lifecycle_states && cargo check -q, SRS-NFR-01:start:end -->
