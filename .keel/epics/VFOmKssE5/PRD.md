# Transit-Native Context Addressing - Product Requirements

## Problem Statement

Context components in paddles (evidence budgets, artifact envelopes, thread summaries, operator memory, planner loop state) are wired together at factory-assembly time through `MechSuitService` and `PlannerLoopContext`. The only runtime discovery mechanism is `build_planner_prior_context()`, which manually assembles interpretation, recent turns, planner steps, and pending branches into a flat `Vec<String>`. Components cannot find or navigate to related context at runtime — they only see what the orchestrator explicitly passes them.

Transit already provides the primitives needed: `LocalEngine` supports streams, branches with lineage metadata, checkpoints, and replay. The `TransitTraceRecorder` already maps conversation threads to transit branch streams. But today transit is write-only from paddles' perspective: records go in via `engine.append()`, nothing reads them back during a turn except full replay for `ConversationReplayView`. Components need addressable transit-backed locators so any context artifact can resolve related context through transit lineage rather than manual orchestrator assembly.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define a `ContextLocator` type that can address any context artifact across tiers (inline, transit, sift, filesystem) | Locator type compiles and can represent all four tier addresses | Type exists with serialization |
| GOAL-02 | Implement transit read-back: resolve a locator to its full artifact content from transit streams during a turn | A truncated inline artifact's locator can be resolved to its full content from the transit engine | Single-artifact resolution works |
| GOAL-03 | Wire locator resolution into the planner loop so the planner can request full context for any truncated artifact | Planner prior context can include resolved artifacts alongside flat string assembly | Planner loop uses locators |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Planner loop | The recursive planner accumulating evidence across steps | Navigate from a truncated artifact or summary to its full content without re-gathering |
| Turn orchestrator | The MechSuitService assembling context for each turn phase | Discover related context artifacts through lineage rather than manual wiring |

## Scope

### In Scope

- [SCOPE-01] `ContextLocator` enum type with variants for inline, transit, sift, and filesystem addresses
- [SCOPE-02] Transit read-back: resolve a transit locator to artifact content via `LocalEngine::replay()`
- [SCOPE-03] Integration with `build_planner_prior_context()` to resolve truncated artifacts on demand

### Out of Scope

- [SCOPE-04] Sift-tier or filesystem-tier locator resolution (future epics)
- [SCOPE-05] Replacing the existing factory-time wiring pattern (this extends, not replaces)
- [SCOPE-06] Cross-task or cross-session context resolution

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Define `ContextLocator` enum with `Inline { content }`, `Transit { task_id, record_id }`, `Sift { index_ref }`, `Filesystem { path }` variants | GOAL-01 | must | Establishes the universal addressing type for all context tiers |
| FR-02 | Implement `resolve_transit_locator()` that reads a specific record from a transit stream by task_id and record_id | GOAL-02 | must | Enables read-back from transit during a turn |
| FR-03 | Extend `ArtifactEnvelope` to carry a `ContextLocator` instead of a bare `paddles-artifact://` string | GOAL-01 | must | Connects existing truncation mechanism to typed locators |
| FR-04 | Add a `ContextResolver` port trait with `resolve(locator) -> Result<String>` | GOAL-02 | must | Decouples resolution from specific storage backends |
| FR-05 | Wire `ContextResolver` into `PlannerLoopContext` so the planner can resolve truncated artifacts | GOAL-03 | should | Enables the planner to pull full context on demand |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Locator resolution is lazy — only performed when explicitly requested, not eagerly on all artifacts | GOAL-02 | must | Prevents loading full artifact content into memory unless needed |
| NFR-02 | Transit read-back adds no latency to the default turn path when no resolution is requested | GOAL-02 | must | Default performance unchanged |
| NFR-03 | `ContextLocator` preserves the paddles domain boundary — no transit-core types leak through | GOAL-01 | must | Maintains architectural separation per charter constraint |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Locator type | Unit test: construct locators for each tier variant, verify serialization round-trip | Test output |
| Transit resolution | Integration test: write artifact to transit, resolve via locator, compare content | Test output |
| Planner integration | Manual: run multi-step planner turn, verify truncated artifacts can be resolved | Session observation |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Transit `LocalEngine::replay()` can efficiently read individual records by position | May need indexed lookup instead of sequential replay | Profile replay performance on typical task sizes |
| `ArtifactEnvelope` locator field migration is backward-compatible | May need a migration step for existing serialized envelopes | Check serialization format |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should locator resolution be synchronous or async? | Implementation | Open — transit replay is sync today but could become async |
| How to handle locator resolution failure (missing stream, corrupted record)? | Implementation | Open — likely return error with degraded content |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `ContextLocator` type exists with four tier variants and serializes correctly
- [ ] A truncated artifact's transit locator can be resolved to its full content
- [ ] The planner loop can request full context for any truncated artifact via the resolver
- [ ] No transit-core types appear in paddles domain ports
<!-- END SUCCESS_CRITERIA -->
