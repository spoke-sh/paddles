---
# system-managed
id: VGEVvrJOP
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Extract Shell Transcript And Composer Surfaces
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsWLk2
index: 2
---

# Extract Shell Transcript And Composer Surfaces

## Summary

Extract the runtime shell, transcript rendering, and composer behavior into dedicated modules while preserving current interaction behavior and test contracts.

## Acceptance Criteria

- [ ] The runtime shell delegates transcript and composer rendering to dedicated modules instead of owning those concerns inline in the root runtime app file. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Multiline paste compression, prompt history recall, sticky-tail scrolling, and transcript-driven manifold turn selection remain covered as preserved behavior in the same slice. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
