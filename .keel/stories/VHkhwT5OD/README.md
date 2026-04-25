---
# system-managed
id: VHkhwT5OD
status: done
created_at: 2026-04-24T16:02:25
updated_at: 2026-04-24T19:01:03
# authored
title: Replay Fork And Compact Session Context
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgNakSc
index: 3
started_at: 2026-04-24T18:58:08
completed_at: 2026-04-24T19:01:03
---

# Replay Fork And Compact Session Context

## Summary

Support replay, fork, and compaction metadata so recursive context can be reconstructed from durable local session state.

## Acceptance Criteria

- [x] Session records can reconstruct model-visible context through replay metadata. [SRS-03/AC-01] <!-- verify: cargo test session_replay -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Compaction summaries link back to source turns and evidence. [SRS-03/AC-02] <!-- verify: cargo test session_compaction_lineage -- --nocapture, SRS-03:start:end, proof: ac-2.log-->
