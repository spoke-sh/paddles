---
# system-managed
id: VFH4Cw86b
status: done
created_at: 2026-03-29T09:24:56
updated_at: 2026-03-29T09:57:18
# authored
title: Define Paddles Trace Contract And Lineage Model
type: feat
operator-signal:
scope: VFH4BXH4F/VFH4CCJ4d
index: 1
started_at: 2026-03-29T09:45:33
submitted_at: 2026-03-29T09:57:17
completed_at: 2026-03-29T09:57:18
---

# Define Paddles Trace Contract And Lineage Model

## Summary

Define the stable `paddles` trace entities and lineage identifiers that later
recorders will persist, keeping the contract aligned with `transit` AI trace
semantics without exposing raw `transit` types across the domain boundary.

## Acceptance Criteria

- [x] The domain defines typed trace entities for task roots, planner branches, tool request/result pairs, selection artifacts, and completion checkpoints. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] The trace entities use stable machine-readable identifiers and lineage references rather than UI-only labels. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [x] The contract remains `paddles`-owned and does not leak raw `transit` types across the domain boundary. [SRS-NFR-04/AC-03] <!-- verify: manual, SRS-NFR-04:start:end -->
