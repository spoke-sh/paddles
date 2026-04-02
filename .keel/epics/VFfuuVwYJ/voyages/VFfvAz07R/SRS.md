# Bootstrap Turborepo Workspace And React Runtime App - SRS

## Summary

Epic: VFfuuVwYJ
Goal: Create a shared Node workspace, move the docs app into it, and add a tested React runtime app shell that can begin replacing the embedded web UI.

## Scope

### In Scope

- [SCOPE-01] Create a root Turborepo workspace with shared package scripts for `build`, `lint`, and `test`.
- [SCOPE-02] Relocate the existing Docusaurus site into the shared workspace without losing build, lint, or E2E coverage.
- [SCOPE-03] Bootstrap a TypeScript React runtime web app under the shared workspace with route ownership for `/`, `/transit`, and `/manifold`.
- [SCOPE-04] Add workspace-aware verification so `just quality` and `just test` drive frontend checks through the shared root entry points.

### Out of Scope

- [SCOPE-05] Full production cutover from the embedded Rust HTML shell to the React runtime app.
- [SCOPE-06] Rewriting backend Axum endpoints or moving the API to Node.
- [SCOPE-07] Visual redesign or runtime feature expansion beyond the migration boundary.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The repository must define a root Node workspace and `turbo.json` that orchestrate frontend `build`, `lint`, and `test` tasks. | SCOPE-01 | FR-01 | automated |
| SRS-02 | The current docs app must live under the shared workspace and keep passing typecheck/build/browser checks through workspace commands. | SCOPE-02 | FR-02 | automated |
| SRS-03 | The repository must add a new React runtime app package with route scaffolding for `/`, `/transit`, and `/manifold`. | SCOPE-03 | FR-03 | automated |
| SRS-04 | The runtime React app must include automated unit/integration coverage and browser E2E coverage from the first slice. | SCOPE-03, SCOPE-04 | FR-04 | automated |
| SRS-05 | The first slice must preserve the Rust backend API surface and current embedded shell while the React app is bootstrapped. | SCOPE-03, SCOPE-04 | FR-05 | automated + manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The shared frontend workspace must run inside the existing Nix shell and CI environment without ad hoc local prerequisites. | SCOPE-01, SCOPE-04 | NFR-01 | automated |
| SRS-NFR-02 | The migration slice must document that the embedded shell remains the runtime source of truth until React cutover work is complete. | SCOPE-04 | NFR-03 | automated + manual |
| SRS-NFR-03 | The workspace layout should minimize long-lived duplication and make later route migration into React straightforward. | SCOPE-01, SCOPE-03 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
