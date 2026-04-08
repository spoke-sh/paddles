---
# system-managed
id: VGEVvsMPy
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Modularize The Manifold Route Surface
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsWxjv
index: 1
---

# Modularize The Manifold Route Surface

## Summary

Break the manifold route into dedicated stage, viewport, playback, camera, gate-field, and readout modules so the temporal steering surface has clear internal seams.

## Acceptance Criteria

- [ ] The manifold route composes dedicated modules/hooks for playback state, camera interaction, gate-field derivation, and readout presentation. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Existing manifold controls and interactions, including transcript-driven turn selection, playback, pan/tilt/rotate, and zoom behavior, remain regression-covered after extraction. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
