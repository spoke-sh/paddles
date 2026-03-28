# Autonomous Retrieval Planning Lane - Product Requirements

## Problem Statement

Paddles can gather static evidence before synthesis, but it cannot yet use Sift's new bounded autonomous planner to decompose multi-hop repository investigation into iterative retrieval turns with planner trace, stop reasons, and retained evidence.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Route decomposition-worthy repository investigation prompts through a bounded autonomous gatherer lane before final synthesis. | Multi-hop retrieval prompts use autonomous planning and return synthesis-ready evidence plus planner metadata | Verified CLI and test proofs |
| GOAL-02 | Preserve the current synthesizer-first answer path for chat, tool execution, and shallow retrieval. | Non-decomposition turns stay on existing paths with no regression in tool or chat routing | Verified routing proofs |
| GOAL-03 | Make autonomous planning observable and controllable for operators. | Planner strategy, trace, stop reasons, retained artifacts, and fallback causes are visible in verbose/debug output | Verified runtime and doc proofs |
| GOAL-04 | Keep future planner evolution optional rather than hard-wiring one model-dependent implementation. | Heuristic planning works by default and model-driven planner profiles remain optional capability-gated extensions | Verified config and adapter design |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer or agent using `paddles` for codebase investigation inside a local workspace. | A retrieval lane that can decompose multi-hop questions without replacing the normal local answer runtime. |
| Runtime Maintainer | The engineer evolving `paddles` model routing and gatherer interfaces. | A clean adapter seam for autonomous planning that remains compatible with other gatherers such as `context-1`. |

## Scope

### In Scope

- [SCOPE-01] Extend the gatherer contract so it can carry planner trace, stop reason, retained artifacts, strategy metadata, and synthesis-ready evidence.
- [SCOPE-02] Add a local Sift autonomous gatherer adapter that wraps `Sift::search_autonomous`.
- [SCOPE-03] Route decomposition-worthy prompts through the autonomous gatherer while preserving the current synthesizer and deterministic tool paths for other turns.
- [SCOPE-04] Expose planner telemetry and fallback behavior in verbose/debug output.
- [SCOPE-05] Document autonomous retrieval planning in foundational architecture/configuration guidance.
- [SCOPE-06] Add evaluation or proof coverage comparing current static context assembly against autonomous retrieval planning on real repository-investigation tasks.

### Out of Scope

- [SCOPE-07] Replacing the synthesizer lane with autonomous planning or turning Sift into the final answer model.
- [SCOPE-08] Implementing branch/graph planning beyond the bounded linear autonomous runtime currently shipped by Sift.
- [SCOPE-09] Making a model-driven planner profile mandatory for the local default path.
- [SCOPE-10] Replacing the experimental `context-1` boundary or remote gatherer work already modeled elsewhere in the architecture.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must define a typed autonomous-gathering result that can carry synthesis-ready evidence, planner trace, stop reason, retained artifacts, warnings, and strategy metadata. | GOAL-01, GOAL-03, GOAL-04 | must | The existing gatherer seam needs richer planner outputs before Sift autonomous search can fit cleanly. |
| FR-02 | Paddles must provide a local Sift autonomous gatherer adapter that maps `ContextGatherRequest` into `Sift::search_autonomous` and returns evidence-first results. | GOAL-01, GOAL-04 | must | This is the primary runtime integration for the new upstream capability. |
| FR-03 | The controller must classify decomposition-worthy prompts and route them through autonomous retrieval planning before final synthesis. | GOAL-01 | must | The new capability only matters if the controller can deliberately select it. |
| FR-04 | Casual chat, coding, deterministic tool turns, and shallow retrieval must preserve the current synthesizer-first path when autonomous planning is unnecessary or unavailable. | GOAL-02 | must | Prevents the new lane from degrading the common path. |
| FR-05 | Verbose/debug output must surface planner strategy, step count, stop reason, retained evidence summary, and fallback causes for autonomous-gatherer turns. | GOAL-03 | should | Operators need enough telemetry to trust and debug routing decisions. |
| FR-06 | Paddles must provide proof or evaluation coverage comparing static context assembly against autonomous retrieval planning for representative repository-investigation prompts. | GOAL-01, GOAL-03 | should | We need evidence that the new lane is worth keeping and where it should be selected. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The default answer/tool path must remain local-first and operational with no new mandatory remote dependency for ordinary prompt handling. | GOAL-02 | must | Preserves a core project invariant. |
| NFR-02 | Autonomous gatherer failures must degrade safely to the current synthesizer path without breaking tool execution or leaving routing state ambiguous. | GOAL-02, GOAL-03 | must | Failure handling is part of the controller contract. |
| NFR-03 | Planner observability must be sufficient for operators to diagnose why the controller chose or skipped autonomous planning. | GOAL-03 | should | Debuggability matters for a routing-heavy system. |
| NFR-04 | The adapter shape must stay extensible so heuristic and model-driven planner strategies can coexist without duplicating controller logic. | GOAL-04 | should | Supports future planner profiles without another runtime rewrite. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Planner contract | Tests and code review of typed gatherer response surfaces | Story evidence for contract and adapter mapping |
| Routing behavior | CLI/manual proofs plus targeted tests for decomposition-worthy versus ordinary prompts | Story evidence showing lane selection and fallback behavior |
| Planner telemetry | Verbose runtime proofs and documentation review | Story evidence showing strategy, trace, stop reason, and retained-artifact visibility |
| Retrieval value | Evaluation notes or comparison proofs against static context assembly | Story evidence demonstrating when autonomous planning improves context gathering |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Sift's bounded autonomous planner can be expressed as an evidence-first gatherer without requiring Paddles to surrender final answer synthesis. | The adapter seam may need to widen or the planner may remain an internal experiment only. | Validate during contract and adapter stories. |
| Decomposition-worthy prompts can be identified well enough with current controller heuristics to justify a separate lane. | Operators may need explicit flags or better classifiers. | Validate during routing story proofs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Planner traces and retained artifacts may enlarge evidence payloads enough to crowd out synthesis context unless budgeted carefully. | Runtime maintainer | Open |
| Heuristic planning may help retrieval quality less than expected on small or obvious prompts, making routing precision important. | Epic owner | Open |
| Future model-driven planner profiles could tempt a silent model/runtime dependency unless capability gating stays explicit. | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles has a typed autonomous-gatherer seam that can represent planner trace, stop reasons, retained artifacts, and synthesis-ready evidence.
- [ ] Multi-hop repository-investigation prompts can route through Sift autonomous retrieval planning while ordinary chat/tool turns preserve the current synthesizer-first path.
- [ ] Operators can inspect planner strategy, planner outcome, and fallback behavior from verbose/runtime output.
- [ ] The repo has proof or evaluation artifacts showing when autonomous retrieval planning outperforms static context assembly for retrieval-heavy tasks.
<!-- END SUCCESS_CRITERIA -->
