---
# system-managed
id: VFesYl8XK
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:31:46
# authored
title: Add A Dedicated Steering Signal Manifold Route To The Web UI
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 2
started_at: 2026-04-02T11:23:01
completed_at: 2026-04-02T11:31:46
---

# Add A Dedicated Steering Signal Manifold Route To The Web UI

## Summary

Add the dedicated web route and shell for the steering signal manifold. This slice establishes the separate surface, preserves the existing precise forensic inspector, and gives the manifold route room for its own canvas, timeline, and detail panes.

## Acceptance Criteria

- [x] The web UI exposes a dedicated steering signal manifold route distinct from the precise forensic inspector [SRS-02/AC-01] <!-- verify: cargo test -q web_router_serves_dedicated_manifold_and_transit_routes && cargo test -q web_html_exposes_manifold_route_shell_and_path_sync, SRS-02:start:end -->
- [x] The route layout makes the manifold visualization primary on that route while preserving room for detail and source panes [SRS-02/AC-02] <!-- verify: cargo test -q web_html_exposes_manifold_route_shell_and_path_sync && cargo check -q, SRS-02:start:end -->
- [x] The route remains usable for long conversations through bounded local scrolling and without page-level overflow churn [SRS-NFR-02/AC-03] <!-- verify: cargo test -q manifold_route_html_uses_bounded_local_scrollers && cargo check -q, SRS-NFR-02:start:end -->
