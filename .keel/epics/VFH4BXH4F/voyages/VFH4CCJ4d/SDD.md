# Trace Contract And Recorder Port - Software Design Description

> Define a paddles-owned trace contract and recorder boundary that can capture recursive planner turns, branches, tool activity, and artifacts in a form that later maps cleanly onto embedded transit lineage semantics.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a recorder boundary that sits beside the existing
interactive event stream rather than replacing it.

The intended flow is:

1. define a `paddles`-owned trace contract with stable lineage identifiers,
2. refactor planner and turn state so runtime structure exists independently of
   UI rendering,
3. emit typed trace records into a `TraceRecorder` port,
4. keep large payloads behind artifact envelopes instead of assuming inline
   strings forever,
5. map the contract into embedded `transit-core` roots, appends, branches, and
   checkpoints,
6. preserve transcript rendering as a projection over the same structured data.

## Context & Boundaries

- In scope:
  - `paddles`-owned trace entities and stable ids
  - recorder-ready planner and turn state
  - `TraceRecorder` port plus noop/in-memory adapters
  - artifact-envelope support for large turn payloads
  - embedded `transit-core` adapter and replay proof
  - foundational docs that explain the boundary clearly
- Out of scope:
  - requiring `transit` server mode
  - replacing the interactive TUI
  - building a full shared trace service
  - leaking raw `transit` record or kernel types into domain contracts

```
┌─────────────────────────────────────────────────────────────┐
│                         This Voyage                         │
│                                                             │
│  Planner / Gatherer / Tool Runtime                          │
│     ↓ produces structured trace entities                    │
│  Paddles Trace Contract                                     │
│  (root | branch | tool event | artifact | checkpoint)       │
│     ↓                                                       │
│  TraceRecorder Port                                         │
│     ↓                    ↘                                  │
│  Noop / In-Memory       Transcript Rendering Projection     │
│     ↓                                                       │
│  Embedded Transit Recorder Adapter                          │
│     ↓                                                       │
│  transit-core Root / Append / Branch / Replay              │
└─────────────────────────────────────────────────────────────┘
          ↑                                     ↑
     Local-first runtime                  Operator-visible UI
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `transit-core` | library | Embedded lineage-aware storage backend for the first durable recorder adapter | current local workspace revision |
| existing `paddles` planner/gatherer/tool runtime | internal runtime | Produces the structured trace entities that the recorder will persist | current repo implementation |
| existing TUI / turn event sink | internal runtime | Continues to render user-visible action streams from structured runtime state | current repo implementation |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Recorder boundary is separate from transcript rendering | Introduce `TraceRecorder` alongside `TurnEventSink` | UI rows are not durable truth |
| Trace contract is `paddles`-owned | Map into `transit` instead of exposing `transit` types in the domain | Keeps the runtime storage-agnostic |
| Embedded `transit-core` comes before server/client modes | Start with local recording and replay proof | Matches `paddles` local-first posture |
| Large payloads use artifact envelopes | Preserve inline summaries plus logical refs | Prevents recorder growth from bloating the hot path |

## Architecture

The voyage touches five cooperating layers:

1. `TraceContract`
   Defines roots, branches, tool events, artifacts, and checkpoints with stable
   ids and lineage references.

2. `RuntimeTraceProjection`
   Refactors planner loop state and turn execution so the runtime can emit
   structured trace data before any renderer or storage adapter formats it.

3. `TraceRecorderPort`
   Accepts typed trace records and isolates durable recording from transcript
   rendering.

4. `ArtifactEnvelopeBoundary`
   Keeps large payloads behind logical references plus inline metadata.

5. `EmbeddedTransitRecorder`
   Maps the `paddles` trace contract into embedded `transit-core` operations
   and supports replay proofs.

## Components

- `TraceContract`
  Purpose: represent the durable lineage of a `paddles` turn independently of
  how it is rendered in the UI.

- `RuntimeTraceProjection`
  Purpose: derive structured trace entities from planner actions, gatherer
  results, tool calls, branch decisions, and synthesis checkpoints.

- `TraceRecorderPort`
  Purpose: accept typed trace entities and support `noop`, in-memory, and
  embedded-storage implementations.

- `ArtifactEnvelopeBoundary`
  Purpose: represent large prompts, outputs, and traces through stable logical
  references plus inline metadata.

- `EmbeddedTransitRecorder`
  Purpose: project the trace contract into `transit-core` roots, appends,
  branches, and checkpoints with replayable local proofs.

## Interfaces

- `TraceRecord`
  The `paddles`-owned durable trace entity used throughout the recorder path.

- `TraceRecorder`
  A port for appending trace records, recording branches/checkpoints, and
  replaying prior local traces.

- `ArtifactEnvelope`
  A logical reference plus inline metadata for large turn payloads.

- `TransitTraceMapper`
  An adapter that translates `TraceRecord` entities into embedded
  `transit-core` stream operations.

## Data Flow

1. The runtime executes a user turn through planner, gatherer, tools, and
   synthesis.
2. Each meaningful transition projects into structured `TraceRecord` entities.
3. The same structured state feeds transcript rendering through the existing
   event/UI projection.
4. The `TraceRecorder` port accepts the durable trace entities.
5. `noop` and in-memory adapters support early verification without storage.
6. The embedded `transit-core` adapter maps roots, appends, branches, and
   checkpoints into lineage-aware local streams.
7. Replay proofs read the stored trace back and verify lineage and payload
   reconstruction.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Recorder adapter is unavailable | Capability check or constructor failure | Fall back to noop recording and emit a visible recorder warning | Preserve turn execution without pretending durability succeeded |
| Trace projection depends on renderer-only strings | Contract tests or runtime assertions fail | Refactor the projection to emit structured data first | Keep UI and recorder paths derived from the same structure |
| Large payloads exceed inline budgets | Size checks during trace projection | Store summaries inline and move the full payload behind an artifact envelope | Preserve replay and auditability without bloating records |
| Embedded transit mapping loses lineage semantics | Replay proof or adapter tests fail | Fix the mapper before claiming durable support | Keep the trace contract storage-agnostic and local |
