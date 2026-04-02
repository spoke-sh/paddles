---
# system-managed
id: VFbXMCHl6
status: backlog
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T21:29:42
# authored
title: Expose Web Inspector Replay And Live Projection APIs
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 3
---

# Expose Web Inspector Replay And Live Projection APIs

## Summary

Expose transit-backed forensic data to the browser through replay and live update projection APIs. The web layer should be able to rebuild the inspector from replay and receive provisional/final artifact updates during active turns without treating the DOM as the source of truth.

## Acceptance Criteria

- [ ] The application/web layer exposes conversation- or turn-scoped replay for forensic artifacts, lineage edges, and force snapshots [SRS-04/AC-01] <!-- verify: test, SRS-04:start:end -->
- [ ] Replay payloads distinguish provisional, superseded, and final artifact states [SRS-04/AC-02] <!-- verify: test, SRS-04:start:end -->
- [ ] Live updates deliver forensic artifact changes without requiring page reload and remain recoverable through replay [SRS-04/AC-03] <!-- verify: test, SRS-04:start:end -->
- [ ] Replay is sufficient to rebuild the forensic inspector after missed live updates without UI-local repair heuristics [SRS-NFR-01/AC-04] <!-- verify: test, SRS-NFR-01:start:end -->
