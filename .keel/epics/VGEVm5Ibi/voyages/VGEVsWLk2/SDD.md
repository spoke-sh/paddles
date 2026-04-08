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

### Target Module Map

```text
apps/web/src/
  app/
    runtime-app.tsx
    runtime-router.tsx
    runtime-shell-layout.tsx
    runtime-tabs.tsx

  chat/
    transcript-pane.tsx
    transcript-message.tsx
    assistant-message.tsx
    plain-message.tsx
    composer.tsx
    composer-chip.tsx
    use-chat-composer.ts
    use-sticky-tail-scroll.ts
    manifold-turn-selection-context.tsx

  store/
    runtime-store.tsx
    runtime-client.ts
    projection-stream.ts
    event-log.ts

  presentation/
    render-document.tsx
    event-row.ts

  routes/
    inspector-route.tsx
    manifold-route.tsx
    transit-route.tsx
```

Voyage one only establishes the `app/`, `chat/`, `store/`, and `presentation/` seams plus any route entry-point rewiring needed to consume them. Route-family internals remain for later voyages.

## Components

- `app/runtime-shell-layout`: owns top-level layout and cross-route shared state.
- `chat/*`: transcript pane, message rendering, composer UI, sticky-tail behavior, manifold turn selection UI hooks.
- `store/*`: bootstrap client, projection stream handling, event-log reduction, and outward-facing store context.

### Shared State Ownership Matrix

| State / Behavior | Owner After Voyage One | Why |
|------------------|------------------------|-----|
| Active trace view | `app/runtime-shell-layout` | Route shell concern shared by chat and route pane. |
| Selected manifold turn | `app/runtime-shell-layout` via `chat/manifold-turn-selection-context` | Transcript clicks select manifold turns across route boundaries. |
| Prompt field text | `chat/use-chat-composer` | Composer-local behavior. |
| Composer paste chips | `chat/use-chat-composer` | Composer-local behavior with no route dependency. |
| Prompt history cursor + draft | `chat/use-chat-composer` | Composer-local history behavior. |
| Sticky-tail scroll state | `chat/use-sticky-tail-scroll` | Transcript viewport concern. |
| Projection snapshot | `store/runtime-store` | Shared runtime data contract consumed by all routes. |
| Event stream accumulation | `store/event-log` | Transport reduction concern, not a shell/render concern. |
| Bootstrap + SSE transport lifecycle | `store/runtime-client` + `store/projection-stream` | Transport concern that should not stay in the UI shell. |

### Migration Sequence

1. Extract render-only transcript/message primitives into `chat/` and `presentation/` without changing behavior.
2. Move composer state transitions and paste/history logic behind `chat/use-chat-composer`.
3. Move sticky-tail behavior behind `chat/use-sticky-tail-scroll`.
4. Move manifold turn selection context out of the monolithic runtime file into `chat/manifold-turn-selection-context`.
5. Split bootstrap/SSE/send-turn transport into `store/runtime-client` and `store/projection-stream`.
6. Reduce streaming event accumulation into `store/event-log` while keeping `useRuntimeStore()` stable for callers.
7. Leave inspector/manifold/transit internals in place until voyage two, but rewire them to import from the new shell/store seams.

## Interfaces

- `useRuntimeStore()` remains the primary UI-facing contract.
- Chat components receive transcript/event state and callbacks from the shell/store boundary rather than calling transport APIs directly.
- Route components continue to receive projection data from the shared store and shared manifold selection from the shell context; voyage one does not let route modules reach into transport primitives directly.

## Data Flow

Bootstrap and SSE updates flow into store modules, are reduced into projection/event state, and then feed the shell. The shell hands transcript/composer state into chat modules and route projections into route modules.

### Invariants For Later Stories

- `useRuntimeStore()` remains the only public store hook until voyage-three support-surface cleanup is complete.
- Transcript-driven manifold selection stays shell-owned even after route extraction because its source of truth lives in chat interactions.
- Composer behaviors already visible to users must remain bit-for-bit compatible through extraction before any support-surface cleanup story is allowed to change structure around them.

## Error Handling

The primary failure mode is behavioral regression during extraction. Detection comes from existing runtime tests plus voyage-specific regressions for prompt history, multiline paste, sticky-tail scroll, and transcript-driven manifold selection.

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Extracted module changes chat/composer behavior | Runtime tests fail | Restore previous behavior before further extraction | Keep the shell-facing contract stable while refactoring internals |
| Store transport split breaks projection updates | Runtime tests/SSE mocks fail | Reconnect the split through a stable store API | Add targeted store/transport tests in the same slice |
