---
id: VE5qkBKmV
title: Migrate Boot and Domain Logic
type: feat
status: done
created_at: 2026-03-16T20:45:15
started_at: 2026-03-16T20:52:00
updated_at: 2026-03-16T20:56:59
operator-signal: 
scope: VE5qX07aD/VE5qdK37s
index: 2
submitted_at: 2026-03-16T20:56:54
completed_at: 2026-03-16T20:56:59
---

# Migrate Boot and Domain Logic

## Summary

Move `BootContext`, `Constitution`, and `Dogma` to `domain::model`.

## Acceptance Criteria

- [x] `domain::model` contains all boot and validation logic. [SRS-16/AC-01] <!-- verify: manual, SRS-16:start:end -->
- [x] Logic is decoupled from CLI parsing. [SRS-NFR-08/AC-02] <!-- verify: manual, SRS-NFR-08:start:end -->
