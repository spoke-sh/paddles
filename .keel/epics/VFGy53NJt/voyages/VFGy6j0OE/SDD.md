# Graph Retrieval Through The Gatherer Boundary - Software Design Description

> Upgrade the Sift gatherer path so Paddles can use bounded graph-mode autonomous search, preserve graph episode state and branch/frontier metadata, and surface that richer context through the recursive planning harness without sacrificing local-first safety.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage pulls the latest upstream `sift` graph runtime into `paddles` and
keeps it behind the existing gatherer boundary.

The intended flow is:

1. upgrade `sift` to the graph-capable revision,
2. extend gatherer/planning config so autonomous retrieval can run in `linear`
   or `graph` mode,
3. execute graph-mode retrieval through the Sift autonomous gatherer adapter,
4. map graph episode/frontier/branch metadata into typed `paddles` evidence and
   structured turn-trace data with stable identifiers,
5. keep the recursive planner and synthesizer architecture generic while giving
   operators a richer view of recursive retrieval work.

## Context & Boundaries

- In scope:
  - latest upstream `sift` graph/branching runtime
  - graph-mode selection in gatherer/planning config
  - typed graph evidence/telemetry mapping with recorder-friendly stable ids
  - recursive planner use of graph-capable gatherers
  - foundational docs and proof artifacts
- Out of scope:
  - Keel-specific graph intents
  - unbounded autonomy
  - remote-only planner requirements
  - direct `transit` integration in this slice
  - a full cross-tool unified resource graph in one slice

```
┌──────────────────────────────────────────────────────────────┐
│                         This Voyage                          │
│                                                              │
│  Planner / Recursive Loop                                    │
│     ↓ emits graph-capable gather request                     │
│  Gatherer Config                                              │
│  (mode=linear|graph, strategy, profile, budgets)             │
│     ↓                                                        │
│  Sift Autonomous Gatherer Adapter                            │
│     ↓                                                        │
│  Sift search_autonomous(mode=graph)                          │
│     ↓                                                        │
│  Graph Episode / Frontier / Branch State                     │
│     ↓                                                        │
│  Typed Paddles Evidence + Structured Turn Trace              │
│     ↓                                                        │
│  Recursive Harness / Synthesizer Handoff                     │
└──────────────────────────────────────────────────────────────┘
          ↑                                   ↑
      Local-first models                 Operator-visible traces / future recorder
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | library | Supplies bounded linear/graph autonomous search, graph episode DTOs, and planner traces | latest upstream `main` revision at implementation time |
| local Qwen planner/synth lanes | internal runtime | Continue to own top-level action selection and final synthesis | current candle-backed runtime |
| operator memory loader | internal runtime | Continues to shape first action selection and recursive work | existing repo implementation |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Keep graph mode behind the gatherer boundary | Do not create a graph-specific top-level turn intent | Preserves the generic recursive harness |
| Preserve typed graph metadata, not raw upstream structs | Map graph episode/frontier/branch state into `paddles` DTOs with stable machine-readable ids | Avoids leaking `sift` internals through the domain and keeps later recorders from depending on UI prose |
| Graph mode is configurable and bounded | Use config/planner hints plus budgets, not free-form autonomy | Maintains local-first fail-closed behavior |
| Default UX must show graph work | Extend the event stream instead of hiding graph state in debug-only logs | Operators need to see the extra recursive work |
| Do not couple graph telemetry to terminal-only strings | Preserve structured trace data first, then render it into the operator UX | Keeps the path open for future durable recording and replay |
| Keep future recording embedded-first | Leave room for an embedded `transit-core` recorder instead of assuming a network server dependency | Matches `paddles` local-first runtime posture |

## Architecture

The voyage touches four cooperating layers:

1. `GraphModeConfig`
   Extends gatherer/planning configuration so requests can choose linear or
   graph autonomous retrieval.

2. `SiftGraphGathererAdapter`
   Calls the latest upstream `sift` autonomous runtime in graph mode and
   captures graph episode state.

3. `GraphEvidenceMapper`
   Translates graph episode/frontier/branch metadata into typed `paddles`
   planner/evidence DTOs and structured turn-trace data.

4. `RecursiveHarnessIntegration`
   Lets the existing model-directed planner loop request graph-capable gatherer
   work without adding repository-specific routing logic.

## Components

- `GraphModeConfig`
  Purpose: express `linear` vs `graph` autonomous retrieval and optional
  planner profile selection.

- `SiftGraphGathererAdapter`
  Purpose: select `AutonomousSearchMode::Graph` when requested and execute the
  bounded graph runtime through the supported upstream `sift` API.

- `GraphEvidenceMapper`
  Purpose: preserve branch/frontier/episode state, graph stop reasons, and
  branch-local retained evidence as domain-friendly metadata with stable ids
  that a later recorder can persist directly.

- `RecursiveHarnessIntegration`
  Purpose: reuse the graph-capable gatherer path from the existing recursive
  planner loop and synthesizer handoff.

## Interfaces

- `PlannerConfig`
  Extended with an autonomous retrieval mode and optional planner profile.

- `ContextGatherRequest`
  Carries graph-capable planning config into the gatherer boundary.

- `PlannerTraceMetadata`
  Extended or paired with typed graph summary fields so graph episode/frontier
  state remains visible without leaking raw upstream types.

- structured turn-trace payloads
  Extended with concise graph-mode summaries and stable ids that the default
  transcript/event stream can render without becoming the source of truth.

## Data Flow

1. The top-level planner selects a search/refine action.
2. The recursive harness lowers that action into a `ContextGatherRequest` with
   bounded planning config.
3. The Sift autonomous gatherer chooses `linear` or `graph` mode from that
   config.
4. The upstream `sift` runtime executes bounded graph search and returns
   planner traces, turns, retained artifacts, and graph episode state.
5. `paddles` maps those results into typed evidence items, graph-aware planner
   metadata, structured trace payloads, and operator-visible turn events.
6. The synthesizer consumes the resulting evidence bundle without needing to
   know about raw upstream graph DTOs.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Upstream `sift` graph API drifts under the new dependency | Compile or adapter tests fail | Fix the adapter/config mapping before shipping | Keep the dependency lift and adapter changes in the same slice |
| Graph mode is requested but unavailable/invalid | Capability checks or graph runtime errors surface | Emit a visible fallback and degrade to linear or unsupported behavior honestly | Preserve local-first bounded execution |
| Raw graph episode state is too large or too coupled to expose directly | Evidence/event payloads become noisy or leaky | Map into typed summary DTOs and keep only the useful branch/frontier/stop fields | Maintain a stable `paddles` domain boundary |
| Graph trace payloads become too large for inline-only evidence or telemetry | Evidence or event records grow beyond reasonable local budgets | Keep summaries inline and preserve room for later external artifact refs rather than forcing all payloads into terminal-visible strings | Stay compatible with a later artifact-envelope recorder |
| Graph telemetry overwhelms the default transcript | Event rendering becomes noisy | Summarize graph state concisely and leave detailed proof artifacts to docs/tests | Preserve operator readability |
