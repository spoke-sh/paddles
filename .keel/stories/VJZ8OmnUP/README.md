---
# system-managed
id: VJZ8OmnUP
status: icebox
created_at: 2026-05-13T21:30:00
updated_at: 2026-05-13T21:36:51
# authored
title: Rename Internal Lane Types To Turn Runtime Concepts
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 1
---

# Rename Internal Lane Types To Turn Runtime Concepts

## Summary

Rename internal Rust runtime lane types to turn runtime concepts. This should
change active architecture names, not just user-visible labels.

## Acceptance Criteria

- [ ] Internal types such as `RuntimeLaneConfig`, `PreparedRuntimeLanes`, `PreparedModelLane`, and `PreparedGathererLane` are replaced with turn-runtime/model-client/retrieval concepts or documented migration shims. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] Tests and module names use the new turn-runtime vocabulary where they describe active runtime architecture. [SRS-01/AC-02] <!-- verify: automated, SRS-01:start:end -->
- [ ] Behavior stays covered by existing runtime construction and turn-loop tests after the rename. [SRS-NFR-01/AC-03] <!-- verify: automated, SRS-NFR-01:start:end -->
