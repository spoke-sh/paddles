---
id: VE5qkBKmV
title: Migrate Boot and Domain Logic
type: feat
status: backlog
created_at: 2026-03-16T20:45:15
updated_at: 2026-03-16T20:53:51
operator-signal: 
scope: VE5qX07aD/VE5qdK37s
index: 2
---

# Migrate Boot and Domain Logic

## Summary

Move `BootContext`, `Constitution`, and `Dogma` to `domain::model`.

## Acceptance Criteria

- [ ] `domain::model` contains all boot and validation logic. [SRS-16/AC-01] <!-- verify: manual, SRS-16:start:end -->
- [ ] Logic is decoupled from CLI parsing. [SRS-NFR-08/AC-02] <!-- verify: manual, SRS-NFR-08:start:end -->
