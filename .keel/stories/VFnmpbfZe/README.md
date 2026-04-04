---
# system-managed
id: VFnmpbfZe
status: done
created_at: 2026-04-03T23:42:54
updated_at: 2026-04-04T00:16:23
# authored
title: Lock Diff Visibility With Projection And Contract Tests
type: feat
operator-signal:
scope: VFnmIbFW2/VFnmfzD3E
index: 4
started_at: 2026-04-04T00:16:03
completed_at: 2026-04-04T00:16:23
---

# Lock Diff Visibility With Projection And Contract Tests

## Summary

Add projection, runtime-contract, and UI coverage for applied-edit artifacts so the new diff visibility surface stays stable and can be used as mission completion evidence.

## Acceptance Criteria

- [x] Automated tests cover the applied-edit artifact shape and its cross-surface rendering contracts [SRS-05/AC-01] <!-- verify: cargo nextest run projects_applied_workspace_edits_into_diff_presentations workspace_editor_boundary_budget_signal_credits_boundary_source workspace_editor_edits_emit_applied_edit_events applied_edit_events_render_diff_lines_in_the_tui_transcript && npm --workspace @paddles/web exec vitest run src/runtime-helpers.test.ts src/runtime-app.test.tsx, SRS-05:start:end, proof: ac-1.log-->
