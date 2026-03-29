# Model-Directed Turn Routing - Product Requirements

## Problem Statement

Top-level turn routing still depends on controller heuristics before the model can select a bounded action from interpretation context, which prevents AGENTS-driven interpretation from steering recursive resource use and leaves weak planner fallbacks in place.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Replace heuristic top-level routing with model-selected bounded next actions for non-trivial turns. | Non-trivial turns choose their first action from a constrained model schema instead of controller string heuristics | Verified runtime traces and tests |
| GOAL-02 | Make `AGENTS.md` and linked foundational docs influence first action selection, not just later synthesis. | Planner/action traces show interpretation context is assembled before next-action choice and materially shapes resource selection | Verified trace and prompt proofs |
| GOAL-03 | Preserve controller safety and observability while moving routing ownership to the model. | The controller still validates actions, enforces budgets, and renders action traces while no longer heuristically deciding the route | Verified design, tests, and TUI/plain proofs |
| GOAL-04 | Keep the backbone architecture and foundational docs explicit about this shift. | README and companion docs explain model-directed action selection, recursive routing, and current transitional gaps clearly | Verified docs and diagram proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using `paddles` interactively with small local models. | Better turn routing because the model can choose the next resource action from interpretation context instead of being trapped behind heuristics. |
| Runtime Maintainer | An engineer evolving routing, planning, and synthesis boundaries. | A durable contract where the model owns bounded action selection and the controller owns safety. |
| Model Router | The person deciding which planner/synthesizer models to run locally. | A clean workload boundary that can route planner selection, gatherers, and synthesis independently. |

## Scope

### In Scope

- [SCOPE-01] Define a constrained first-action schema that can express direct answer/synthesize, search, read, inspect, refine, branch, and stop decisions.
- [SCOPE-02] Move first non-trivial action selection behind interpretation-context-aware model output instead of controller string heuristics.
- [SCOPE-03] Feed `AGENTS.md`, linked foundational docs, recent turns, and loop state into first action selection.
- [SCOPE-04] Preserve controller validation, safe inspect/tool execution, budgets, and observable turn events.
- [SCOPE-05] Update foundational docs so the backbone architecture clearly describes model-directed top-level routing and the remaining transitional gaps.

### Out of Scope

- [SCOPE-06] Hardcoded Keel-specific runtime intents or board-only route selectors.
- [SCOPE-07] Mandatory remote models or making `context-1` the default answer runtime.
- [SCOPE-08] Replacing the TUI shell or unrelated boot/pacemaker behavior.
- [SCOPE-09] Unbounded autonomous execution outside validated action and budget contracts.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must assemble interpretation context from `AGENTS.md`, linked foundational docs, recent turns, and relevant local state before first bounded action selection for non-trivial turns. | GOAL-01, GOAL-02 | must | The model cannot route effectively if interpretation context arrives only after the controller has already chosen the path. |
| FR-02 | The first non-trivial routing decision must be selected by a planner-capable model from a constrained action schema rather than controller string heuristics. | GOAL-01, GOAL-03 | must | This is the core architectural change. |
| FR-03 | The action schema must be able to express direct answer/synthesize, search, read, inspect, refine, branch, and stop decisions. | GOAL-01, GOAL-03 | must | The model needs a bounded but complete vocabulary for top-level routing. |
| FR-04 | Controller code must validate chosen actions, enforce budgets and safe-command allowlists, and fail closed when model output is invalid. | GOAL-03 | must | Routing can be model-owned without giving up safety. |
| FR-05 | The recursive planner loop and synthesizer handoff must consume the model-selected action path instead of a separate heuristic classifier. | GOAL-01, GOAL-03 | must | The new contract should replace the old gate rather than layering on top of it. |
| FR-06 | Foundational docs must explain the model-directed top-level routing contract, recursive loop, and current transitional state. | GOAL-04 | must | Operators need the mental model in the docs, not just in code. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Model-directed routing must remain local-first by default and degrade safely when a planner model returns invalid output or a heavier provider is unavailable. | GOAL-01, GOAL-03 | must | Preserves the core runtime constraint. |
| NFR-02 | The controller must keep turn events, fallback reasons, and final handoff state observable in the default user surface. | GOAL-03, GOAL-04 | must | Operators need to see what action the model chose and why. |
| NFR-03 | The contract must remain general-purpose across repositories and evidence domains rather than overfitting to Keel. | GOAL-01, GOAL-03 | should | The harness should lift small models through context, not product-specific special cases. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| First-action contract | Unit tests, routing transcripts, and manual review | Story evidence showing model-selected bounded actions replace heuristic top-level routing |
| Interpretation influence | Prompt/trace proofs and targeted regression tests | Story evidence showing `AGENTS.md` and linked docs shape first action choice |
| Safety and execution | Tests and CLI/TUI proofs | Story evidence showing controller validation, budgets, and safe inspect/tool execution remain intact |
| Docs and diagrams | Doc review plus rendered examples | Updated foundational docs and proof artifacts |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Small local models can choose bounded actions more reliably than they can answer difficult workspace questions in one shot. | The new contract may add complexity without enough quality gain. | Validate with before/after transcript proofs. |
| Interpretation context from `AGENTS.md` and linked docs is useful enough to shape first action selection. | The model may still need stronger planner-specific guidance or a different planner model. | Validate through routing and trace evidence. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The first-action schema may need a `synthesize` or `answer` terminal action rather than overloading `stop`. | Runtime maintainer | Open |
| Local planner models may still emit invalid action JSON often enough that fallback behavior becomes a quality bottleneck. | Runtime maintainer | Open |
| Removing heuristic routing too abruptly could regress obvious low-latency turns unless the new contract handles trivial replies cleanly. | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Non-trivial turns choose their first bounded action through a constrained model contract informed by interpretation context.
- [ ] `AGENTS.md` and linked foundational docs influence first action selection before recursive resource use begins.
- [ ] Controller safety and observability remain intact while heuristic top-level routing is retired.
- [ ] Foundational docs explain the model-directed routing backbone and current transitional state clearly.
<!-- END SUCCESS_CRITERIA -->
