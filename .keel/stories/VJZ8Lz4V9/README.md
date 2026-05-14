---
# system-managed
id: VJZ8Lz4V9
status: icebox
created_at: 2026-05-13T21:29:49
updated_at: 2026-05-13T21:38:11
# authored
title: Migrate Legacy Runtime Lane Preferences
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 2
---

# Migrate Legacy Runtime Lane Preferences

## Summary

Keep legacy lane-shaped config readable as migration input while making the new
turn-runtime preference shape the only write target. Legacy Sift model-provider
values still hard-fail rather than remapping silently.

## Acceptance Criteria

- [ ] Migration fixture tests prove legacy runtime-lane config is read and normalized into turn-runtime preferences. [SRS-02/AC-01] <!-- verify: automated, SRS-02:start:end -->
- [ ] Persistence tests prove new writes use only the turn-runtime preference shape. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] Legacy lane config containing Sift model-provider values fails with the approved `ollama:<model>` migration hint. [SRS-NFR-01/AC-03] <!-- verify: automated, SRS-NFR-01:start:end -->
