---
# system-managed
id: VJXwmjOZl
status: backlog
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T16:40:08
# authored
title: Execute First Model Decision Inside Recursive Agent Loop
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlE718
index: 1
---

# Execute First Model Decision Inside Recursive Agent Loop

## Summary

Move the first model-selected action into the recursive agent loop as step zero
instead of treating it as pre-loop routing.

## Acceptance Criteria

- [ ] Direct `answer` and `stop` decisions terminate the recursive agent loop as terminal step-zero actions and preserve existing user-visible responses. [SRS-02/AC-01] <!-- verify: cargo test first_agent_action_terminal_answer_and_stop --lib, SRS-02:start:end -->
- [ ] First workspace, refine, and branch decisions enter the same execution path and loop-state recording used by later recursive decisions. [SRS-03/AC-02] <!-- verify: cargo test first_agent_action_executes_as_loop_step_zero --lib, SRS-03:start:end -->
- [ ] The turn orchestration no longer needs a separate `select_initial_action` routing branch to decide whether the recursive loop exists. [SRS-01/AC-03] <!-- verify: cargo test first_action_does_not_bypass_agent_loop --lib, SRS-01:start:end -->
