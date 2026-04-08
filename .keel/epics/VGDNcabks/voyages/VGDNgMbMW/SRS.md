# Build Deterministic Resolver Backbone - SRS

## Summary

Epic: VGDNcabks
Goal: Give the planner a deterministic, cache-backed way to resolve likely workspace entities into authored file targets before it spends edit budget on broad search or malformed patch attempts.

## Scope

### In Scope

- [SCOPE-01] A typed resolver request/result contract for deterministic entity and path lookup.
- [SCOPE-02] A self-discovering workspace index/cache sourced from authored files only.
- [SCOPE-03] Deterministic query handling for exact paths, basename/component names, and symbol-like path hints.

### Out of Scope

- [SCOPE-04] Planner-loop steering changes beyond feeding the resolver output into downstream integration points.
- [SCOPE-05] IDE-fed entity context or language-server-backed semantic resolution.
- [SCOPE-06] Visualization of resolver outcomes outside the minimum artifacts needed for downstream integration.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The domain exposes a deterministic resolver contract that accepts normalized entity/path hints and returns ranked authored-file candidates or explicit miss/ambiguity results. | SCOPE-01 | FR-01 | cargo nextest |
| SRS-02 | The infrastructure can build and reuse a self-discovering workspace index/cache from authored files while respecting `.gitignore` and generated-directory exclusions. | SCOPE-02 | NFR-01 | cargo nextest |
| SRS-03 | Resolver query handling covers exact relative paths, basename/component hints, and symbol-like path fragments without requiring IDE state or LSP services. | SCOPE-03 | FR-01 | cargo nextest |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Resolver indexing and lookup remain local-first and never return paths outside the authored workspace boundary. | SCOPE-02 | NFR-01 | cargo nextest |
| SRS-NFR-02 | Cache invalidation is deterministic enough that changed, added, or removed authored files cannot silently resolve through stale entries across turns. | SCOPE-02 | NFR-02 | cargo nextest |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
