# Unbounded Context Tier Model - Product Requirements

## Problem Statement

Paddles has access to effectively unlimited context across four tiers — inline artifacts (truncated at various char limits), transit streams (full trace records), sift indexes (autonomous retrieval with graph-mode exploration), and the filesystem (workspace files). But these tiers are disconnected: a truncated inline artifact cannot resolve to its full content in transit, related evidence in sift, or source files on disk. The `paddles-artifact://` locator scheme exists but is a bare string with no resolution mechanism.

The tier model needs formalization with traversal semantics so the system can reach any depth on demand. When an artifact is truncated at the inline tier, its locator should specify exactly where the full content lives (transit stream, sift index, filesystem path). Resolution should be lazy and pull-based — components retrieve related context through locators, not by eagerly loading everything into memory.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Formalize the four-tier context model with explicit tier boundaries and traversal semantics | Documented tier model with clear promotion/demotion rules | Architecture documented |
| GOAL-02 | Implement tier traversal: given a locator at any tier, resolve or navigate to content at adjacent tiers | Transit locator resolves to content; content can reference sift evidence or filesystem paths | Cross-tier navigation works |
| GOAL-03 | Integrate tier model into artifact envelope so truncated content always carries a resolvable locator | Every truncated `ArtifactEnvelope` has a typed `ContextLocator` that resolves | No orphaned locators |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Planner loop | Recursive planner navigating evidence across steps | Follow a truncated artifact through tiers to its full content without re-searching |
| Synthesizer | Final answer composer citing sources | Resolve artifact references to authoritative source content |

## Scope

### In Scope

- [SCOPE-01] Formal tier model documentation: inline, transit, sift, filesystem with boundaries and traversal rules
- [SCOPE-02] Tier-aware `ArtifactEnvelope` that carries `ContextLocator` with tier metadata
- [SCOPE-03] Cross-tier navigation: resolve inline->transit and transit->filesystem locators
- [SCOPE-04] Fail-closed degradation: locator resolution fails honestly when a tier is unavailable

### Out of Scope

- [SCOPE-05] Sift-tier locator resolution (requires sift API changes)
- [SCOPE-06] Automatic tier promotion/demotion (future work)
- [SCOPE-07] Cross-session tier traversal

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Document the four-tier model with explicit boundaries: inline (< configurable limit), transit (full records), sift (indexed evidence), filesystem (workspace files) | GOAL-01 | must | Formalizes the implicit tier structure |
| FR-02 | Extend `ArtifactEnvelope` to carry a `ContextLocator` with tier metadata instead of a bare locator string | GOAL-03 | must | Typed locators enable programmatic resolution |
| FR-03 | Implement inline-to-transit resolution: given a truncated artifact's locator, retrieve full content from transit | GOAL-02 | must | Most common tier traversal path |
| FR-04 | Implement transit-to-filesystem resolution: given a transit record referencing a file path, resolve to file content | GOAL-02 | should | Enables full depth navigation |
| FR-05 | Locator resolution fails closed with an explicit error when the target tier is unavailable or the record is missing | GOAL-02 | must | Honest degradation per charter constraint |
| FR-06 | `ContextLocator` includes a `tier: ContextTier` field so consumers know which tier a locator targets | GOAL-01 | must | Enables tier-aware routing decisions |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Resolution is lazy and pull-based — no eager loading of full content at any tier | GOAL-02 | must | Per charter constraint: resolution on demand |
| NFR-02 | Tier boundaries are domain types — no transit-core or sift-core types in paddles ports | GOAL-01 | must | Preserves domain boundary per charter constraint |
| NFR-03 | Local-first: resolution attempts local tiers (inline, transit) before remote tiers (sift) | GOAL-02 | should | Fastest resolution path first |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Tier model docs | Manual: review tier model documentation for completeness | Document reviewed |
| Locator typing | Unit test: construct locators for each tier, verify tier field | Test output |
| Cross-tier resolution | Integration test: truncate artifact, write full to transit, resolve through locator | Test output |
| Fail-closed degradation | Unit test: resolve locator with missing transit stream, verify error | Test output |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Transit read-back is fast enough for on-demand resolution during a turn | May need caching or pre-fetching | Benchmark transit replay on typical record sizes |
| The existing `paddles-artifact://` locator format can be extended with tier metadata | May need a migration for existing locators | Check all locator generation sites |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the tier model be a compile-time enum or a runtime-configurable registry? | Design | Open — enum first for type safety |
| How to handle locators that span tier boundaries (e.g., artifact partially in transit, partially in sift)? | Design | Open — likely one canonical tier per locator |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Four-tier model is documented with explicit boundaries and traversal semantics
- [ ] `ArtifactEnvelope` carries typed `ContextLocator` with tier metadata
- [ ] Inline-to-transit locator resolution works end-to-end
- [ ] Locator resolution fails closed with explicit error when tier is unavailable
<!-- END SUCCESS_CRITERIA -->
