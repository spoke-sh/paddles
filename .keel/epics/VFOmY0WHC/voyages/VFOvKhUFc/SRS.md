# Tier Model And Cross-Tier Locator Resolution - SRS

## Summary

Epic: VFOmY0WHC
Goal: Formalize the four-tier context model with explicit boundaries, extend ArtifactEnvelope with typed locators, and implement cross-tier resolution with fail-closed degradation.

## Scope

### In Scope

- [SCOPE-01] Formal tier model documentation: inline, transit, sift, filesystem with boundaries and traversal rules
- [SCOPE-02] Tier-aware ArtifactEnvelope that carries ContextLocator with tier metadata
- [SCOPE-03] Cross-tier navigation: resolve inline->transit and transit->filesystem locators
- [SCOPE-04] Fail-closed degradation: locator resolution fails honestly when a tier is unavailable

### Out of Scope

- [SCOPE-05] Sift-tier locator resolution
- [SCOPE-06] Automatic tier promotion/demotion
- [SCOPE-07] Cross-session tier traversal

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | ContextTier enum: Inline, Transit, Sift, Filesystem with documented boundaries | SCOPE-01 | FR-01 | test |
| SRS-02 | ContextLocator includes tier: ContextTier field | SCOPE-01 | FR-06 | test |
| SRS-03 | ArtifactEnvelope carries ContextLocator with tier metadata instead of bare locator string | SCOPE-02 | FR-02 | test |
| SRS-04 | Inline-to-transit resolution: given truncated artifact locator, retrieve full content from transit | SCOPE-03 | FR-03 | test |
| SRS-05 | Transit-to-filesystem resolution: given transit record referencing file path, resolve to file content | SCOPE-03 | FR-04 | manual |
| SRS-06 | Resolution returns explicit error with context when target tier is unavailable or record missing | SCOPE-04 | FR-05 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Resolution is lazy and pull-based — no eager loading | SCOPE-03 | NFR-01 | manual |
| SRS-NFR-02 | No transit-core or sift-core types in paddles domain ports | SCOPE-02 | NFR-02 | test |
| SRS-NFR-03 | Local tiers (inline, transit) attempted before remote tiers (sift) | SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
