---
# system-managed
id: VHXK93dcv
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T10:58:15
# authored
title: Use Deliberation Signals In Recursive Branch Refine And Stop Decisions
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJiqMD1
index: 2
started_at: 2026-04-22T10:42:23
completed_at: 2026-04-22T10:58:15
---

# Use Deliberation Signals In Recursive Branch Refine And Stop Decisions

## Summary

Wire normalized deliberation signals into the recursive harness so the planner
can make better continue, branch, refine, retry, and stop decisions without
matching on provider-native payloads.

## Acceptance Criteria

- [x] The recursive harness uses normalized deliberation signals to improve branch, refine, retry, and stop decisions. [SRS-03/AC-01] <!-- verify: cargo test continuation_signals_ -- --nocapture && cargo test explicit_none_ -- --nocapture && cargo test action_bias_ -- --nocapture && cargo test premise_challenge_ -- --nocapture && cargo test execution_pressure_prefers_resolved_targets_over_repeated_search -- --nocapture && cargo test parse_planner_action_separates_direct_answer_from_rationale -- --nocapture && cargo test provider_turn_request_and_response_keep_deliberation_state_separate_from_content -- --nocapture, SRS-03:start:end -->
