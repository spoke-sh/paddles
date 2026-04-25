---
# system-managed
id: VHkhoGo0Y
status: done
created_at: 2026-04-24T16:01:54
updated_at: 2026-04-24T18:16:40
# authored
title: Preserve File Format And Lock Workspace Edits
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgLtij9
index: 1
started_at: 2026-04-24T18:11:13
completed_at: 2026-04-24T18:16:40
---

# Preserve File Format And Lock Workspace Edits

## Summary

Harden workspace edits by preserving file format details and serializing writes through per-file locks.

## Acceptance Criteria

- [x] Write, replace, and patch operations preserve line endings and BOM markers where present. [SRS-01/AC-01] <!-- verify: cargo test workspace_edit_preserves_format -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Concurrent edit attempts on the same file are serialized or rejected with clear evidence. [SRS-02/AC-01] <!-- verify: cargo test workspace_edit_locking -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
