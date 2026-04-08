# Split Route Surfaces Into Domain Modules - Software Design Description

> Move inspector, manifold, and transit into dedicated domain modules with localized state and view composition.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns each runtime route into its own domain module. The route entry files should mostly compose local hooks, selectors, and presentation components instead of owning all state, geometry, and UI markup inline.

## Context & Boundaries

The shell/store boundary from voyage one is assumed to exist. This voyage only restructures route-local logic for inspector, manifold, and transit, keeping their runtime data contracts unchanged.

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
| Runtime shell/store modules | repo contract | Provide route data and callbacks | voyage one outputs |
| runtime-helpers selectors | repo module | Existing helper logic to relocate/co-locate as needed | current repo state |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Split by route family | Inspector/manifold/transit each become a local mini-surface | This aligns ownership with how maintainers reason about the product. |
| Co-locate local hooks and selectors | Route-local state machines move with their route | This prevents another catch-all helper file from forming. |
| Preserve DOM/test hooks intentionally | Keep IDs and data attributes stable unless updated in the same slice | Test churn should track conscious contract changes. |

## Architecture

Each route gets its own folder or module group. Route entry points read projection data from the shared store, then compose local hooks/selectors/presentation subcomponents.

## Components

- `inspector/*`: overview cards, nav, record list, detail pane, selection helpers.
- `manifold/*`: stage shell, viewport, playback hook, camera hook, gate-field helpers, readout cards.
- `transit/*`: toolbar, board container, trace node rendering, layout/pan-zoom hook.

## Interfaces

No external API changes are introduced. The route modules continue to consume the existing projection shapes from `runtime-types.ts`.

## Data Flow

Projection data enters via the shared store, then route-local selectors derive the focused state the view needs. User interactions update local route state only, except for shell-owned shared selections already defined in voyage one.

## Error Handling

The main risks are breaking route-local interactions or silently changing test hooks. Detection comes from route-specific tests and runtime e2e coverage that already exercise the product surfaces.

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Route split breaks focus/selection behavior | Route tests fail | Restore route-local selectors/hooks before further moves | Keep data contracts unchanged while moving code |
| Camera/playback or transit pan/zoom behavior drifts | Interaction tests fail | Rebind extracted hooks to preserve existing behavior | Add route-local regression coverage during extraction |
