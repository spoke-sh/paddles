# Extract Runtime Shell And Chat Boundaries - Software Design Description

> Break the runtime shell into modular app, chat, and store boundaries without changing transcript, composer, or routing behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage establishes the top-level modular seams for the runtime React application. The root runtime app should become a thin assembler that wires providers and routes together while dedicated chat/app/store modules own transcript rendering, composer behavior, manifold turn selection, and runtime projection transport.

## Context & Boundaries

The work stays inside the React runtime application and its immediate transport layer. Route-specific internals for inspector, manifold, and transit remain in place for later voyages, but they should consume clearer shell and store interfaces created here.

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
| React | library | Component/module decomposition and state ownership | existing workspace dependency |
| TanStack Router | library | Route assembly and shell composition | existing workspace dependency |
| Runtime SSE/bootstrap endpoints | repo contract | Projection bootstrap and live updates | existing runtime web API |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Extract by domain seam, not by tiny atom | App/chat/store modules first | This reduces coupling without producing a shallow component explosion. |
| Keep shell-owned cross-route state explicit | Manifold turn selection stays a shared shell concern | It is intentionally driven by transcript clicks across views. |
| Preserve the existing store API while moving internals | `useRuntimeStore` remains the shell-facing seam | This lowers refactor risk for later route extractions. |

## Architecture

`runtime-app.tsx` should shrink into router/provider assembly. `runtime-shell-layout`, chat modules, and store modules carry the current shell behavior. Store internals separate transport (`fetch`/`EventSource`) from event accumulation and projection state reduction.

## Components

- `app/runtime-shell-layout`: owns top-level layout and cross-route shared state.
- `chat/*`: transcript pane, message rendering, composer UI, sticky-tail behavior, manifold turn selection UI hooks.
- `store/*`: bootstrap client, projection stream handling, event-log reduction, and outward-facing store context.

## Interfaces

- `useRuntimeStore()` remains the primary UI-facing contract.
- Chat components receive transcript/event state and callbacks from the shell/store boundary rather than calling transport APIs directly.

## Data Flow

Bootstrap and SSE updates flow into store modules, are reduced into projection/event state, and then feed the shell. The shell hands transcript/composer state into chat modules and route projections into route modules.

## Error Handling

The primary failure mode is behavioral regression during extraction. Detection comes from existing runtime tests plus voyage-specific regressions for prompt history, multiline paste, sticky-tail scroll, and transcript-driven manifold selection.

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Extracted module changes chat/composer behavior | Runtime tests fail | Restore previous behavior before further extraction | Keep the shell-facing contract stable while refactoring internals |
| Store transport split breaks projection updates | Runtime tests/SSE mocks fail | Reconnect the split through a stable store API | Add targeted store/transport tests in the same slice |
