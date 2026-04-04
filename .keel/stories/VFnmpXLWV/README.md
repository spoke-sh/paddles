---
# system-managed
id: VFnmpXLWV
status: done
created_at: 2026-04-03T23:42:53
updated_at: 2026-04-04T00:14:07
# authored
title: Emit Structured Applied Edit Artifacts From The Workspace Editor
type: feat
operator-signal:
scope: VFnmIbFW2/VFnmfzD3E
index: 1
started_at: 2026-04-03T23:49:05
completed_at: 2026-04-04T00:14:07
---

# Emit Structured Applied Edit Artifacts From The Workspace Editor

## Summary

Extend the workspace editor result path so successful edit actions emit a structured applied-edit artifact with file identity and diff content instead of only a prose tool summary.

## Acceptance Criteria

- [x] Successful `apply_patch`, `replace_in_file`, and `write_file` actions return structured applied-edit data that can feed runtime events and projections [SRS-01/AC-01] <!-- verify: cargo nextest run edit_actions_return_structured_applied_edit_artifacts, SRS-01:start:end, proof: ac-1.log-->
- [x] Successful workspace editor edits emit a shared applied-edit runtime artifact instead of only a generic tool summary [SRS-02/AC-02] <!-- verify: cargo nextest run workspace_editor_edits_emit_applied_edit_events projects_applied_workspace_edits_into_diff_presentations, SRS-02:start:end, proof: ac-2.log-->
