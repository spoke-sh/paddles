---
# system-managed
id: VFesYldXL
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:37:23
# authored
title: Project Steering Signals Into Chambers And Conduits
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 3
started_at: 2026-04-02T11:32:54
completed_at: 2026-04-02T11:37:23
---

# Project Steering Signals Into Chambers And Conduits

## Summary

Define the signal-topology mapping that turns steering signal families and lineage structure into chambers, conduits, reservoirs, valves, or equivalent expressive primitives. This slice gives the manifold metaphor real semantics instead of treating it as an arbitrary skin.

## Acceptance Criteria

- [x] Steering signal families and lineage structure map to stable manifold primitives such as chambers, conduits, reservoirs, or valves [SRS-03/AC-01] <!-- verify: cargo test -q projection_maps_signal_families_and_lineage_anchors_into_stable_primitives && cargo test -q manifold_route_html_renders_topology_primitives_and_conduits, SRS-03:start:end -->
- [x] The topology mapping defines which signal families feed which manifold primitives before time-based rendering is applied [SRS-03/AC-02] <!-- verify: cargo test -q projection_builds_cumulative_frames_from_signal_snapshots && cargo test -q manifold_route_html_renders_topology_primitives_and_conduits, SRS-03:start:end -->
- [x] Every rendered manifold primitive has an evidence anchor or explicit lineage basis so the metaphor remains interpretable [SRS-NFR-04/AC-03] <!-- verify: cargo test -q projected_topology_keeps_evidence_or_lineage_basis_for_every_primitive && cargo check -q, SRS-NFR-04:start:end -->
