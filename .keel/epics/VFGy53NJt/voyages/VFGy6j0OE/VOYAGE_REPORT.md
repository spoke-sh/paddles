# VOYAGE REPORT: Graph Retrieval Through The Gatherer Boundary

## Voyage Metadata
- **ID:** VFGy6j0OE
- **Epic:** VFGy53NJt
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Upgrade Sift And Expose Graph Mode In Gatherer Config
- **ID:** VFGy8yaU7
- **Status:** done

#### Summary
Advance the pinned `sift` dependency to the latest upstream revision that
includes bounded graph-mode autonomous search, then extend the gatherer-facing
planning config so `paddles` can request bounded `linear` versus `graph`
retrieval without introducing repository-specific routing types.

#### Acceptance Criteria
- [x] `Cargo.lock` advances to the latest upstream `sift` revision that includes graph-mode autonomous search and branching/episode APIs. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] The gatherer/planning config can express bounded `linear` versus `graph` autonomous retrieval along with optional planner profile selection. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [x] The config surface remains generic across repositories and evidence domains rather than introducing a board-specific or Keel-specific route selector. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->

### Preserve Graph Episode State In Evidence And Turn Events
- **ID:** VFGy8zKU8
- **Status:** done

#### Summary
Map the richer upstream graph episode/frontier/branch state into typed
`paddles` metadata so graph-mode gatherers can surface useful branch-local
evidence, graph stop reasons, stable machine-readable lineage identifiers, and
concise operator-visible telemetry without leaking raw `sift` internals
through the domain boundary.

#### Acceptance Criteria
- [x] Graph-mode gatherer results preserve typed graph episode/frontier/branch metadata, stable step/turn identifiers, and graph stop reasons in the gathered evidence bundle. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [x] The metadata boundary remains `paddles`-owned rather than exposing raw upstream `sift` graph DTOs across the domain. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->

### Route Recursive Search Through Graph-Capable Sift Gatherers
- **ID:** VFGy8zqVS
- **Status:** done

#### Summary
Use the new graph-capable gatherer path from the existing model-directed
recursive harness so search/refine work can benefit from bounded graph-mode
retrieval while preserving local-first fallback behavior and avoiding new
repository-specific top-level intents.

#### Acceptance Criteria
- [x] Recursive search/refine work can request graph-capable gatherer behavior through the generic planner/gatherer path. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [x] Graph-mode gatherers remain local-first, bounded, and fail closed when graph planning is invalid or unavailable. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
- [x] Recursive planner/synthesizer handoff continues to operate with graph-capable gathered evidence instead of flattening the path back into an opaque linear summary. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] The default operator surface renders concise graph planner summaries, branch/frontier state, and graph stop reasons when graph-mode retrieval is active. [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end -->
- [x] The graph-capable route remains compatible with a future embedded recorder and does not assume a networked trace server. [SRS-NFR-04/AC-05] <!-- verify: manual, SRS-NFR-04:start:end -->

### Document And Prove Graph-Mode Gatherer Routing
- **ID:** VFGy90OWi
- **Status:** done

#### Summary
Update the foundational docs and capture proof artifacts so operators can see
how graph-mode gatherers fit into the recursive harness, how to configure them,
what telemetry to expect, and where the current implementation still stops
short of a fully unified resource graph or durable recorder integration.

#### Acceptance Criteria
- [x] README and companion architecture/config docs describe graph-mode gatherers as part of the recursive harness rather than as a special-case product feature. [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [x] Operator guidance documents explain the config, local-first fallback behavior, default telemetry, and future embedded-recorder seam for graph-mode gatherers. [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end -->
- [x] Proof artifacts show at least one graph-mode gatherer trace or before/after comparison against the prior linear-only gatherer behavior. [SRS-07/AC-03] <!-- verify: manual, SRS-07:start:end -->
- [x] The docs note that large graph traces and tool outputs may later move behind artifact references instead of remaining inline forever. [SRS-06/AC-04] <!-- verify: manual, SRS-06:start:end -->


