# HTTP Inference And Turn Loop Cleanup Research - Product Requirements

## Problem Statement

Paddles has legacy in-process Sift model inference and planner/synthesizer/gatherer lane concepts mixed into runtime configuration, code, tests, and documentation. Before implementation, the cleanup needs a source-backed migration map that separates HTTP-only model inference, Sift-backed retrieval, turn-loop phase boundaries, compatibility/deprecation choices, tests, and owning docs.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Identify every load-bearing in-process Sift model inference surface before deletion work begins. | Inventory covers source, tests, CLI/config, and docs with cited evidence. | Reviewed migration map |
| GOAL-02 | Define the target HTTP-only inference boundary and separate it from Sift-backed retrieval/indexing. | Recommendation distinguishes model-client transport from retrieval backend concerns. | Reviewed migration map |
| GOAL-03 | Collapse planner, synthesizer, and gatherer as public lane concepts into turn-loop-centered phases without erasing useful internal contracts. | Migration plan names the public concepts to retire, internal phase boundaries to preserve, and tests that protect behavior. | Reviewed migration map |
| GOAL-04 | Identify ADR and owning-document updates required before implementation. | Plan lists the docs and decision records that must change with each implementation slice. | Reviewed migration map |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Runtime maintainer | Engineer responsible for keeping paddles coherent across model providers, local-first operation, and turn-loop behavior. | A safe deletion and migration plan that removes obsolete concepts without breaking shipped behavior. |
| Operator | Person running paddles locally or through HTTP-backed model services. | Configuration and docs that describe the real runtime boundary without legacy Sift-model or lane confusion. |
| Future contributor | Engineer implementing follow-up cleanup slices. | Clear source ownership, test anchors, and ADR/doc guidance before code changes begin. |

## Scope

### In Scope

- [SCOPE-01] Source inventory for Sift model-provider, model-preparation, and in-process inference code paths.
- [SCOPE-02] Source inventory for HTTP provider/model-client seams and provider capability negotiation.
- [SCOPE-03] Source inventory for planner, synthesizer, and gatherer lane configuration, prepared state, ports, prompts, tests, and docs.
- [SCOPE-04] Recommendation for which Sift retrieval/indexing concepts stay, move, or rename separately from model inference.
- [SCOPE-05] Sealed-slice migration plan with test anchors and owning documentation updates.
- [SCOPE-06] ADR recommendation for deleting paddles-owned local model loading.

### Out of Scope

- [SCOPE-07] Deleting Sift model inference or lane APIs before the recommendation is reviewed.
- [SCOPE-08] Replacing Sift-backed retrieval/indexing unless the research explicitly recommends that as a later slice.
- [SCOPE-09] Changing model-provider behavior, CLI flags, configuration files, or docs outside the research artifacts in this slice.
- [SCOPE-10] UI redesign or web runtime changes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Inventory all in-process Sift model inference and model-preparation surfaces with file-level evidence. | GOAL-01 | must | The cleanup cannot safely delete local inference code without knowing every runtime and test dependency. |
| FR-02 | Inventory the HTTP provider/model-client seams that can replace Sift-backed planner and synthesizer execution. | GOAL-02 | must | The migration should build on the existing HTTP adapter boundary instead of inventing a new transport layer. |
| FR-03 | Inventory planner, synthesizer, and gatherer lane concepts across code, configuration, prompts, tests, and docs. | GOAL-03 | must | Lane collapse needs a precise distinction between public concepts to retire and internal phase contracts to preserve. |
| FR-04 | Produce a sealed-slice migration plan with test anchors, compatibility/deprecation choices, and docs/ADR ownership. | GOAL-02, GOAL-03, GOAL-04 | must | Implementation should proceed in reviewed slices rather than a broad cleanup diff. |
| FR-05 | Present the recommendation for human review before runtime implementation begins. | GOAL-04 | must | The user explicitly asked to review recommendations before action. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Keep this epic behavior-neutral until the recommendation is accepted. | GOAL-01, GOAL-02, GOAL-03 | must | Research should not accidentally change runtime behavior. |
| NFR-02 | Preserve local-first constraints in the target architecture by allowing local HTTP-hosted models while removing paddles-owned model loading. | GOAL-02 | must | HTTP-only inference should simplify ownership without requiring remote hosted providers. |
| NFR-03 | Keep evidence citations tied to concrete source files, board artifacts, or owning docs. | GOAL-01, GOAL-03, GOAL-04 | must | The cleanup plan must be auditable and not based on stale architectural memory. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Sift model inventory | Targeted `rg` scans and source review | Story evidence linking source paths and findings |
| Lane/turn-loop inventory | Targeted `rg` scans and source review | Story evidence linking lane configuration, ports, prompts, and docs |
| Migration recommendation | Manual review against bearing VJZ034dF2 and voyage report | Human-reviewed recommendation before implementation |
| Board integrity | `keel doctor` and `keel flow --scene` | Clean board after research decomposition |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| HTTP-backed local model servers are an acceptable replacement for paddles-owned local model loading. | The cleanup may remove a local-first capability the project still wants in-process. | Confirm through human review and likely ADR before implementation. |
| Sift retrieval/indexing is separable from Sift model inference. | A single deletion slice could remove needed retrieval behavior or leave confusing names behind. | Inventory Sift provider, registry, agent, and gatherer surfaces separately. |
| Planner/synthesizer/gatherer can collapse as public concepts while preserving internal phase contracts. | Over-collapse could make the turn loop harder to test or reason about. | Map ports and tests before recommending renames/deletions. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should deleting in-process local model loading require a new ADR? | Epic owner | Open |
| Should `sift-direct` remain the default retrieval backend after `sift` is removed as a model provider? | Epic owner | Open |
| Should existing planner/gatherer CLI flags become compatibility aliases before removal? | Epic owner | Open |
| Which naming survives internally: planner/synthesizer ports, turn-loop phases, model-client roles, or another vocabulary? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Inventory identifies every model-provider and model-loading surface that a future implementation must touch.
- [ ] Inventory identifies every public lane concept and the internal turn-loop phase or helper it should map to.
- [ ] Migration plan lists ordered sealed slices, red/green test anchors, docs, and ADR needs.
- [ ] Human receives recommendations before implementation begins.
<!-- END SUCCESS_CRITERIA -->
