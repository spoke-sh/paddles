# Establish A Turborepo-Driven React Frontend Platform - Product Requirements

## Problem Statement

Paddles splits its public docs and live runtime web UI across a Docusaurus package and a Rust-embedded HTML shell, which blocks coherent frontend architecture, shared testing, and incremental React migration.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Unify frontend ownership under a single Turborepo-managed Node workspace. | Docs and runtime web apps live under one workspace with shared root scripts for lint, test, and build | First voyage |
| GOAL-02 | Establish a real React runtime web application that can progressively replace the current embedded shell. | A tested `apps/web` React app exists with the core route shell and can evolve without ad hoc inline HTML/JS growth | First voyage |
| GOAL-03 | Preserve operator continuity during migration. | Existing backend APIs and current route access continue to function while React adoption proceeds in slices | First voyage |
| GOAL-04 | Make verification credible across the migration boundary. | Workspace quality/test commands exercise Rust, frontend type/lint, frontend unit tests, and browser E2E in one normal path | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Engineer using the live Paddles web UI while debugging or steering turns | A maintainable runtime web application that can evolve quickly without brittle inline shell regressions |
| Maintainer | Engineer changing docs, UI routes, or frontend tooling | One coherent frontend workspace with predictable scripts, tests, and dependency management |

## Scope

### In Scope

- [SCOPE-01] Turborepo workspace scaffolding and root Node package management for frontend apps
- [SCOPE-02] Relocating the existing docs site into the shared workspace without breaking its build, typecheck, or browser tests
- [SCOPE-03] Creating a React runtime web app with tested route scaffolding for `/`, `/transit`, and `/manifold`
- [SCOPE-04] Updating verification and developer workflows so frontend lint/test/build/E2E run through the shared workspace
- [SCOPE-05] Staged cutover work that incrementally replaces the embedded web shell with React-owned surfaces while preserving the Rust backend API
- [SCOPE-06] Documentation updates that accurately describe the migration state and source of truth for runtime web behavior

### Out of Scope

- [SCOPE-07] Replacing the Rust backend or Axum API with a Node backend
- [SCOPE-08] Adopting hosted frontend infrastructure or remote-only build requirements
- [SCOPE-09] Large visual redesign work unrelated to the migration boundary itself
- [SCOPE-10] Pretending full parity is complete before the React app actually owns the routes

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The repository must define a root Turborepo workspace that orchestrates frontend packages through shared `lint`, `test`, and `build` entry points. | GOAL-01, GOAL-04 | must | A monorepo migration is not credible if the workspace is still pieced together with per-folder scripts only. |
| FR-02 | The existing docs site must move under the shared workspace without losing typecheck, build, or browser E2E coverage. | GOAL-01, GOAL-04 | must | The docs app is already React-based and should become a first-class workspace package. |
| FR-03 | The repository must add a new runtime React web app package with React Router route ownership for `/`, `/transit`, and `/manifold`. | GOAL-02 | must | The runtime UI needs a real React boundary before feature migration can proceed safely. |
| FR-04 | The runtime React app must start with automated unit/integration coverage and browser E2E coverage. | GOAL-02, GOAL-04 | must | The user explicitly asked for a well-tested application, not just a toolchain migration. |
| FR-05 | The migration must preserve the current Rust web API surface so the React app can consume the existing session/transcript/forensic/manifold endpoints. | GOAL-03 | must | Preserves local-first runtime behavior and reduces backend risk. |
| FR-06 | Build and developer commands must route through the shared workspace so operators do not need separate ad hoc frontend commands for docs vs runtime UI. | GOAL-01, GOAL-04 | must | One of the main value propositions of the migration is coherent frontend workflow. |
| FR-07 | The staged cutover must keep the current embedded HTML shell alive until equivalent React-owned route behavior exists. | GOAL-03 | should | Avoids breaking operators while the runtime UI is migrated incrementally. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Frontend migration slices must remain local-first and runnable inside the existing Nix development environment. | GOAL-01, GOAL-03 | must | Preserves the repo's local operating contract. |
| NFR-02 | Workspace tests must be deterministic in CI and use committed lockfiles. | GOAL-01, GOAL-04 | must | Node workspace drift is a common failure mode during monorepo migrations. |
| NFR-03 | The React runtime app should not claim feature parity with the embedded shell until route behavior is actually migrated and tested. | GOAL-02, GOAL-03 | must | Avoids documentation drift and false confidence. |
| NFR-04 | The migration should minimize duplicated frontend logic and converge on React-owned modules instead of long-lived shell copies. | GOAL-02 | should | The point of the migration is maintainability, not permanent duplication. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Workspace scaffolding | Root contract tests plus workspace command execution | Story-level Rust tests and command output |
| Docs relocation | Build/typecheck/E2E via shared workspace commands | Story-level test output |
| Runtime React app | App tests plus browser E2E | Story-level test output |
| Cutover integrity | Backend route/API regression tests plus manual UI checks where needed | Story-level evidence and route-specific proofs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current runtime web shell is valuable enough to migrate incrementally instead of replacing wholesale. | We may decide a greenfield React rewrite is cheaper than route-by-route migration. | Validate during the runtime app slice. |
| Turborepo adds useful orchestration rather than unnecessary complexity for two frontend apps. | A plain npm workspace may be sufficient. | Validate by keeping the initial workspace minimal and observable. |
| The Rust backend can continue serving the API while frontend ownership migrates to React. | We may need a deeper delivery boundary change than planned. | Validate during cutover stories. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How should production/static asset delivery work once the React runtime app owns the routes? | Web / infrastructure | Open |
| Which parts of the current embedded shell should migrate first: transcript view, transit, manifold, or route chrome? | Web / product | Open |
| Should the final React runtime app be Vite-based, another SPA build, or SSR-capable? | Web / architecture | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The repo has a root Turborepo workspace that owns both frontend apps through shared scripts
- [ ] The docs app lives inside the workspace and still passes typecheck, build, and browser E2E
- [ ] A tested React runtime app exists with route scaffolding for `/`, `/transit`, and `/manifold`
- [ ] The migration docs clearly distinguish between React-owned surfaces and the remaining embedded shell boundary
<!-- END SUCCESS_CRITERIA -->
