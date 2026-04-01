# Direct Sift Retrieval Boundary - Software Design Description

> Replace the nested sift-autonomous planner path with direct sift-backed retrieval, expose concrete retrieval-stage progress, and leave paddles as the sole recursive planner.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the nested `sift-autonomous` planner from paddles’ gatherer boundary and replaces it with a direct sift-backed retrieval adapter. Paddles remains responsible for choosing when to search, refine, branch, or stop. Sift becomes an execution engine that performs indexing, lexical/hybrid retrieval, ranking, and snippet production while reporting concrete progress stages back through the existing event sink.

## Context & Boundaries

In scope is the gatherer boundary used after paddles has already chosen a search action. Out of scope is any attempt to preserve sift’s internal autonomous planning loop in the user-facing execution path. The direct adapter should consume the query and retrieval settings chosen by paddles, execute sift library calls, and surface low-level retrieval progress without inventing a second planner.

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
| `sift` crate | library | direct indexing, retrieval, ranking, snippets, and progress signals | workspace git dependency |
| paddles event sink / `TurnEvent` | internal | deliver progress and summary updates to TUI and web | current domain contract |
| paddles planner loop | internal | remains the only owner of recursive search/refinement decisions | current application layer |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Planning ownership | Keep recursive planning in paddles only | Avoid duplicate hidden planning and confusing nested traces |
| Sift role | Use sift as a retrieval backend, not as an autonomous planner | Produces a cleaner architectural split and more controllable UX |
| Progress model | Surface execution stages instead of planner internals | Users need to know what work is happening and why it is slow |
| Compatibility | Rewire config/provider names to describe retrieval semantics | Reduce future confusion and make traces self-explanatory |

## Architecture

`PlannerAction::Search` remains a paddles decision. The application layer routes that request into a gatherer adapter that builds a direct sift retrieval request instead of an autonomous search plan. The adapter emits `GathererSearchProgress` events from sift execution stages and returns evidence bundles plus trace metadata that describe retrieval work, not nested planner decisions.

## Components

### Direct retrieval gatherer adapter

Consumes `ContextGatherRequest`, translates it into sift library retrieval calls, and returns evidence bundles to paddles without invoking sift autonomous planning.

### Progress translation layer

Maps sift execution callbacks or stage observations into `GathererSearchProgress` events with stable, user-facing labels such as `initializing`, `indexing`, `retrieving`, and `ranking`.

### Configuration/runtime selector

Removes or renames provider paths that imply autonomous planning and ensures runtime selection describes sift-backed retrieval accurately.

### Documentation boundary

Updates mission/voyage artifacts and user-facing docs so future changes preserve the split: paddles plans, sift retrieves.

## Interfaces

The gatherer boundary continues to accept `ContextGatherRequest` and emit `ContextGatherResult`. The change is internal to the adapter choice and trace semantics. `TurnEvent::GathererSearchProgress` remains the user-facing progress carrier, but its detail text should be framed around retrieval execution stages rather than autonomous planner actions.

## Data Flow

1. Paddles planner selects `search` or `refine`.
2. Application layer builds a `ContextGatherRequest` with query, retrieval mode, strategy, and limits.
3. Direct sift adapter executes indexing/retrieval/ranking as needed.
4. Adapter emits progress events that describe the active retrieval stage and available ETA or reason for unknown ETA.
5. Adapter returns retained evidence and trace metadata to paddles.
6. Paddles decides whether to answer, refine again, branch, or stop.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Direct sift retrieval cannot initialize | adapter construction or first retrieval call fails | emit initialization failure summary and return gatherer error | planner fallback or explicit error path |
| ETA is unavailable for a stage | sift progress callback omits estimate | show `eta: unknown` plus current stage/reason | continue periodic updates until estimate appears or work completes |
| Retrieval returns no retained artifacts | empty evidence bundle | emit clear summary including zero-result reason or fallback stage | allow paddles planner to refine or stop |
| Legacy config still points at autonomous naming | config parse or runtime selection mismatch | normalize to direct retrieval provider label or fail with clear message | migration story updates config defaults and aliases |
