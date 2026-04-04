---
# system-managed
id: VFnmpaHZS
status: done
created_at: 2026-04-03T23:42:54
updated_at: 2026-04-04T00:15:45
# authored
title: Render Applied Edit Diffs In The TUI Transcript Stream
type: feat
operator-signal:
scope: VFnmIbFW2/VFnmfzD3E
index: 3
started_at: 2026-04-04T00:15:37
completed_at: 2026-04-04T00:15:45
---

# Render Applied Edit Diffs In The TUI Transcript Stream

## Summary

Render the same applied-edit artifact semantics in the TUI transcript stream so interactive terminal turns make workspace editor activity visually obvious.

## Acceptance Criteria

- [x] The TUI transcript stream renders applied-edit artifacts with the same semantic content as the web surface [SRS-04/AC-01] <!-- verify: cargo nextest run applied_edit_events_render_diff_lines_in_the_tui_transcript, SRS-04:start:end, proof: ac-1.log-->
