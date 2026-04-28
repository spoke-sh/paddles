---
# system-managed
id: VI2snS9cl
status: backlog
created_at: 2026-04-27T18:38:26
updated_at: 2026-04-27T18:46:11
# authored
title: Stream Planner Shell And Inspect Output
type: feat
operator-signal:
scope: VI2sHovAf/VI2seFPac
index: 1
---

# Stream Planner Shell And Inspect Output

## Summary

Replace the buffered `process::Output` capture in `src/application/planner_action_execution.rs` and `src/infrastructure/terminal.rs` with streamed stdout/stderr that fans out to the operator `TurnEventSink` and the planner request as bytes arrive. Drop the `trim_for_planner(&rendered, 1_200)` cap; the planner-bound copy may be capped at 32k+ with head+tail truncation, but operator-visible output and the trace recorder receive the full stream.

## Acceptance Criteria

- [ ] Shell and inspect tool output streams to the TUI / operator sink as bytes arrive instead of being delivered as a single end-of-command payload. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The 1,200-character `trim_for_planner` cap on shell and inspect summaries is removed; any remaining planner-bound budget is at least 32k characters and uses head+tail truncation with an explicit truncation marker. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] The trace recorder persists the full untrimmed stdout/stderr for shell and inspect invocations regardless of what the planner-bound summary contains. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
