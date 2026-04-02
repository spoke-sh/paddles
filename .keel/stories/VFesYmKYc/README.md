---
# system-managed
id: VFesYmKYc
status: backlog
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:10:13
# authored
title: Stream Active-Turn Steering Signal Changes Into The Manifold View
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 5
---

# Stream Active-Turn Steering Signal Changes Into The Manifold View

## Summary

Stream active-turn signal changes into the manifold route so the route is useful during live debugging rather than only after the turn finishes. This slice makes provisional states visible, reconciles them to final state, and keeps replay as the recovery path.

## Acceptance Criteria

- [ ] Active turns update the manifold route with provisional and final signal changes without reload [SRS-06/AC-01] <!-- verify: test, SRS-06:start:end -->
- [ ] Provisional, superseded, and final manifold states are visibly distinguishable during live turns [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] Missed live updates reconcile correctly from replay without leaving stale manifold state behind [SRS-NFR-01/AC-03] <!-- verify: test, SRS-NFR-01:start:end -->
