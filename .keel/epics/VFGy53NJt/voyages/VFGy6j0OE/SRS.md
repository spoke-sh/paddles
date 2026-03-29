# Graph Retrieval Through The Gatherer Boundary - SRS

## Summary

Epic: VFGy53NJt
Goal: Upgrade the Sift gatherer path so Paddles can use bounded graph-mode autonomous search, preserve graph episode state and branch/frontier metadata, and surface that richer context through the recursive planning harness without sacrificing local-first safety.

## Scope

### In Scope

- [SCOPE-01] Update the pinned `sift` dependency and gatherer-facing config for graph-mode autonomous search.
- [SCOPE-02] Preserve graph episode/frontier/branch metadata in typed `paddles` evidence and turn events.
- [SCOPE-03] Route recursive search/refine work through graph-capable gatherers via the generic planner/gatherer boundary.
- [SCOPE-04] Document and prove the graph-mode gatherer integration.

### Out of Scope

- [SCOPE-05] Keel-specific runtime intents or board-only graph routing.
- [SCOPE-06] Mandatory remote graph planners or making `context-1` the default graph provider.
- [SCOPE-07] A full unified resource graph beyond the gatherer/search boundary in this slice.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `paddles` must advance to the latest upstream `sift` revision that includes bounded graph-mode autonomous search and branching/episode contracts. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The gatherer configuration must be able to express bounded `linear` versus `graph` autonomous retrieval, along with optional planner profile selection. | SCOPE-01 | FR-02 | manual |
| SRS-03 | The Sift autonomous gatherer adapter must be able to execute graph-mode retrieval and preserve typed graph episode/frontier/branch metadata, stable step and turn identifiers, and graph stop reasons in the evidence bundle. | SCOPE-01, SCOPE-02 | FR-03 | manual |
| SRS-04 | Recursive search/refine work must be able to request graph-capable gatherer behavior through the generic planner/gatherer path without introducing repository-specific top-level routing branches. | SCOPE-03 | FR-04 | manual |
| SRS-05 | Default operator surfaces must render graph planner summaries, branch/frontier state, and graph stop reasons from structured trace/event data when graph-mode gatherers are used. | SCOPE-02, SCOPE-03 | FR-05 | manual |
| SRS-06 | The graph-mode evidence and telemetry contract must leave room for future external artifact references instead of forcing large graph traces and tool outputs to remain inline forever. | SCOPE-02, SCOPE-03, SCOPE-04 | FR-06 | manual |
| SRS-07 | Foundational docs and proof artifacts must explain the graph-mode gatherer capability, config surface, fallback behavior, current limits, and future embedded-recorder seam honestly. | SCOPE-04 | FR-07 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Graph-mode gatherers must remain local-first, bounded, and fail closed when graph planning is invalid or unavailable. | SCOPE-01, SCOPE-03 | NFR-01 | manual |
| SRS-NFR-02 | Graph-mode evidence and telemetry must stay generic across repositories and evidence domains rather than Keel-specific. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-03 | The richer graph planner telemetry must remain concise enough for the default operator surface. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-03 | manual |
| SRS-NFR-04 | The graph-mode integration must stay compatible with a future embedded `transit-core` recorder and must not require a networked trace server to preserve graph-capable turns durably. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
