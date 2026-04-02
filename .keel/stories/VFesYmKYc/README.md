---
# system-managed
id: VFesYmKYc
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:47:43
# authored
title: Stream Active-Turn Steering Signal Changes Into The Manifold View
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 5
started_at: 2026-04-02T11:43:00
completed_at: 2026-04-02T11:47:43
---

# Stream Active-Turn Steering Signal Changes Into The Manifold View

## Summary

Stream active-turn signal changes into the manifold route so the route is useful during live debugging rather than only after the turn finishes. This slice makes provisional states visible, reconciles them to final state, and keeps replay as the recovery path.

## Acceptance Criteria

- [x] Active turns update the manifold route with provisional and final signal changes without reload [SRS-06/AC-01] <!-- verify: cargo test -q manifold_route_html_streams_live_updates_and_reconciles_from_replay && cargo test -q infrastructure::web::tests && cargo check -q, SRS-06:start:end -->
- [x] Provisional, superseded, and final manifold states are visibly distinguishable during live turns [SRS-06/AC-02] <!-- verify: cargo test -q manifold_route_html_surfaces_lifecycle_states_during_live_turns && cargo test -q manifold_route_html_encodes_temporal_signal_phases, SRS-06:start:end -->
- [x] Missed live updates reconcile correctly from replay without leaving stale manifold state behind [SRS-NFR-01/AC-03] <!-- verify: cargo test -q manifold_route_html_streams_live_updates_and_reconciles_from_replay && cargo check -q, SRS-NFR-01:start:end -->
