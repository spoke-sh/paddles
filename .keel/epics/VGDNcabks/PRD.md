# Deterministic Workspace Entity Resolution - Product Requirements

## Problem Statement

Paddles still relies on fuzzy retrieval and planner guesses to locate workspace entities, so edit-oriented turns hallucinate files, miss code symbols, and exhaust budget before a safe authored-file edit can land.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Let edit-oriented turns resolve likely entities and path hints into real authored workspace files before they spend planner budget on repeated search, inspect, or malformed patch attempts. | Reproduced hallucinated-path edit turns now reach a validated target or emit a deterministic miss/ambiguity outcome instead of wandering into broad search. | Resolver is consulted in the edit path and covered by regression proofs. |
| GOAL-02 | Make deterministic misses and ambiguities legible to operators. | Stream and trace output explains why a target could not be resolved and what candidates were considered. | Resolver outcomes appear in runtime artifacts and docs. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | Operators using Paddles to make concrete repository edits from natural-language requests. | A fast path from user request to the correct authored file without path hallucination or budget exhaustion. |
| Secondary User | Developers debugging planner drift or failed edit turns. | Clear evidence for why a target resolved, missed, or remained ambiguous. |

## Scope

### In Scope

- [SCOPE-01] A deterministic resolver contract that turns entity/path hints into authored workspace file candidates.
- [SCOPE-02] A self-discovering workspace index/cache that respects `.gitignore` and authored-file boundaries.
- [SCOPE-03] Planner-loop and steering integration so known-edit turns consult the resolver before broad search or edit-state actions.
- [SCOPE-04] Operator-facing diagnostics and docs for deterministic resolution, misses, and ambiguities.

### Out of Scope

- [SCOPE-05] IDE-fed context, editor selection/state sync, or workspace state imported from external tools.
- [SCOPE-06] Full language-server integration, cross-repository indexing, or semantic refactors that require symbol ownership across build graphs.
- [SCOPE-07] General retrieval ranking overhauls outside the deterministic entity/path resolution path.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must deterministically resolve likely entity/path hints into authored workspace file candidates before edit-oriented turns continue broad exploration. | GOAL-01 | must | This is the core workflow change needed to stop hallucinated file paths from consuming edit budget. |
| FR-02 | The resolver must report ambiguity and miss states explicitly instead of silently guessing one target. | GOAL-01, GOAL-02 | must | Deterministic failure is safer than hidden drift when a target cannot be trusted. |
| FR-03 | Planner and steering paths must consume resolver outcomes to narrow read/edit actions toward validated authored files. | GOAL-01 | must | The resolver only matters if the planner loop actually uses it to converge. |
| FR-04 | Runtime/operator surfaces must explain deterministic resolution outcomes in the same turn that uses them. | GOAL-02 | should | Operators need to see why a turn converged or failed to converge. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain local-first execution and authored-workspace safety for every resolver path. | GOAL-01 | must | Deterministic resolution cannot compromise the existing workspace safety boundary. |
| NFR-02 | Keep resolver performance and cache invalidation bounded enough for interactive turns. | GOAL-01 | must | A deterministic resolver that stalls the turn loop would trade one failure mode for another. |
| NFR-03 | Keep resolver behavior explainable through traceable artifacts, tests, and documentation. | GOAL-02 | must | Operators need evidence, not opaque controller folklore. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Red/green regression tests plus targeted runtime proofs on known hallucinated-path turns | Story-level verification artifacts and replay evidence |
| Operator visibility | Stream/UI contract tests and doc build checks | Web/TUI/runtime proofs plus docs build logs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Most hallucinated edit turns can be corrected with deterministic path/entity resolution before we need full LSP semantics. | The epic may reduce misses without solving symbol-heavy tasks. | Measure remaining failure cases after voyage-two integration. |
| The current workspace boundary and `.gitignore` policy are strong enough to seed a resolver index safely. | The resolver could leak generated/vendor files back into planner targeting. | Add resolver tests against ignored/generated trees. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which query forms should be first-class: exact path, basename, component name, selector, exported symbol, or all of the above? | Epic owner | Planned in voyage one |
| How much of the existing fuzzy retrieval path should remain after deterministic resolution exists? | Epic owner | Planned in voyage two |
| Symbol-like resolution without LSP may still miss language-specific ownership edges. | Epic owner | Accepted risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Edit-oriented repro turns that previously hallucinated files now either reach a validated authored target or fail closed with an explicit deterministic miss/ambiguity explanation.
- [ ] The active planner path consults deterministic resolution before repeated broad search once a likely target family is known.
- [ ] Foundational and public docs explain deterministic resolution behavior, scope, and limits without promising IDE-fed or LSP-backed semantics.
<!-- END SUCCESS_CRITERIA -->
