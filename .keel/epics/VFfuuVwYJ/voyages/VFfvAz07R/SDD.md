# Bootstrap Turborepo Workspace And React Runtime App - Software Design Description

> Create a shared Node workspace, move the docs app into it, and add a tested React runtime app shell that can begin replacing the embedded web UI.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage creates a real frontend monorepo boundary without pretending the runtime cutover is already finished. The design keeps the Rust backend and embedded HTML shell intact while establishing a Turborepo-managed Node workspace that owns `apps/docs` for Docusaurus and `apps/web` for the new runtime React application.

## Context & Boundaries

### In Scope

- create a root Node workspace and Turborepo pipeline
- move the existing docs app into the shared workspace
- bootstrap a tested React runtime app with route scaffolding
- route `just quality` and `just test` through the shared workspace scripts

### Out of Scope

- replacing the Rust backend with Node
- claiming full runtime cutover from the embedded shell in this slice
- rewriting existing Axum endpoints

```text
┌───────────────────────────────────────────────────────┐
│                 Turborepo Workspace                  │
│                                                       │
│  ┌────────────────────┐   ┌───────────────────────┐  │
│  │ apps/docs          │   │ apps/web              │  │
│  │ Docusaurus site    │   │ React runtime shell   │  │
│  └────────────────────┘   └───────────────────────┘  │
│             \                  /                      │
│              \                /                       │
│               └── root scripts / turbo ─────────────┘│
└───────────────────────────────────────────────────────┘
                       |
                 existing Rust API
                       |
         embedded shell remains live during migration
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Turborepo | Build orchestration | Shared task graph for frontend apps | workspace |
| npm workspaces | Package management | Root-managed frontend dependencies and scripts | Node 20+ |
| Docusaurus | Docs app framework | Existing documentation site under `apps/docs` | current repo version |
| Vite + React | Runtime app framework | New `apps/web` development/build/test boundary | current repo version |
| Rust Axum web API | Local backend | Existing `/sessions`, transcript, forensics, transit, and manifold endpoints | current repo API |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Workspace orchestrator | Turborepo | The user explicitly requested Turborepo and the repo now has enough frontend surface area to justify it. |
| Backend boundary | Keep Rust API unchanged | Minimizes migration risk and preserves local-first runtime behavior. |
| Cutover strategy | Bootstrap React in parallel, keep embedded shell alive temporarily | Avoids breaking current operator workflows before parity exists. |
| Runtime app stack | Vite + React + TypeScript | Fast local iteration and simple SPA route ownership for the first slice. |

## Architecture

The root Node workspace becomes the owning boundary for frontend tooling. `apps/docs` and `apps/web` share root scripts and task orchestration, while the Rust backend continues to own data APIs and live execution. The React runtime app is introduced as a migration target, not yet the only served UI.

## Components

- Root workspace: top-level `package.json`, `turbo.json`, and shared scripts that drive frontend `build`, `lint`, and `test`.
- `apps/docs`: relocated Docusaurus site with its existing documentation and Playwright verification.
- `apps/web`: new React runtime application with React Router, tests, and route shell ownership.
- Rust backend: unchanged API and temporary embedded shell until later cutover stories.

## Interfaces

The voyage does not change backend API contracts. The React runtime app is expected to consume the existing local HTTP/SSE endpoints already exposed by the Rust service.

## Data Flow

1. `just quality` / `just test` enter through root repo commands.
2. Root commands invoke workspace-level Node scripts.
3. Turborepo fans those scripts out to `apps/docs` and `apps/web`.
4. The React runtime app uses the Rust API boundary when runtime integration begins.
5. Until route cutover, the embedded shell remains the live operator surface.

## Error Handling

The main failure modes are workspace path drift, broken docs build/test paths after relocation, and a React app scaffold that is not actually wired into repo verification. We detect these through repo contract tests, workspace command execution, and browser E2E checks.

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Docs relocation breaks build | Workspace build/test failure | Fix moved paths and scripts before sealing | Keep docs app passing through root scripts |
| React app scaffold is not tested | Contract tests or missing app test commands | Add app-level tests before claiming completion | Keep `just` wired to workspace scripts |
| Migration docs overstate cutover | Doc review / contract mismatch | Update docs to state embedded shell remains live | Defer cutover claims to later stories |
