---
# system-managed
id: VFesYm0Yk
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:42:25
# authored
title: Render Time-Scrubbable Manifold Accumulation And Flow
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 4
started_at: 2026-04-02T11:38:40
completed_at: 2026-04-02T11:42:25
---

# Render Time-Scrubbable Manifold Accumulation And Flow

## Summary

Render the manifold states over time so operators can watch accumulation, stabilization, supersession, and bleed-off across a selected conversation or turn. This slice owns the temporal experience: replay, pause, scrub, and the visual language of fill, opacity, and conduit activity.

## Acceptance Criteria

- [x] Chamber and conduit visuals change over time according to accumulation, stabilization, supersession, and bleed-off state [SRS-04/AC-01] <!-- verify: cargo test -q manifold_route_html_encodes_temporal_signal_phases && cargo test -q manifold_route_html_renders_topology_primitives_and_conduits, SRS-04:start:end -->
- [x] Operators can pause, replay, and scrub manifold state over time for the selected conversation or turn [SRS-05/AC-02] <!-- verify: cargo test -q manifold_route_html_supports_time_scrub_controls && cargo test -q manifold_route_html_renders_topology_primitives_and_conduits, SRS-05:start:end -->
- [x] Time-based rendering remains usable on long histories without unbounded repaint or layout churn [SRS-NFR-02/AC-03] <!-- verify: cargo test -q manifold_route_html_uses_bounded_local_scrollers && cargo check -q, SRS-NFR-02:start:end -->
