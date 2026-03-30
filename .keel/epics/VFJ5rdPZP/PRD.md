# Model-Judged Heuristic Removal - Product Requirements

## Problem Statement

Paddles still contains lexical routing, fallback, interpretation-scoring, and retrieval-ranking heuristics that substitute controller guesses for model judgement in the recursive harness.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Move remaining reasoning-heavy controller guesses on the primary harness path into constrained model-judged decisions. | Primary turn paths no longer depend on string heuristics or lexical fallback ranking for routing, interpretation, or next-step reasoning | Verified runtime/tests |
| GOAL-02 | Shift retrieval-selection and evidence-prioritization choices that reflect reasoning into model-judged flows. | Recursive retrieval paths no longer rely on hardcoded lexical ranking or static fallback queries where model judgement should decide | Verified runtime/tests |
| GOAL-03 | Preserve controller-owned safety constraints while removing reasoning heuristics. | Allowlists, budgets, deterministic execution, path validation, and fail-closed behavior remain intact and explicitly documented | Verified code/docs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using `paddles` with small local models. | The harness should let the model reason from project guidance and tool outputs instead of being boxed in by controller guesses. |
| Runtime Maintainer | An engineer evolving planner/gatherer/synthesizer boundaries. | A clearer controller-versus-model split so heuristics do not quietly drift back into the runtime. |
| Small Local Planner Model | The constrained planner lane used by `paddles`. | More of the reasoning budget should stay in-model, with the controller acting as a validator and executor rather than a hidden classifier. |

## Scope

### In Scope

- [SCOPE-01] Replace remaining legacy direct-path routing/tool inference heuristics that still think for the model.
- [SCOPE-02] Replace lexical interpretation scoring and hint/procedure ranking where model judgement should select the relevant guidance and next step.
- [SCOPE-03] Replace heuristic planner fallback selection and fallback query derivation with constrained model re-decision passes wherever practical.
- [SCOPE-04] Replace retrieval-selection and evidence-prioritization heuristics that encode reasoning rather than safety.
- [SCOPE-05] Update foundational docs and proof artifacts so the controller-versus-model boundary is explicit.

### Out of Scope

- [SCOPE-06] Removing controller-owned budgets, allowlists, validation, or fail-closed behavior.
- [SCOPE-07] Replacing the local-first runtime model stack or requiring a remote planner.
- [SCOPE-08] Making `AGENTS.md` root discovery configurable in this slice.
- [SCOPE-09] A full unified resource graph across every tool surface in one mission.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Primary turn processing must no longer rely on legacy string heuristics such as casual detection, direct tool preference, or canned follow-up execution inference where a constrained model decision should choose the next step. | GOAL-01 | must | The harness should stop performing hidden top-level reasoning that belongs to the model. |
| FR-02 | Interpretation-time guidance selection must be driven by constrained model judgement from `AGENTS.md` roots and their referenced subgraph instead of lexical scoring and static relevance ranking. | GOAL-01 | must | Guidance relevance is a reasoning problem, not a controller keyword problem. |
| FR-03 | Invalid initial-action and planner-action replies must prefer a constrained model re-decision path before controller fallback, and any residual fallback must be minimal, explicit, and fail-closed. | GOAL-01 | must | The current harness still substitutes controller guesses when the model replies are invalid. |
| FR-04 | Retrieval mode/query selection and evidence prioritization in recursive work must move toward model-judged choices wherever those choices represent reasoning rather than safety constraints. | GOAL-02 | must | Static lexical defaults and source-priority heuristics still distort what evidence the model sees. |
| FR-05 | The controller must continue to own budgets, safe-command allowlists, path validation, deterministic execution, and fail-closed behavior after the heuristic-removal work lands. | GOAL-03 | must | The mission is about removing reasoning heuristics, not safety constraints. |
| FR-06 | Foundational docs and proof artifacts must explain which choices are model-judged, which remain controller-owned, and why. | GOAL-03 | must | The architecture should be legible and resistant to future drift. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The heuristic-removal work must remain local-first, bounded, observable, and fail-closed. | GOAL-01, GOAL-03 | must | Controller safety and operator trust cannot regress while shifting reasoning into the model. |
| NFR-02 | The new model-judged reasoning path must stay generic across repositories rather than encoding project-specific intents as replacement heuristics. | GOAL-01, GOAL-02 | must | The harness should improve generally, not just for one board shape. |
| NFR-03 | Operator-visible traces must continue to explain what the model chose versus what the controller validated/executed. | GOAL-03 | should | Removing heuristics is only trustworthy if the resulting decisions stay visible. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Primary routing/fallback behavior | Unit/integration tests plus transcript proofs | Story evidence showing model-selected interpretation and re-decision replacing legacy heuristics |
| Retrieval/evidence behavior | Tests and CLI/TUI proofs | Story evidence showing model-judged retrieval selection and evidence prioritization on recursive turns |
| Controller safety preservation | Tests and code review | Story evidence showing allowlists, budgets, and fail-closed behavior remain intact |
| Docs and architecture | Doc review plus proof artifact | Updated foundational docs and a controller-vs-model boundary proof |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Small local planner models can make better decisions than the current lexical/controller heuristics when given constrained prompts and the right evidence. | The mission could remove stable fallbacks without improving behavior. | Validate through targeted runtime proofs and regression tests. |
| Some retrieval and evidence-ranking heuristics are standing in for reasoning rather than safety. | The mission scope could accidentally target useful controller constraints. | Validate explicitly in SRS/SDD and preserve the safety-owned surfaces. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How far can heuristic removal go in one slice before the planner needs a richer multi-pass interpretation/retrieval contract? | Runtime maintainer | Open |
| Some retrieval heuristics may actually be compensating for weak local models; removing them prematurely could hurt quality. | Runtime maintainer | Open |
| Legacy direct-path helpers may still be useful as emergency fail-closed recovery even after they stop being part of the primary reasoning path. | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The primary harness path no longer depends on reasoning-heavy string/lexical heuristics for routing, interpretation, and planner fallback.
- [ ] Retrieval-selection and evidence-prioritization choices that reflect reasoning have moved to constrained model judgement on the recursive path.
- [ ] Controller-owned safety constraints remain intact and are explicitly documented as such.
<!-- END SUCCESS_CRITERIA -->
