# Direct Sift Retrieval Boundary - SRS

## Summary

Epic: VFV0VmEj0
Goal: Replace the nested sift-autonomous planner path with direct sift-backed retrieval, expose concrete retrieval-stage progress, and leave paddles as the sole recursive planner.

## Scope

### In Scope

- [SCOPE-01] Replace the `sift-autonomous` gatherer boundary with a direct sift-backed retrieval adapter owned by paddles.
- [SCOPE-02] Surface concrete retrieval execution stages and delay reasons in user-facing progress events.
- [SCOPE-03] Rename or rewire gatherer configuration and runtime selection so the architecture no longer implies nested autonomous planning.
- [SCOPE-04] Document the direct search boundary, including what paddles owns versus what sift owns.

### Out of Scope

- [SCOPE-05] Reworking sift’s upstream autonomous planner for general-purpose use.
- [SCOPE-06] Broad ranking-quality tuning beyond what is needed to preserve current harness behavior.
- [SCOPE-07] Search cancellation, remote providers, or unrelated network retrieval features.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Planner-driven gatherer turns execute retrieval through a direct sift-backed adapter instead of `sift-autonomous` recursive planning. | SCOPE-01 | FR-01 | test |
| SRS-02 | The direct adapter accepts the query, retrieval mode, strategy, limits, and local context needed by the current paddles harness. | SCOPE-01 | FR-02 | test |
| SRS-03 | Gatherer progress events describe concrete retrieval stages such as initialization, indexing, retrieval, ranking, and completion/fallback. | SCOPE-02 | FR-03 | manual |
| SRS-04 | User-facing progress must not expose autonomous planner action labels like `Terminate` as the primary status for direct retrieval turns. | SCOPE-02 | FR-03 | manual |
| SRS-05 | Runtime configuration and provider labels reflect sift as a retrieval backend rather than an autonomous planner. | SCOPE-03 | FR-04 | test |
| SRS-06 | Documentation explains the direct search boundary, constraints, and ownership split between paddles planning and sift retrieval. | SCOPE-04 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Long-running retrieval emits periodic progress updates throughout execution instead of appearing stalled. | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | The replacement boundary remains local-first and introduces no new network dependency. | SCOPE-01 | NFR-02 | review |
| SRS-NFR-03 | Trace output and summaries remain sufficient to explain why retrieval is slow or fell back. | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
