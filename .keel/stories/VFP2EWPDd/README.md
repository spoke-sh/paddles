---
# system-managed
id: VFP2EWPDd
status: done
created_at: 2026-03-30T18:07:16
updated_at: 2026-03-30T18:34:17
# authored
title: Update ArtifactEnvelope To Carry Typed ContextLocator
type: feat
operator-signal:
scope: VFOmKssE5/VFOvGdksF
index: 3
started_at: 2026-03-30T18:41:00
completed_at: 2026-03-30T18:34:17
---

# Update ArtifactEnvelope To Carry Typed ContextLocator

## Summary

Migrate the `ArtifactEnvelope` structure to use the typed `ContextLocator` enum instead of a bare string for its `locator` field. This enables programmatic resolution of truncated artifacts.

## Acceptance Criteria

- [x] ArtifactEnvelope locator field accepts ContextLocator enum [SRS-05/AC-01] <!-- verify: cargo test -- domain::model::traces::tests, SRS-05:start:end, proof: tests_passed.log -->
- [x] Backward compatibility or migration for existing serialized envelopes [SRS-05/AC-02] <!-- verify: cargo test -- domain::model::traces::tests, SRS-05:start:end, proof: migration_verify.log -->
