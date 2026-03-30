# Sift Search Progress In Paddles TUI - Product Requirements

## Problem Statement

The sift graph/hybrid search blocks synchronously for 30-60+ seconds during workspace indexing with zero progress reporting to the user, making the TUI appear frozen — users cannot tell if paddles is working, stuck, or how long they need to wait.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Users see continuous progress updates during sift search operations | TUI shows indexing status, planner step progress, and elapsed time during the wait | Updates at least every 2 seconds |
| GOAL-02 | The search call doesn't block the tokio runtime | search_autonomous runs on a blocking thread with channel-based progress reporting | UI remains responsive during search |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer waiting for graph/hybrid search results | Know if the system is working and roughly how long to wait |

## Scope

### In Scope

- [SCOPE-01] Move search_autonomous off the tokio runtime thread (spawn_blocking)
- [SCOPE-02] Emit TurnEvents during sift search for indexing and planner step progress
- [SCOPE-03] TUI rendering of search progress (elapsed time, phase description)
- [SCOPE-04] Graceful degradation when sift provides no callbacks (elapsed timer only)
- [SCOPE-05] Document upstream sift callback requirements for future integration

### Out of Scope

- [SCOPE-06] Implementing the sift-side callback API (upstream dependency)
- [SCOPE-07] Cancellation of in-progress sift searches
- [SCOPE-08] Percentage-based progress bars (requires upstream support)

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | search_autonomous call is wrapped in tokio::task::spawn_blocking | GOAL-02 | must | Prevents blocking the async runtime |
| FR-02 | A new TurnEvent::GathererSearchProgress variant reports search phase and elapsed time | GOAL-01 | must | Enables TUI to show updates during the wait |
| FR-03 | TUI renders search progress events with elapsed timer and phase description | GOAL-01 | must | User sees the system is working |
| FR-04 | When sift exposes a progress callback, paddles can pipe it to TurnEvents | GOAL-01 | should | Future-proofing for upstream integration |
| FR-05 | Upstream sift progress callback requirements documented as a bearing or ADR | GOAL-01 | must | Coordinates cross-repo dependency |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Progress events emitted at most every 2 seconds to avoid TUI flicker | GOAL-01 | should | Smooth UX |
| NFR-02 | No performance regression on the search path from the spawn_blocking wrapper | GOAL-02 | must | Same or better throughput |
| NFR-03 | Graceful fallback: elapsed timer shown even without sift callbacks | GOAL-01 | must | Works today, improves with upstream |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Non-blocking search | Manual: TUI remains responsive during sift search | Session observation |
| Progress events | Manual: TUI shows elapsed time during search | Session output |
| Upstream documentation | Bearing or ADR authored | Document artifact |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| sift's search_autonomous can be called from a non-tokio thread via spawn_blocking | May need to restructure the adapter | Test with spawn_blocking wrapper |
| Elapsed timer provides sufficient UX improvement without percentage progress | Users may still feel uncertain | Monitor feedback after integration |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What callback shape should sift expose? (channel, trait object, closure) | Sift upstream | Open — needs design |
| Should sift report indexing and planning as separate phases? | Sift upstream | Open |
| Can sift estimate indexing completion time from file count? | Sift upstream | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] TUI shows elapsed time and phase during sift search (not silent)
- [ ] search_autonomous no longer blocks the tokio runtime thread
- [ ] Upstream sift progress requirements are documented
<!-- END SUCCESS_CRITERIA -->
