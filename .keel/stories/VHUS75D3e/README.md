---
# system-managed
id: VHUS75D3e
status: in-progress
created_at: 2026-04-21T21:19:11
updated_at: 2026-04-21T21:23:03
# authored
title: Persist Typed Authored Responses In Completion Records
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 1
started_at: 2026-04-21T21:23:03
---

# Persist Typed Authored Responses In Completion Records

## Summary

Update the completion/checkpoint path so typed `AuthoredResponse` and
`RenderDocument` data survive durable recording, then replay assistant rows from
that stored structure instead of reparsing flattened prose.

## Acceptance Criteria

- [ ] Completion checkpoints persist typed authored response data sufficient to replay render blocks without reconstructing structure from plain text. [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] Transcript replay preserves response mode, citations, and grounding metadata by reading the persisted structured response path directly. [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
