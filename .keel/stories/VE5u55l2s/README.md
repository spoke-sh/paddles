---
id: VE5u55l2s
title: Implement HF Hub Adapter
type: feat
status: done
created_at: 2026-03-16T21:10:15
started_at: 2026-03-16T21:15:00
updated_at: 2026-03-16T21:09:53
operator-signal: 
scope: VE5ttmBfz/VE5tzQyo5
index: 2
submitted_at: 2026-03-16T21:09:48
completed_at: 2026-03-16T21:09:53
---

# Implement HF Hub Adapter

## Summary

Implement the `ModelRegistry` port using the `hf-hub` crate.

## Acceptance Criteria

- [x] `infrastructure::adapters::hf_hub` implements `ModelRegistry`. [SRS-20/AC-02] <!-- verify: manual, SRS-20:start:end -->
- [x] Model files are downloaded and cached correctly. [SRS-20/AC-03] <!-- verify: manual, SRS-20:start:end -->
