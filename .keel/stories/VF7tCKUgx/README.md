---
# system-managed
id: VF7tCKUgx
status: needs-human-verification
created_at: 2026-03-27T19:44:45
updated_at: 2026-03-27T23:12:05
# authored
title: Add Local Tool Surface
type: feat
operator-signal:
scope: VF7t633ux/VF7tAvs7B
index: 2
started_at: 2026-03-27T23:11:27
submitted_at: 2026-03-27T23:12:05
---

# Add Local Tool Surface

## Summary

Add the initial local coding tool surface so the Sift-native runtime can inspect,
search, and mutate a workspace without depending on wonopcode tool plumbing.

## Acceptance Criteria

- [x] The runtime exposes immediate local tools for search, file operations, shell commands, and edit/diff operations. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Tool results are recorded as searchable local context for later turns. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
