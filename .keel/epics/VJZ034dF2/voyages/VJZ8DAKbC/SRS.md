# Migrate Provider Preferences To Turn Runtime Config - SRS

## Summary

Epic: VJZ034dF2
Goal: Replace lane-shaped provider preferences with turn-runtime model-client preferences, use Ollama as the canonical local HTTP example, and keep legacy config readable only for migration.

## Scope

### In Scope

- [SCOPE-01] Introduce turn-runtime model-client preference naming and data shape.
- [SCOPE-02] Read legacy `runtime-lanes.toml` or lane-shaped settings only as migration input.
- [SCOPE-03] Write new provider preferences only in the turn-runtime shape.
- [SCOPE-04] Use `ollama:<model>` as the canonical local HTTP provider example in docs and migration hints.
- [SCOPE-05] Preserve credential and provider availability behavior for HTTP providers.

### Out of Scope

- [SCOPE-06] Removing Sift retrieval/indexing preferences.
- [SCOPE-07] Deleting Sift inference adapter files.
- [SCOPE-08] Renaming all internal planner/synthesizer/gatherer ports.
- [SCOPE-09] Selecting a fixed Ollama model name as a default.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | New runtime preference state uses turn-runtime, model-client, action-selection, final-rendering, and retrieval terminology rather than planner/synthesizer/gatherer lanes. | SCOPE-01 | FR-05 | automated config tests |
| SRS-02 | Legacy lane-shaped config remains readable for migration, but new writes use only the turn-runtime preference shape. | SCOPE-02, SCOPE-03 | FR-05 | automated config persistence tests |
| SRS-03 | Local HTTP provider documentation and examples use `ollama:<model>` without selecting a fixed model name. | SCOPE-04 | FR-05 | doc checks |
| SRS-04 | HTTP provider credential and availability rules remain provider-specific and fail closed when credentials are required. | SCOPE-05 | FR-05 | automated credential tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Preference migration is deterministic and preserves user intent without silently changing providers. | SCOPE-02, SCOPE-03 | NFR-02 | migration fixture tests |
| SRS-NFR-02 | Docs explain precedence and file ownership in `CONFIGURATION.md`. | SCOPE-04 | NFR-03 | doc review |
| SRS-NFR-03 | Each implementation story starts with a failing test or doc check before runtime behavior or owning docs are changed. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | story evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
