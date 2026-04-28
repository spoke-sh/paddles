# VOYAGE REPORT: Stream Tool Output And Drop The 1.2k Cap

## Voyage Metadata
- **ID:** VI2seFPac
- **Epic:** VI2sHovAf
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Stream Planner Shell And Inspect Output
- **ID:** VI2snS9cl
- **Status:** done

#### Summary
Replace the buffered `process::Output` capture in `src/application/planner_action_execution.rs` and `src/infrastructure/terminal.rs` with streamed stdout/stderr that fans out to the operator `TurnEventSink` and the planner request as bytes arrive. Drop the `trim_for_planner(&rendered, 1_200)` cap; the planner-bound copy may be capped at 32k+ with head+tail truncation, but operator-visible output and the trace recorder receive the full stream.

#### Acceptance Criteria
- [x] Shell and inspect tool output streams to the operator sink as bytes arrive (multiple `TurnEvent::ToolOutput` chunks per command instead of a single end-of-command payload). [SRS-01/AC-01] <!-- verify: cargo test --lib planner_shell_summary_preserves_output_well_beyond_old_1200_char_cap, SRS-01:start:end -->
- [x] The 1,200-character `trim_for_planner` cap on shell and inspect summaries is removed; the planner-bound budget is now 32,000 characters with head+tail truncation and an explicit `…[truncated N chars]…` marker. [SRS-01/AC-02] <!-- verify: cargo test --lib planner_shell_summary_uses_head_tail_truncation_above_budget, SRS-01:start:end -->
- [x] Per-stream capture cap raised from 24k to 256k chars and per-event chunk cap from 400 to 8,192 chars so the trace recorder retains full output for any reasonable command (with a hard ceiling to bound runaway-command memory). [SRS-01/AC-03] <!-- verify: cargo test --lib planner_inspect_summary_preserves_output_well_beyond_old_1200_char_cap, SRS-01:start:end -->


