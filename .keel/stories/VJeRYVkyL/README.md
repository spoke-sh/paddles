---
# system-managed
id: VJeRYVkyL
status: backlog
created_at: 2026-05-14T19:17:27
updated_at: 2026-05-14T19:19:31
# authored
title: Move Turn Obligations Into Loop State
type: refactor
operator-signal:
scope: VJeQx1O20/VJeRAPzHh
index: 2
---

# Move Turn Obligations Into Loop State

## Summary

Move edit, commit, review, and grounding obligations into loop state or instruction-frame data so they guide model action selection inside the loop rather than forcing a first action in `turn.rs`.

## Acceptance Criteria

- [ ] Edit and commit obligations are attached to the first loop request as instruction-frame or loop-state data. [SRS-03/AC-01] <!-- verify: cargo test turn_obligations_are_loop_inputs -- --nocapture, SRS-03:start:end -->
- [ ] Grounding and review pressure no longer force a pre-loop bootstrap action. [SRS-04/AC-02] <!-- verify: cargo test grounding_and_review_pressure_are_loop_context -- --nocapture, SRS-04:start:end -->
- [ ] Read-only, review, and execution mutation behavior remains enforced after model selection and before execution. [SRS-02/AC-03] <!-- verify: cargo test turn_contract_blocks_mutation_inside_loop -- --nocapture, SRS-02:start:end -->
