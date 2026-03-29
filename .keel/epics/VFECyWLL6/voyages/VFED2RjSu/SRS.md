# Model-Directed Routing Backbone - SRS

## Summary

Epic: VFECyWLL6
Goal: Replace heuristic top-level turn routing with model-selected bounded actions driven by AGENTS-informed interpretation context, while keeping controller safety, observability, and grounded synthesis.

## Scope

### In Scope

- [SCOPE-01] Define the top-level bounded action contract for model-directed routing.
- [SCOPE-02] Assemble interpretation context before first action selection for non-trivial turns.
- [SCOPE-03] Replace heuristic top-level routing with model-selected decisions validated by the controller.
- [SCOPE-04] Preserve safe inspect/tool execution, budgets, event visibility, and grounded synthesis handoff.
- [SCOPE-05] Update foundational docs and examples so the intended routing contract is explicit.

### Out of Scope

- [SCOPE-06] Keel-specific first-class runtime intents or board-only routing branches.
- [SCOPE-07] Mandatory remote models or unbounded agentic execution.
- [SCOPE-08] TUI redesign or unrelated boot/runtime polish not required for routing replacement.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Non-trivial turns must assemble interpretation context from `AGENTS.md`, linked foundational docs, recent turns, and relevant local state before the first bounded action is selected. | SCOPE-02 | FR-01 | manual |
| SRS-02 | The top-level action contract must let the model choose among direct answer/synthesize, search, read, inspect, refine, branch, and stop decisions. | SCOPE-01, SCOPE-03 | FR-03 | manual |
| SRS-03 | The runtime must route non-trivial turns by executing the validated model-selected action path instead of a separate heuristic classifier. | SCOPE-03 | FR-02 | manual |
| SRS-04 | Safe inspect/tool execution must remain controller-validated and bounded even when the model owns first action selection. | SCOPE-03, SCOPE-04 | FR-04 | manual |
| SRS-05 | Recursive planner execution and synthesizer handoff must consume the new top-level action contract without regressing grounded answer behavior. | SCOPE-03, SCOPE-04 | FR-05 | manual |
| SRS-06 | Foundational docs must explain the new contract, the recursive loop, and the current transitional gap honestly. | SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Model-directed routing must remain local-first and fail closed when planner output is invalid or a heavier planner provider is unavailable. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | Turn events must continue to show interpretation, selected actions, fallbacks, and synthesis handoff in the default operator surface. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-02 | manual |
| SRS-NFR-03 | The routing contract must remain general-purpose across repositories and evidence domains rather than Keel-specific. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
