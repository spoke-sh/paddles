---
# system-managed
id: VJeRaXliR
status: backlog
created_at: 2026-05-14T19:17:35
updated_at: 2026-05-14T19:19:31
# authored
title: Add Simple Evidence Question Regression
type: chore
operator-signal:
scope: VJeQx1O20/VJeRAR1IS
index: 3
---

# Add Simple Evidence Question Regression

## Summary

Add a regression for the failure mode that prompted the mission: a simple operator-contract question should use live evidence once and answer from the loop state, not spin in repeated pre-loop reads or rely on a hard repeat guard.

## Acceptance Criteria

- [ ] A focused test reproduces a simple evidence-backed operator-contract question entering the loop and answering from gathered evidence. [SRS-04/AC-01] <!-- verify: cargo test simple_evidence_question_answers_from_loop_state -- --nocapture, SRS-04:start:end -->
- [ ] The regression proves observations are fed back into action-selection reasoning, not blocked by a hard duplicate-read guard. [SRS-04/AC-02] <!-- verify: cargo test repeated_read_failure_uses_observation_feedback -- --nocapture, SRS-04:start:end -->
- [ ] Full library tests pass after the regression is added. [SRS-NFR-02/AC-03] <!-- verify: cargo test --lib, SRS-NFR-02:start:end -->
