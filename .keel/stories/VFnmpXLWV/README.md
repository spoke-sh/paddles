---
# system-managed
id: VFnmpXLWV
status: in-progress
created_at: 2026-04-03T23:42:53
updated_at: 2026-04-03T23:49:05
# authored
title: Emit Structured Applied Edit Artifacts From The Workspace Editor
type: feat
operator-signal:
scope: VFnmIbFW2/VFnmfzD3E
index: 1
started_at: 2026-04-03T23:49:05
---

# Emit Structured Applied Edit Artifacts From The Workspace Editor

## Summary

Extend the workspace editor result path so successful edit actions emit a structured applied-edit artifact with file identity and diff content instead of only a prose tool summary.

## Acceptance Criteria

- [ ] Successful `apply_patch`, `replace_in_file`, and `write_file` actions return structured applied-edit data that can feed runtime events and projections [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] Successful workspace editor edits emit a shared applied-edit runtime artifact instead of only a generic tool summary [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
