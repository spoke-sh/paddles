---
# system-managed
id: VHUS75D3e
status: done
created_at: 2026-04-21T21:19:11
updated_at: 2026-04-21T21:40:01
# authored
title: Persist Typed Authored Responses In Completion Records
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 1
started_at: 2026-04-21T21:23:03
completed_at: 2026-04-21T21:40:01
---

# Persist Typed Authored Responses In Completion Records

## Summary

Update the completion/checkpoint path so typed `AuthoredResponse` and
`RenderDocument` data survive durable recording, then replay assistant rows from
that stored structure instead of reparsing flattened prose.

## Acceptance Criteria

- [x] Completion checkpoints persist typed authored response data sufficient to replay render blocks without reconstructing structure from plain text. [SRS-01/AC-01] <!-- verify: cargo test structured_turn_trace_records_lineage_edges_for_model_calls_and_outputs -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Transcript replay preserves response mode, citations, and grounding metadata by reading the persisted structured response path directly. [SRS-02/AC-02] <!-- verify: cargo test domain::model::transcript::tests -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
