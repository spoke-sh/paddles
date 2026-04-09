---
# system-managed
id: VGGIuVrgs
status: done
created_at: 2026-04-08T20:45:56
updated_at: 2026-04-08T21:15:23
# authored
title: Render Diverters Jams And Outputs In Transit
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqtM2e
index: 2
started_at: 2026-04-08T21:13:13
submitted_at: 2026-04-08T21:15:21
completed_at: 2026-04-08T21:15:23
---

# Render Diverters Jams And Outputs In Transit

## Summary

Give the transit stage explicit visual treatment for diverters, jams, replans, steering-force touchpoints, and output bins so operators can read direction changes at a glance.

## Acceptance Criteria

- [x] The transit machine stage visually distinguishes forward progression, diversions, jams, and completed outputs using the shared machine-moment vocabulary. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Selecting a transit moment reveals a concise causal explanation instead of raw-node-first detail. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] Diverters, jams, replans, and outputs are all represented as distinct transit machine parts so operators can see why the turn changed direction. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
