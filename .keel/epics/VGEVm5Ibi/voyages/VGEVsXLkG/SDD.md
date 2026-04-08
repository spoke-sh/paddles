# Stabilize Styling Tests And Fallback Contracts - Software Design Description

> Partition styling, tests, and fallback-shell contracts so the modular React runtime remains evolvable without hidden coupling.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage finishes the modularization by aligning the surrounding support surfaces with the new module boundaries. Styles, tests, and fallback-shell contracts should reinforce the React decomposition instead of dragging the codebase back toward one global surface.

## Context & Boundaries

This work assumes the shell/store and route domains are already modularized. It focuses on the supporting layers that make those module seams maintainable over time.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Modular React runtime routes | repo contract | Styling/test partition targets | outputs of voyages one and two |
| Embedded fallback shell | repo contract | Parity boundary to document and guard | existing runtime release path |
| Vitest/Playwright/Rust HTML contracts | test tooling | Verification of route and fallback behavior | existing repo tooling |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Split support surfaces by feature family | Styles and tests mirror runtime domains | This keeps maintenance local after the refactor lands. |
| Make fallback parity explicit | Document what must stay aligned and guard it with tests | Hidden duplication is the main long-term risk. |
| Prefer shared setup over global test files | Shared fixtures stay reusable without returning to one kitchen-sink test suite | This keeps tests modular and legible. |

## Architecture

Style imports, tests, and fallback contracts become parallel support layers around the modular runtime domains created in earlier voyages.

## Components

- `styles/*`: shell/chat, inspector, manifold, transit, and shared token groupings.
- `test/*`: shared runtime fixtures plus domain-focused test files.
- Fallback-shell contract docs/tests: explicit parity statements and HTML/runtime contract assertions.

## Interfaces

No new runtime API contracts are introduced. The main interface work is internal: style/test/fallback boundaries mirror the module boundaries already exposed by the React runtime.

## Data Flow

Style ownership follows component ownership. Tests consume shared fixtures and target route-local surfaces. Fallback-shell contracts consume the embedded HTML surface and assert only the bounded parity behaviors the team intends to maintain.

## Error Handling

The main failure mode is support-surface drift: styles, tests, or fallback behavior stop matching the modular runtime architecture. Detection comes from route-level tests, embedded-shell contract tests, and docs review in the same slice.

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| CSS split breaks a runtime surface | Visual/unit tests fail | Restore missing imports or relocate shared rules | Keep shared tokens explicit and feature styles local |
| Test split loses coverage | Route/e2e regressions fail or coverage gaps become obvious | Reintroduce missing domain tests in the same slice | Preserve shared setup while moving assertions locally |
| Fallback parity becomes implicit again | Contract docs/tests drift | Re-state parity boundary and update tests | Treat fallback parity as a first-class contract artifact |
