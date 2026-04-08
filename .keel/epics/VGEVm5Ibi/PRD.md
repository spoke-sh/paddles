# Decompose The React Runtime Web UI - Product Requirements

## Problem Statement

The React runtime web UI is concentrated in a few monolithic files, which makes state flow, route behavior, fallback parity, and tests harder to evolve safely.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Split the React runtime into coherent modules for shell/chat, store/transport, and route domains so maintainers can change one surface without reopening the entire runtime file. | The runtime no longer depends on one file to own shell state plus all three route implementations. | Shell, store, and route responsibilities live in dedicated modules with clear ownership. |
| GOAL-02 | Preserve current user-visible behavior while the runtime is decomposed. | Existing runtime/chat/inspector/manifold/transit behaviors remain covered by tests and route contracts during the refactor. | Modularization lands without intentional UX drift. |
| GOAL-03 | Make the styling and fallback-shell boundary explicit so future work can evolve the React app without hidden duplication. | Styling and fallback parity rules are documented and enforced through module-local tests or contracts. | The embedded shell is treated as a bounded contract surface instead of implicit copy-over logic. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | Maintainers evolving the web runtime and adding runtime UI features. | Clear, localized module ownership so changes to one surface do not require editing a 2k-line React file. |
| Secondary User | Operators debugging runtime regressions across chat, manifold, and transit views. | Stable behavior and tests while the internal structure changes. |

## Scope

### In Scope

- [SCOPE-01] Extract app/shell/chat/composer concerns out of the monolithic runtime app into dedicated modules.
- [SCOPE-02] Separate runtime transport and event/projection reduction concerns from the UI shell.
- [SCOPE-03] Move inspector, manifold, and transit routes into dedicated domain modules with local hooks/selectors where needed.
- [SCOPE-04] Partition runtime styles and tests by feature surface.
- [SCOPE-05] Define and guard the embedded fallback-shell parity boundary affected by the modularization.

### Out of Scope

- [SCOPE-06] Redesigning the runtime UI, changing product behavior, or introducing new runtime features.
- [SCOPE-07] Replacing the embedded fallback shell with a new delivery mechanism.
- [SCOPE-08] General frontend architecture changes outside the runtime web surface.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime shell must compose dedicated modules for app layout, transcript/composer behavior, and runtime-store transport instead of implementing all of them in one route file. | GOAL-01 | must | This creates the first stable ownership boundary for the React runtime. |
| FR-02 | Inspector, manifold, and transit must each have dedicated route modules with localized stateful helpers/selectors. | GOAL-01 | must | Route-specific logic is the main remaining source of monolithic concentration. |
| FR-03 | Styling and tests must align with the decomposed module boundaries so maintainers can change a surface without editing one global stylesheet or one kitchen-sink test file. | GOAL-01, GOAL-02 | should | The module split is incomplete if CSS and tests remain centralized. |
| FR-04 | The embedded fallback-shell contract affected by the modularization must be explicitly documented and regression-guarded. | GOAL-02, GOAL-03 | should | The current duplication is risky unless the contract boundary is made explicit. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve existing runtime behavior, route contracts, and current stream interactions while refactoring the module structure. | GOAL-02 | must | This is a structural refactor, not a product redesign. |
| NFR-02 | Keep module interfaces explicit and low-coupling so future changes can stay within one domain surface where possible. | GOAL-01, GOAL-03 | must | A decomposition that only moves code without clarifying seams does not solve the maintainability problem. |
| NFR-03 | Keep the decomposition explainable through authored planning docs, tests, and fallback parity contracts. | GOAL-02, GOAL-03 | must | Hidden architectural folklore would recreate the same maintenance problem. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Red/green route- and store-level tests plus targeted runtime proofs during each extraction slice | Story-level verification artifacts and route-specific test runs |
| Fallback boundary | Contract tests and explicit documentation review | Embedded-shell tests and voyage/story evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current runtime behavior is already sufficiently covered to let us refactor in sealed slices. | The team may need to add coverage before extracting modules safely. | Voyage stories add or preserve tests around each extracted surface. |
| The embedded fallback shell should remain supported during this decomposition rather than being replaced. | Later stories may need human direction on whether to keep parity or retire the fallback path. | Explicitly document fallback parity scope in voyage three. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much React-side behavior should remain mirrored in the embedded shell versus intentionally constrained to a smaller contract? | Epic owner | Planned in voyage three |
| CSS partitioning may reveal hidden coupling that forces small markup adjustments during extraction. | Epic owner | Accepted risk |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Shell/chat/store concerns are extracted into dedicated modules so the runtime app file primarily assembles routes and providers.
- [ ] Inspector, manifold, and transit route logic each live in dedicated domain modules rather than one monolithic runtime file.
- [ ] Styling, tests, and embedded-shell parity boundaries are documented and aligned with the decomposed module structure.
<!-- END SUCCESS_CRITERIA -->
