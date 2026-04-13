---
# system-managed
id: VGcZTH2gV
status: done
created_at: 2026-04-12T16:09:43
updated_at: 2026-04-12T17:00:02
# authored
title: Project External Capability Evidence And Degradation Across Surfaces
type: feat
operator-signal:
scope: VGb1c1XAL/VGcZRpCKi
index: 3
started_at: 2026-04-12T16:50:56
submitted_at: 2026-04-12T17:00:02
completed_at: 2026-04-12T17:00:02
---

# Project External Capability Evidence And Degradation Across Surfaces

## Summary

Project external capability state into trace and operator surfaces so active
fabrics, degraded outcomes, and provenance remain legible across TUI, web, API,
and docs once the recursive harness can already negotiate and normalize those
results.

## Acceptance Criteria

- [x] Tool absence, auth failure, or stale capability metadata degrades honestly with explicit runtime state and no false success. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] TUI, web, and API projections plus operator docs expose active fabrics, external result provenance, and degraded states using one shared vocabulary. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] External capability metadata and results remain observable through trace, transcript, and API surfaces. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
