---
# system-managed
id: VFJ5wSYep
status: done
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T18:27:30
# authored
title: Remove Legacy Direct Routing Heuristics From Remaining Turn Paths
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 1
started_at: 2026-03-29T17:57:39
completed_at: 2026-03-29T18:27:30
---

# Remove Legacy Direct Routing Heuristics From Remaining Turn Paths

## Summary

Remove the remaining legacy direct-path reasoning helpers such as string-based
casual/tool/follow-up inference from the primary harness path so bounded
model-selected actions remain the source of reasoning even when the turn does
not immediately recurse through the planner loop.

## Acceptance Criteria

- [x] Remaining legacy direct-path routing/tool-inference heuristics are removed or demoted so they no longer decide the primary turn path. [SRS-01/AC-01] <!-- verify: cargo test -q deterministic_action_turns_require_model_selected_tool_calls && cargo test -q respond_starts_a_fresh_conversation_each_turn, SRS-01:start:end, proof: ac-1.log-->
- [x] Any retained direct-path fallback stays clearly fail-closed and controller-owned rather than silently reasoning for the model. [SRS-NFR-01/AC-02] <!-- verify: cargo test -q invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid && cargo test -q invalid_planner_replies_fail_closed_after_redecision_is_still_invalid, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Transcript/tests prove the model-selected path still handles conversational versus workspace turns without the old string classifier. [SRS-01/AC-03] <!-- verify: cargo test -q process_prompt_assembles_interpretation_before_model_selected_initial_action, SRS-01:start:end, proof: ac-3.log-->
