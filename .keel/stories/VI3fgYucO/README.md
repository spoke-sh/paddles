---
# system-managed
id: VI3fgYucO
status: done
created_at: 2026-04-27T21:52:38
updated_at: 2026-04-27T22:03:14
# authored
title: Trust Operator Memory Over Probe Procedure
type: refactor
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 4
started_at: 2026-04-27T21:53:22
completed_at: 2026-04-27T22:03:14
---

# Trust Operator Memory Over Probe Procedure

## Summary

Stop the harness from overriding what the operator wrote in AGENTS.md. Pass the full operator-memory documents alongside the summarized `InterpretationContext` into every action-selection `PlannerRequest`, render them in the planner system prompt as the **primary source of truth**, and reframe the controller-authored "Probe Required Local Tools" procedure as a **validating cache layer** that confirms operator-documented CLIs rather than prescribing generic `command -v <tool>` discovery sweeps. After this lands, "continue executing mission VI2q5DKHe" should produce `keel mission show VI2q5DKHe` as the first action even with a small local planner.

## Acceptance Criteria

- [x] `PlannerRequest` carries `operator_memory: Vec<OperatorMemoryDocument>` populated at construction time from the OperatorMemory port; both production action-selection sites (initial and recursive) and the steering-review path call `.with_operator_memory(...)`. [SRS-01/AC-01] <!-- verify: cargo test --lib planner_request_carries_full_operator_memory_alongside, SRS-01:start:end -->
- [x] Sift and HTTP planner adapters render the full operator-memory documents in the planner prompt, ahead of the summarized interpretation context, with explicit precedence text declaring operator memory as the primary source of truth. [SRS-01/AC-02] <!-- verify: cargo test --lib planner_request_carries_full_operator_memory_alongside, SRS-01:start:end -->
- [x] The controller-authored procedure is renamed to "Validate And Cache Documented Local Tools" with a purpose that explicitly defers to operator memory; sample probe steps point at operator-documented tools (`command -v <tool-named-by-operator-memory>`) rather than prescribing a generic discovery sweep. [SRS-01/AC-03] <!-- verify: cargo test --lib validate_and_cache_procedure_does_not_prescribe, SRS-01:start:end -->
