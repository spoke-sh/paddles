# Runtime Boundary Separation - Product Requirements

## Problem Statement

Planner orchestration still owns the recursive executor loop plus terminal, workspace-action, and external-capability execution helpers inside src/application/mod.rs, making planner, executor, and capability adapter boundaries harder to review and extend.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make planner orchestration call dedicated executor-side modules for the recursive executor loop, terminal/workspace execution metadata, and external-capability execution. | `src/application/mod.rs` no longer owns those helper implementations, and behavior remains green under focused and full tests. | Extract the recursive executor loop, planner action execution helpers, and external-capability execution helpers into application modules. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime Maintainer | Engineers changing planner, executor, or external-capability behavior. | Clear module ownership so planner orchestration does not also own adapter execution details. |

## Scope

### In Scope

- [SCOPE-01] Move recursive planner executor loop behavior out of `src/application/mod.rs`.
- [SCOPE-02] Move planner action execution helper behavior out of `src/application/mod.rs`.
- [SCOPE-03] Move external-capability invocation formatting, governance execution, result summarization, and evidence projection out of `src/application/mod.rs`.
- [SCOPE-04] Preserve existing planner/executor/capability behavior with focused regression tests and full repository verification.

### Out of Scope

- [SCOPE-05] Changing planner decision policy, executor governance policy, or external-capability availability semantics.
- [SCOPE-06] Rewriting provider adapters such as HTTP or Sift planners.
- [SCOPE-07] Introducing new external tools, services, or network dependencies.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The recursive planner executor loop is owned by the recursive control chamber. | GOAL-01 | must | Separates service wiring from executor loop control and action dispatch. |
| FR-02 | Planner action execution helpers are owned by a dedicated application module. | GOAL-01 | must | Separates planner orchestration from executor-side formatting and terminal helper details. |
| FR-03 | External-capability execution is owned by a dedicated application module. | GOAL-01 | must | Keeps capability adapter governance, invocation formatting, result summarization, and evidence projection reusable. |
| FR-04 | Existing service callers keep using the same runtime APIs after extraction. | GOAL-01 | must | Confirms the refactor is behavior-preserving. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve local-first governance and evidence behavior during the extraction. | GOAL-01 | must | Planner/executor separation cannot bypass execution policy or evidence recording. |
| NFR-02 | Keep verification on the standard Rust and Keel paths. | GOAL-01 | must | Boundary refactors need fast, repeatable proof. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Planner/executor separation | Focused tests for recursive execution and planner action helper modules plus code review | Story evidence |
| External-capability separation | Focused tests for external capability execution behavior | Story evidence |
| Behavior preservation | `cargo test` and `keel doctor` | Story evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing tests cover the selected helper behavior strongly enough to detect regressions. | The story must add focused tests before moving code. | Red/green TDD. |
| The extraction can keep helper visibility `pub(super)` and avoid widening the public API. | More public exports may be needed. | Compile and review module imports. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should provider-specific planner adapter parsing be split in a later mission? | Epic owner | Watch |
| Should workspace action execution become a standalone service object after helper extraction? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Recursive planner executor loop lives outside `src/application/mod.rs`.
- [ ] Planner action execution helpers live outside `src/application/mod.rs`.
- [ ] External-capability execution helpers live outside `src/application/mod.rs`.
- [ ] Focused and full repository tests pass, and Keel evidence is linked.
<!-- END SUCCESS_CRITERIA -->
