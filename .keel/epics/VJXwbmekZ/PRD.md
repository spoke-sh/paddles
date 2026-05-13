# Unified Recursive Agent Action Contract - Product Requirements

## Problem Statement

Paddles currently has a pre-loop `InitialAction` routing decision and
planner-phase nomenclature that imply planning is separate from the recursive
agent loop. That is the wrong mental model for the product: the model's
reasoning is the planning, and it should be expressed as bounded action
selection inside one recursive agent loop, including the first action.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Replace the `InitialAction` / recursive-action split with one recursive agent action decision contract. | Domain and schema tests prove first and later decisions share one action vocabulary. | 100% passing contract tests |
| GOAL-02 | Preserve direct answers, edit obligations, candidate-file hints, capability-manifest gating, and provider behavior while moving the first model choice into the loop. | Existing behavior regressions are covered by focused tests and the full quality hook. | No regressions |
| GOAL-03 | Remove architecture language that treats planning as a separate phase outside the agent loop. | Foundational docs and prompt tests describe model reasoning as bounded recursive action selection. | All owning docs updated |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Paddles operator | A developer using Paddles to inspect, edit, verify, or answer questions about a repository. | A coherent recursive agent loop that does not hide a pre-loop routing phase. |
| Paddles maintainer | A contributor evolving action schemas, providers, or loop behavior. | One contract to test and extend without duplicating first-action and recursive-action semantics. |

## Scope

### In Scope

- [SCOPE-01] Domain action contract refactor from first-action/recursive-action split to one agent action decision model.
- [SCOPE-02] Shared schema renderer migration so first and later decisions use one canonical action entry set with variant-specific availability.
- [SCOPE-03] Runtime migration so the first model decision is step zero of the recursive agent loop.
- [SCOPE-04] Preservation of direct-answer, stop, known-edit, commit, review, repository-grounding, and fail-closed behavior.
- [SCOPE-05] Prompt, adapter, test, and foundational documentation vocabulary updates.

### Out of Scope

- [SCOPE-06] New model providers or transport protocols.
- [SCOPE-07] New external capability fabrics.
- [SCOPE-08] UI redesign unrelated to loop/action vocabulary.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must expose one recursive agent action decision contract for first and later model choices. | GOAL-01 | must | Removes the architectural split that made the first action look outside the loop. |
| FR-02 | The shared schema renderer must render one action vocabulary with variant-specific availability rather than adapter-local first/recursive action universes. | GOAL-01 | must | Keeps prompts, tests, and Rust contracts aligned. |
| FR-03 | The runtime must execute the first model action inside the recursive agent loop as step zero, with `answer` and `stop` as terminal loop actions. | GOAL-02 | must | Makes the whole agent loop recursive without losing direct-answer behavior. |
| FR-04 | Edit obligations, candidate-file hints, known-edit bootstrap, review bootstrap, commit bootstrap, and repository-grounding guardrails must survive the migration. | GOAL-02 | must | These are existing safety and convergence contracts. |
| FR-05 | Sift/local and HTTP/remote planner lanes must parse and receive the same unified action schema. | GOAL-01, GOAL-02 | must | Prevents lane drift. |
| FR-06 | Foundational documentation must describe planning as model reasoning through bounded recursive agent actions, not as a separate architecture phase. | GOAL-03 | must | Aligns operator-facing explanation with runtime reality. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve local-first execution and avoid new network dependencies. | GOAL-02 | must | The refactor is architectural, not a provider expansion. |
| NFR-02 | Preserve observability of loop steps, terminal actions, steering reviews, and evidence. | GOAL-02 | must | Operators must still see why the agent acted or stopped. |
| NFR-03 | Avoid compatibility aliases that become permanent hidden contracts. | GOAL-01, GOAL-03 | should | Transitional names are acceptable only when removed or explicitly bounded by tests. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Domain contract | Focused Rust unit tests for the unified action enum and schema parity | Story evidence logs |
| Runtime migration | Application tests for direct answer, first workspace action, edit obligation, and fail-closed paths | Story evidence logs |
| Adapter parity | Mocked Sift/HTTP prompt and parser tests | Story evidence logs |
| Documentation | Executable `rg` checks plus doc review | Story evidence logs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The `InitialAction` split is not required for provider transports. | If wrong, adapters may need a temporary compatibility wrapper. | Adapter parser tests in the contract voyage. |
| Direct answer can be represented as a terminal action in the same loop. | If wrong, the loop abstraction needs a small terminal-result wrapper. | Runtime migration tests. |
| Edit metadata can live on the decision envelope instead of a special initial decision type. | If wrong, edit obligation handling may need a dedicated loop context field. | Known-edit and commit steering tests. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How aggressively should public Rust names be renamed in one mission? | Mission implementer | Track in contract story; prefer tests over cosmetic churn. |
| Do downstream users depend on `InitialAction` names? | Mission implementer | Search and preserve temporary aliases only if needed. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The first model action and later model actions use one recursive agent action contract.
- [ ] Direct answer and stop are terminal actions inside the recursive loop, not a pre-loop bypass.
- [ ] Existing edit and evidence-gathering behavior remains covered by tests.
- [ ] Foundational docs explain that model reasoning is the planning inside the recursive agent loop.
<!-- END SUCCESS_CRITERIA -->
