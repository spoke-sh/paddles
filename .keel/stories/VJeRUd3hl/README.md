---
# system-managed
id: VJeRUd3hl
status: done
created_at: 2026-05-14T19:17:12
updated_at: 2026-05-14T19:41:23
# authored
title: Route First Action Through Agent Loop
type: feat
operator-signal:
scope: VJeQx1O20/VJeRAOoHj
index: 1
started_at: 2026-05-14T19:22:08
completed_at: 2026-05-14T19:41:23
---

# Route First Action Through Agent Loop

## Summary

Route normal turn execution into the recursive agent loop before any model-selected action is accepted. The first loop iteration should own direct answers, stops, and workspace actions instead of receiving a preselected initial decision from `turn.rs`.

## Acceptance Criteria

- [x] Turn orchestration enters `execute_agent_loop` before model-selected action execution for normal turns. [SRS-01/AC-01] <!-- verify: cargo test route_first_action_through_agent_loop -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The first loop iteration can return direct-answer, stop, and workspace-action outcomes. [SRS-02/AC-02] <!-- verify: cargo test first_agent_loop_iteration_handles_initial_outcomes -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] Direct-answer and final-rendering behavior remains available through `AgentLoopOutcome`. [SRS-03/AC-03] <!-- verify: cargo test agent_loop_outcome_preserves_direct_answer_rendering -- --nocapture, SRS-03:start:end, proof: ac-3.log-->
- [x] First-action trace evidence is emitted by the loop rather than by a pre-loop router. [SRS-NFR-01/AC-03] <!-- verify: cargo test first_action_trace_is_loop_owned -- --nocapture, SRS-NFR-01:start:end, proof: ac-4.log-->
