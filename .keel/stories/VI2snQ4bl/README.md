---
# system-managed
id: VI2snQ4bl
status: backlog
created_at: 2026-04-27T18:38:26
updated_at: 2026-04-27T18:46:11
# authored
title: Route Controller Signal Summary To Sibling Field
type: refactor
operator-signal:
scope: VI2sGaOrg/VI2sdFRI7
index: 1
---

# Route Controller Signal Summary To Sibling Field

## Summary

Land the rationale-trust change end-to-end: stop assigning to `decision.rationale` in `src/application/recursive_control.rs`, add a sibling field for the controller-derived signal summary, and update `TurnEvent::PlannerActionSelected` plus forensics / manifold projections so the model's own rationale text flows through unchanged while controller annotations remain visible alongside it.

## Acceptance Criteria

- [ ] `recursive_control.rs` no longer assigns to `decision.rationale`; the planner model's rationale text is preserved verbatim from `RecursivePlannerDecision` through to the trace. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] A sibling field carries the controller-derived signal summary (output of the prior `compile_recursive_paddles_rationale`) and is emitted on `TurnEvent::PlannerActionSelected` alongside the model's rationale. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Forensics and manifold projections render both fields distinctly so an operator inspecting a turn can tell the model's rationale apart from controller annotations. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
