---
# system-managed
id: VFP2EUJD3
status: done
created_at: 2026-03-30T18:07:16
updated_at: 2026-03-30T18:29:26
# authored
title: Define ContextLocator And ContextTier Domain Types
type: feat
operator-signal:
scope: VFOmKssE5/VFOvGdksF
index: 1
started_at: 2026-03-30T18:24:20
completed_at: 2026-03-30T18:29:26
---

# Define ContextLocator And ContextTier Domain Types

## Summary

Define the core `ContextTier` and `ContextLocator` domain types in `paddles`. These types establish a universal addressing scheme across the four context tiers (Inline, Transit, Sift, Filesystem).

## Acceptance Criteria

- [x] ContextTier enum with Inline, Transit, Sift, Filesystem variants [SRS-01/AC-01] <!-- verify: cargo test -- domain::model::traces::tests, SRS-01:start:end, proof: tests_passed.log -->
- [x] ContextLocator enum with Inline { content }, Transit { task_id, record_id }, Sift { index_ref }, Filesystem { path } variants [SRS-01/AC-02] <!-- verify: cargo test -- domain::model::traces::tests, SRS-01:start:end, proof: tests_passed.log -->
- [x] ContextLocator implements Serialize and Deserialize [SRS-02/AC-01] <!-- verify: cargo test -- domain::model::traces::tests, SRS-02:start:end, proof: tests_passed.log -->
- [x] No transit-core types leak into ContextLocator [SRS-NFR-02/AC-01] <!-- verify: cargo test -- domain::model::traces::tests, SRS-NFR-02:start:end, proof: types_audit.log -->
