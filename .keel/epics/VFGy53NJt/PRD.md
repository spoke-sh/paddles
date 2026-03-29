# Graph-Mode Gatherer Integration - Product Requirements

## Problem Statement

Paddles uses Sift autonomous search through the gatherer boundary today, but it does not expose the new graph/branching runtime or its richer episode state, frontier behavior, and branch-local evidence to the recursive planning harness.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Upgrade `paddles` to the current upstream `sift` graph/branching surface. | `Cargo.lock` advances to the graph-capable upstream `sift` revision and graph-mode APIs are used in the gatherer adapter | Verified dependency diff and runtime tests |
| GOAL-02 | Make graph-mode retrieval available through the generic gatherer boundary. | Gatherer config can choose bounded `linear` or `graph` autonomous retrieval without introducing repository-specific top-level intents | Verified contract/tests |
| GOAL-03 | Preserve graph episode/frontier/branch context as typed evidence and visible operator telemetry. | Graph-mode turns surface stable branch/frontier/step metadata and branch-local evidence in typed trace/event data and the operator UX | Verified traces and tests |
| GOAL-04 | Keep the integration local-first, bounded, and honestly documented. | Invalid or unavailable graph mode fails closed, and foundational docs describe the new capability and remaining edges accurately | Verified docs and proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using `paddles` interactively with small local models. | Better multi-hop retrieval because the harness can preserve branch-local evidence instead of flattening everything into a linear search summary. |
| Runtime Maintainer | An engineer evolving routing, gatherers, and synthesis seams. | A clean gatherer boundary that can adopt richer Sift search modes without collapsing the generic harness architecture. |
| Model Router | The person choosing local planner/synth/gatherer workloads. | A bounded graph-capable gatherer lane that improves retrieval without forcing a larger default answer model. |

## Scope

### In Scope

- [SCOPE-01] Update `sift` to the latest upstream `main` revision that includes graph-mode autonomous runtime and branching/episode surfaces.
- [SCOPE-02] Extend `paddles` gatherer/planning configuration so graph mode can be selected as a generic autonomous retrieval mode.
- [SCOPE-03] Map graph episode/frontier/branch metadata into `paddles` evidence and operator-visible turn telemetry.
- [SCOPE-04] Route recursive search/refine work through graph-capable Sift gatherers while preserving bounded local-first fallback behavior.
- [SCOPE-05] Update foundational docs and proof artifacts so operators understand graph-mode gatherers as part of the recursive harness.

### Out of Scope

- [SCOPE-06] Hardcoded Keel-specific or board-specific graph routing logic.
- [SCOPE-07] Mandatory remote planner models or making `context-1` the default graph runtime.
- [SCOPE-08] Replacing the top-level model-directed action contract delivered by mission `VFECvinGX`.
- [SCOPE-09] A full unified resource graph across every tool surface in one slice.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The repository must advance to the latest upstream `sift` revision that includes graph-mode autonomous search and branching/episode contracts. | GOAL-01 | must | `paddles` should use the real upstream surface rather than emulate graph behavior locally. |
| FR-02 | The gatherer/planning contract must be able to express whether autonomous retrieval should run in bounded `linear` or `graph` mode, along with optional planner profile selection. | GOAL-01, GOAL-02 | must | The harness needs a generic way to ask for graph retrieval without introducing product-specific intents. |
| FR-03 | The Sift autonomous gatherer adapter must be able to execute graph-mode retrieval and preserve graph episode/frontier/branch information through typed `paddles` metadata with stable machine-readable identifiers for steps, turns, and branch-local evidence rather than leaking raw upstream internals. | GOAL-02, GOAL-03 | must | Richer retrieval is only useful if the rest of the harness can see and reason over it, and later durable recorders should not need to reconstruct lineage from prose. |
| FR-04 | Recursive search/refine work must be able to request graph-capable gatherer behavior through the existing generic planner/gatherer path instead of top-level heuristics or repository-specific routing branches. | GOAL-02, GOAL-03 | must | The graph integration should strengthen the generic recursive harness, not bypass it. |
| FR-05 | Turn events and gathered evidence must surface graph planner summaries, branch/frontier state, and graph stop reasons through structured `paddles` trace/event data that can be rendered in the default operator UX without depending on prose-only strings. | GOAL-03, GOAL-04 | must | Operators need to see the extra retrieval work the graph runtime is doing, and future recorders should be able to persist the same structure directly. |
| FR-06 | The graph-mode evidence and telemetry contract must leave room for future external artifact references instead of assuming that large graph traces, tool outputs, or evidence payloads always remain inline. | GOAL-03, GOAL-04 | should | Transit-style artifact envelopes are easier to add later if this slice does not hardcode an inline-only shape now. |
| FR-07 | Foundational docs and proof artifacts must explain the graph-mode gatherer capability, config, fallback behavior, and remaining limitations, including the intended future handoff to an embedded durable recorder boundary. | GOAL-04 | must | The architecture shift should be legible from the docs, not only from code. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Graph-mode gatherers must remain local-first, bounded, and fail closed when upstream graph planning is invalid or unavailable. | GOAL-01, GOAL-04 | must | The richer retrieval path cannot weaken the core runtime guarantees. |
| NFR-02 | Graph-mode evidence and telemetry must remain generic across repositories and evidence domains rather than overfitting to Keel or one project layout. | GOAL-02, GOAL-03 | must | The value is in a better gatherer boundary, not a special-case board runtime. |
| NFR-03 | Default operator surfaces must remain observable and concise even when graph metadata is richer than the previous linear autonomous summaries. | GOAL-03, GOAL-04 | should | Graph mode should improve trace quality without overwhelming the transcript. |
| NFR-04 | The graph-mode integration must remain compatible with a future embedded `transit-core` recorder and must not require a networked trace server to preserve graph-capable turns durably. | GOAL-03, GOAL-04 | should | `paddles` should be able to record recursive work locally before any shared-service deployment exists. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Upstream dependency + config | Cargo diff, compile/tests, and contract review | Story evidence showing the updated `sift` revision and graph-mode config path |
| Graph metadata preservation | Unit tests and runtime trace proofs | Story evidence showing graph episode/frontier/branch metadata in evidence and turn events |
| Recursive routing + fallback | Tests and CLI/TUI proofs | Story evidence showing graph-capable gatherer usage, bounded fallback, and local-first behavior |
| Docs and operator guidance | Doc review plus proof artifact | Updated foundational docs, transit-readiness notes, and a graph-mode routing proof |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The upstream `sift` graph runtime is mature enough to embed behind the current gatherer boundary without redesigning the whole recursive planner loop. | The mission may need to stop at dependency/config prep and defer full routing adoption. | Validate through compile/runtime tests and proof traces. |
| Preserving graph episode state in typed `paddles` metadata is more useful than flattening graph results into a linear summary. | The integration may add complexity without materially improving grounded synthesis. | Validate through operator-visible traces and before/after evidence proofs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much raw graph episode state should `paddles` preserve versus summarizing into a smaller domain DTO while still keeping stable ids for a later recorder boundary? | Runtime maintainer | Open |
| The top-level planner may eventually need to choose graph vs linear mode dynamically rather than through static config alone. | Runtime maintainer | Open |
| Upstream graph planner profiles may evolve quickly enough to require another dependency lift soon after this mission. | Runtime maintainer | Open |
| Should future durable recording consume the same typed planner/evidence data directly or a normalized trace contract one level above the current turn-event sink? | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles` runs on the current upstream `sift` revision that exposes bounded graph-mode autonomous search.
- [ ] The gatherer boundary can select graph mode and preserve graph branch/frontier/step/stop metadata with stable ids as typed evidence and trace data.
- [ ] Recursive retrieval can use graph-capable gatherers without introducing repository-specific routing logic.
- [ ] Foundational docs and proof artifacts explain the integration, its remaining limits, and how it stays ready for a future embedded recorder boundary.
<!-- END SUCCESS_CRITERIA -->
