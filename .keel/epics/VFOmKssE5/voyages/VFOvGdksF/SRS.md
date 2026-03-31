# Context Locator And Transit Resolution - SRS

## Summary

Epic: VFOmKssE5
Goal: Define ContextLocator type and implement transit read-back resolution so context artifacts can be addressed and resolved across tiers.

## Scope

### In Scope

- [SCOPE-01] ContextLocator enum type with variants for inline, transit, sift, and filesystem addresses
- [SCOPE-02] Transit read-back: resolve a transit locator to artifact content via LocalEngine replay
- [SCOPE-03] Integration with build_planner_prior_context() to resolve truncated artifacts on demand

### Out of Scope

- [SCOPE-04] Sift-tier or filesystem-tier locator resolution
- [SCOPE-05] Replacing existing factory-time wiring
- [SCOPE-06] Cross-task or cross-session context resolution

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | ContextLocator enum with Inline, Transit { task_id, record_id }, Sift { index_ref }, Filesystem { path } variants | SCOPE-01 | FR-01 | test |
| SRS-02 | ContextLocator implements Serialize and Deserialize for persistence | SCOPE-01 | FR-01 | test |
| SRS-03 | ContextResolver port trait with async resolve(locator) -> Result<String> method | SCOPE-01 | FR-04 | test |
| SRS-04 | TransitContextResolver implements ContextResolver using TransitTraceRecorder replay | SCOPE-02 | FR-02 | test |
| SRS-05 | ArtifactEnvelope locator field accepts ContextLocator instead of bare string | SCOPE-01 | FR-03 | test |
| SRS-06 | PlannerLoopContext carries an optional ContextResolver for on-demand artifact resolution | SCOPE-03 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Resolution is lazy — only on explicit request, not eagerly on all artifacts | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | No transit-core types appear in ContextLocator or ContextResolver | SCOPE-01 | NFR-03 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
