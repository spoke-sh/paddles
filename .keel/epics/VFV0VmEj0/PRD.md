# Use Sift As A Retrieval Engine Instead Of A Nested Planner - Product Requirements

## Problem Statement

Paddles already performs recursive planning in its own harness, but the sift-autonomous gatherer introduces a second hidden planner that duplicates decisions, slows turns, and obscures progress. We need a direct sift-backed retrieval boundary that preserves paddles-owned planning and makes retrieval execution observable.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Ensure paddles remains the sole planner for recursive investigation and refinement. | Search-heavy turns no longer route through `sift-autonomous` planning | 100% of planner-driven gatherer turns use direct retrieval execution |
| GOAL-02 | Make sift-backed retrieval execution observable during long-running turns. | Progress output identifies concrete retrieval stages and current work instead of opaque autonomous planner states | User-visible progress covers initialization, indexing, retrieval, ranking, and completion/fallback states |
| GOAL-03 | Preserve search quality and operational flexibility while removing the nested planner. | Direct retrieval path supports current lexical/hybrid usage and configurable fallback behavior | No loss of supported retrieval modes needed by the paddles harness |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | A paddles operator using the recursive coding harness on large workspaces. | Fast, comprehensible retrieval that does not disappear into a second hidden planner. |
| Secondary User | A maintainer evolving paddles and sift integration boundaries. | A clean architectural split between planning decisions and retrieval execution. |

## Scope

### In Scope

- [SCOPE-01] Replace the `sift-autonomous` gatherer adapter path used by planner-driven turns with direct sift-backed retrieval execution.
- [SCOPE-02] Define and surface execution-stage progress semantics for direct sift retrieval in the TUI and web UI.
- [SCOPE-03] Update configuration, adapter boundaries, and harness assumptions to reflect paddles-owned planning.
- [SCOPE-04] Document the new search boundary, capabilities, and constraints clearly enough for future maintenance.

### Out of Scope

- [SCOPE-05] Rebuilding sift’s full upstream autonomous planner implementation.
- [SCOPE-06] Broad search relevance tuning unrelated to removing nested planning.
- [SCOPE-07] New remote retrieval providers or network-backed search features.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Planner-driven gatherer turns must execute retrieval through a direct sift-backed search path rather than `sift-autonomous` planning. | GOAL-01 | must | Removes duplicate hidden planning and restores architectural control to paddles. |
| FR-02 | The direct retrieval path must support the retrieval modes and strategies the harness currently relies on for workspace search. | GOAL-01, GOAL-03 | must | Prevents regression while changing the boundary. |
| FR-03 | Progress events must describe concrete retrieval stages and current work, not autonomous planner internals like `Terminate`. | GOAL-02 | must | Users need insight into why search is slow and what stage is active. |
| FR-04 | Configuration and provider selection must make it clear that sift is being used as a retrieval backend, not as a nested planner. | GOAL-01, GOAL-03 | should | Keeps the runtime model comprehensible and reduces future drift. |
| FR-05 | Search documentation must explain the new boundary, what sift contributes, and what paddles itself owns. | GOAL-02, GOAL-03 | should | Maintainers need a single clear source of truth for the integration model. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Long-running retrieval must continue emitting progress updates during work that exceeds a few seconds. | GOAL-02 | must | Prevents frozen-looking UX during indexing or ranking. |
| NFR-02 | The new retrieval boundary must keep local-first execution and avoid introducing new network dependencies. | GOAL-01, GOAL-03 | must | Preserves the repository’s local-first operating model. |
| NFR-03 | The boundary change must remain debuggable through trace output, progress events, and clear stop/fallback reasons. | GOAL-02 | must | Maintainers need to understand slow or degraded search behavior. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Boundary replacement | Tests plus trace/progress review | Story-level verification showing paddles-owned planning with direct retrieval execution |
| UX observability | TUI/web/manual review plus event traces | Evidence that progress messages surface retrieval stages and reasons for delay |
| Documentation | Review of mission and voyage artifacts plus user docs | Updated docs that describe constraints, capabilities, and ownership boundaries |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Direct sift-backed retrieval can cover the harness’s current lexical/hybrid needs without relying on autonomous planner recursion. | A larger sift integration change may be required. | Validate in the first voyage design and implementation stories. |
| The current user pain is driven more by opaque nested planning than by raw retrieval quality alone. | Removing autonomy may not materially improve UX. | Validate through progress surfacing and execution traces during implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which direct sift API shape should become the stable gatherer boundary inside paddles? | Epic owner | Open |
| How much progress detail can sift surface without requiring additional upstream instrumentation? | Epic owner | Open |
| Whether any legacy config names should be retained temporarily for compatibility. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles-owned planning executes search turns without delegating recursive planning to `sift-autonomous`.
- [ ] User-visible progress explains concrete retrieval stages during long-running search work.
- [ ] Mission artifacts and docs clearly describe sift as a retrieval backend rather than a second planner.
<!-- END SUCCESS_CRITERIA -->
