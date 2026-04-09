---
# system-managed
id: VGLDQBFqJ
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:07
# authored
title: Define Harness Profile Model For Steering And Compaction
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMvU4i
index: 1
---

# Define Harness Profile Model For Steering And Compaction

## Summary

Define the explicit harness-profile model that controls steering, compaction, and recovery policy. This story should replace hidden provider-shaped heuristics with a versionable profile contract.

## Acceptance Criteria

- [ ] The runtime defines explicit harness-profile semantics for steering and compaction instead of relying on untracked provider-specific heuristics [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Profile selection and downgrade behavior are explicit enough to be surfaced in docs, tests, and trace projections [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
