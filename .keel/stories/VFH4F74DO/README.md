---
# system-managed
id: VFH4F74DO
status: done
created_at: 2026-03-29T09:25:04
updated_at: 2026-03-29T09:57:19
# authored
title: Add Artifact Envelope Support For Large Turn Payloads
type: feat
operator-signal:
scope: VFH4BXH4F/VFH4CCJ4d
index: 4
started_at: 2026-03-29T09:56:31
submitted_at: 2026-03-29T09:57:18
completed_at: 2026-03-29T09:57:19
---

# Add Artifact Envelope Support For Large Turn Payloads

## Summary

Add a recorder-facing artifact envelope contract so large prompts, model
outputs, tool outputs, and graph traces can move behind logical references
without losing replay-critical metadata.

## Acceptance Criteria

- [x] The trace contract supports artifact envelopes with logical refs plus inline metadata for large turn payloads. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [x] The artifact envelope remains storage-model-neutral inside `paddles` even though the first durable adapter will target embedded `transit-core`. [SRS-NFR-04/AC-02] <!-- verify: manual, SRS-NFR-04:start:end -->
- [x] Large payload handling no longer assumes every rich prompt, tool output, or graph trace must remain inline forever. [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end -->
