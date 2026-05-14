# Inventory Legacy Inference And Lane Surfaces - SRS

## Summary

Epic: VJZ0tpZQJ
Goal: Produce the reviewed source inventory and migration recommendation for HTTP-only model inference and turn-loop-centered phase cleanup before implementation begins.

## Scope

### In Scope

- [SCOPE-01] Inventory in-process Sift model-provider and model-preparation surfaces.
- [SCOPE-02] Inventory HTTP provider/model-client seams and provider capability surfaces.
- [SCOPE-03] Inventory planner, synthesizer, and gatherer lane concepts across code, tests, CLI/config, prompts, and docs.
- [SCOPE-04] Produce a migration recommendation with sealed slices, red/green test anchors, compatibility choices, ADR needs, and owning docs.

### Out of Scope

- [SCOPE-05] Runtime implementation changes.
- [SCOPE-06] Deleting or renaming Sift model, lane, or gatherer code.
- [SCOPE-07] Changing CLI flags, configuration semantics, or foundational docs before recommendation review.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Record a source-backed inventory of all Sift model-provider, model-preparation, and in-process inference surfaces that future implementation must migrate or delete. | SCOPE-01 | FR-01 | manual review |
| SRS-02 | Record a source-backed inventory of HTTP provider/model-client seams that can become the sole inference boundary. | SCOPE-02 | FR-02 | manual review |
| SRS-03 | Record a source-backed inventory of planner, synthesizer, and gatherer lane concepts and map each one to a public concept to retire or an internal turn-loop phase/helper to preserve. | SCOPE-03 | FR-03 | manual review |
| SRS-04 | Produce a migration recommendation with ordered sealed slices, test anchors, compatibility/deprecation handling, owning docs, and ADR needs before implementation starts. | SCOPE-04 | FR-04 | manual review |
| SRS-05 | Present the migration recommendation for human review before runtime implementation starts. | SCOPE-04 | FR-05 | manual review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Keep the voyage behavior-neutral; only `.keel` research artifacts may change during this slice. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-01 | git diff review |
| SRS-NFR-02 | Preserve local-first architecture in the recommendation by routing local model execution through HTTP-hosted model services rather than paddles-owned loading. | SCOPE-02, SCOPE-04 | NFR-02 | manual review |
| SRS-NFR-03 | Tie every recommendation to concrete source, board, or doc evidence. | SCOPE-04 | NFR-03 | manual review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
