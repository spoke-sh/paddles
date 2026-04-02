---
# system-managed
id: VFbXMCVl7
status: backlog
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T21:29:42
# authored
title: Add Force Overview And Shadow Comparison To The Web Inspector
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 4
---

# Add Force Overview And Shadow Comparison To The Web Inspector

## Summary

Add the secondary visualization layer above the precise inspector so force and topology become legible at a glance. This slice focuses on default-visible force panels, contribution-by-source, and a shadow comparison against the previous artifact in lineage while keeping the precise 2D inspector primary.

## Acceptance Criteria

- [ ] The default inspector surface shows force magnitudes and contribution-by-source for the current lineage selection [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [ ] A secondary overview above the precise inspector visualizes topology and force state without replacing the 2D inspector [SRS-08/AC-02] <!-- verify: manual, SRS-08:start:end -->
- [ ] The inspector can compare the current selection against the previous artifact in lineage as a shadow baseline [SRS-08/AC-03] <!-- verify: manual, SRS-08:start:end -->
- [ ] Any new overview visualization dependency is served locally or vendored so the feature remains local-first [SRS-NFR-04/AC-04] <!-- verify: review, SRS-NFR-04:start:end -->
