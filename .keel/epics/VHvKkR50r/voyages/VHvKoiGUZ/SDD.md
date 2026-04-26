# Remove Npm Audit Vulnerabilities - Software Design Description

> Upgrade or pin web UI dependency versions so npm audit reports zero vulnerabilities while preserving existing frontend and docs behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage refreshes the npm workspace dependency graph and verifies the
result with the same commands used by the repository commit hooks. The primary
implementation surface is package metadata: direct dependency versions in
workspace manifests and the root lockfile.

## Context & Boundaries

The Rust application, runtime behavior, and web UI feature code are out of
scope unless dependency compatibility requires a minimal build fix. npm audit is
the security source of truth for this slice.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| npm | Package manager | Resolve and audit workspace dependencies | `packageManager` in root `package.json` |
| Docusaurus | Docs framework | Serves and builds `@paddles/docs` | Direct docs dependencies |
| Vite/Vitest/Playwright | Web tooling | Build, test, and e2e verification for `@paddles/web` | Direct web dev dependencies |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Audit source | `npm audit` | Matches the vulnerability report in the user request and commit hook output. |
| Compatibility proof | Existing npm scripts | Reuses repository quality gates instead of inventing a special dependency check. |

## Architecture

No runtime architecture changes are planned. The dependency graph remains a
single root npm workspace with `apps/web` and `apps/docs` packages.

## Components

- Root workspace manifest and lockfile: package manager and resolved graph.
- `apps/web/package.json`: web UI direct dependencies and tooling.
- `apps/docs/package.json`: docs direct dependencies and tooling.

## Interfaces

No public API contracts change.

## Data Flow

`npm install` resolves manifests into `package-lock.json`; `npm audit` evaluates
that resolved graph; lint/test/build/e2e scripts verify compatibility.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Audit still reports vulnerabilities | `npm audit` exits non-zero | Inspect remaining advisory chain | Upgrade or pin the responsible package and regenerate the lockfile |
| Dependency update breaks docs or web UI | npm quality gate fails | Keep the failing output as evidence | Apply the smallest compatibility fix or choose a safer dependency version |
