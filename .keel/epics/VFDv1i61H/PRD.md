# Recursive Planner Loop And Memory-Driven Routing - Product Requirements

## Problem Statement

Paddles still relies on static controller heuristics and one-shot evidence gathering, so small local models do not get enough recursive context refinement to answer difficult workspace questions reliably.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Let a planner model refine its understanding of a turn by reading operator memory and recursively using local resources before the answer is synthesized. | Difficult workspace questions trigger a bounded multi-step plan instead of a single heuristic classification and gather pass | Verified runtime and transcript proofs |
| GOAL-02 | Make operator memory part of turn interpretation, not just answer prompting. | Planner decisions demonstrably reflect `AGENTS.md` and linked foundational docs in first-pass interpretation | Verified prompt/trace and routing proofs |
| GOAL-03 | Keep routing model-specific and extensible so planner, gatherer, and synthesizer roles can use different models. | The runtime can choose a planner-capable model separately from the final synthesizer without collapsing the architecture | Verified design and integration proofs |
| GOAL-04 | Document the recursive harness as the backbone architecture of the paddles mech suit. | README and companion architecture docs explain the recursive loop, model routing, and current implementation status clearly | Verified docs and diagram proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using `paddles` interactively to reason about a workspace with small local models. | Better answers through recursive evidence refinement instead of brittle one-shot prompting. |
| Runtime Maintainer | An engineer evolving controller/model boundaries in `paddles`. | A reusable planner/gatherer/synthesizer architecture that does not hardcode domain-specific turn types. |
| Model Router | The person deciding which local or specialized model should own planning versus final synthesis. | Clear contracts for swapping planner models without rewriting the whole runtime. |

## Scope

### In Scope

- [SCOPE-01] Elevate `AGENTS.md` and linked foundational docs into first-pass interpretation context for non-trivial turns.
- [SCOPE-02] Replace static turn-type selection as the primary reasoning mechanism with a bounded planner action-selection loop.
- [SCOPE-03] Add a recursive search/refine execution loop where a planner can gather, read, search again, branch, or stop within explicit budgets.
- [SCOPE-04] Separate planner and synthesizer roles so the final answer comes from evidence produced by the recursive loop.
- [SCOPE-05] Rewrite foundational docs so the recursive harness is the documented backbone architecture while preserving honesty about current implementation status.

### Out of Scope

- [SCOPE-06] Hardcoding Keel board semantics as first-class runtime intents or special-case controllers.
- [SCOPE-07] Requiring a remote-only model or making `context-1` mandatory for ordinary local operation.
- [SCOPE-08] Replacing the existing interactive TUI work or changing the boot/pacemaker workflow.
- [SCOPE-09] Solving every planner-quality issue in one mission without bounded contracts or observability.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must assemble interpretation context from operator memory plus recent turn state before planner action selection for non-trivial turns. | GOAL-01, GOAL-02 | must | The model cannot reason from AGENTS guidance if the controller commits to a path first. |
| FR-02 | The planner path must support a bounded recursive loop of resource actions such as search, read, inspect tool output, refine query, branch, and stop. | GOAL-01 | must | Recursive refinement is the core behavior change. |
| FR-03 | Planner outputs must be validated through a constrained action contract instead of trusted as arbitrary prose. | GOAL-01, GOAL-03 | must | Small local models need a safe harness boundary. |
| FR-04 | The final answer path must remain a distinct synthesizer step that consumes planner trace plus evidence rather than free-form planner prose. | GOAL-01, GOAL-03 | must | Keeps planner and answer quality independently tunable. |
| FR-05 | Model routing must allow planner-capable and synthesizer-capable models to be configured separately according to intent and runtime budget. | GOAL-03 | must | This is the point of routing by workload rather than one-model-for-all. |
| FR-06 | Foundational docs must explain the recursive harness backbone with diagrams covering interpretation context, recursive loop behavior, and model routing. | GOAL-04 | must | Operators need a durable mental model before deeper implementation lands. |
| FR-07 | The architecture must avoid domain-specific first-class board intents; Keel and other repo artifacts should enter the planner through context and resources. | GOAL-01, GOAL-03 | should | Preserves generality and avoids brittle product logic. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The planner loop must remain local-first by default and degrade safely when a heavier or external planner model is unavailable. | GOAL-01, GOAL-03 | must | Preserves the core runtime constraint. |
| NFR-02 | Recursive planning must be bounded by explicit depth, budget, and action validation rules. | GOAL-01 | must | Prevents runaway loops and incoherent local planning. |
| NFR-03 | Planner traces, chosen actions, stop reasons, and synthesis handoff data must remain observable in the default operator surface or evidence artifacts. | GOAL-01, GOAL-04 | must | Operators need to trust and debug the recursive loop. |
| NFR-04 | The contract must remain general-purpose across repositories and evidence domains, including but not limited to Keel board artifacts. | GOAL-01, GOAL-03 | should | Avoids overfitting the harness to one project tool. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Planner interpretation | Routing tests, prompt/trace proofs, and code review | Story evidence showing operator memory participates in first-pass planning |
| Recursive resource loop | Integration tests and transcript proofs | Story evidence showing bounded search/refine execution and stop conditions |
| Planner/synth role split | Design review plus runtime proofs | Story evidence showing evidence-first synthesizer handoff |
| Foundational docs | Doc review plus rendered diagram proofs | Updated README and architecture guidance artifacts |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A planner model can materially improve small-model performance by recursively shaping context before final synthesis. | The architecture may add complexity without enough quality gain. | Validate during planner-loop proofs. |
| Operator memory provides enough useful priors to improve first-pass interpretation. | AGENTS integration may not change planning quality materially. | Validate with targeted transcript comparisons. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| A free-form planner loop may still drift unless the action schema is constrained tightly. | Runtime maintainer | Open |
| The best planner model may differ from the best synthesizer model and could change over time. | Runtime maintainer | Open |
| Rewriting docs around the recursive backbone before full implementation could confuse operators unless the current-status section is explicit. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Non-trivial workspace questions enter a bounded planner loop informed by operator memory instead of one-shot heuristic routing alone.
- [ ] Planner and synthesizer roles are treated as separate configurable model lanes.
- [ ] Foundational docs explain the recursive harness backbone and clearly distinguish target architecture from current implementation status.
<!-- END SUCCESS_CRITERIA -->
