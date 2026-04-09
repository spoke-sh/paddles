# Define Durable Session And Capability Interfaces - Software Design Description

> Make the session and capability surfaces the stable interfaces around the recursive harness so provider improvements do not force harness rewrites.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines the meta-harness seams that should outlast any particular provider integration: a durable session object and a negotiated capability surface. The implementation work in later stories should make those seams operational without changing the recursive planner/controller essence.

## Context & Boundaries

In scope are the recorder/session boundary, replay and context-resolution semantics, and the capability descriptors that planner/render/tool paths consume. Out of scope are execution-hand lifecycle refactors and adaptive profile policies; those land in later voyages.

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
| `transit-core` | library | Persistent local session/trace spine | existing embedded engine |
| provider adapters | internal boundary | Consume negotiated planner/render/tool-call capabilities | existing runtime adapters |
| context resolver | internal boundary | Expose slice/replay semantics to later harness layers | existing domain port |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Session durability | Make the durable session the default runtime object, not optional metadata | Recovery and replay need to outlive any particular harness process |
| Capability resolution | Negotiate behavior from capabilities instead of provider-name branches | Model behavior changes faster than interface contracts |
| Trace visibility | Keep new session/capability state visible through existing trace surfaces | Generalization should not make the harness opaque |

## Architecture

The voyage adds two stable layers around the recursive loop:

- session spine: recorder, replay, checkpoint, and slice interrogation
- capability spine: shared descriptors for planning, rendering, and tool-calling behavior

Provider adapters and later hand/profile layers consume those spines instead of re-deriving behavior ad hoc.

## Components

- `TraceRecorder` and recorder adapters: persist and replay task/turn lineage
- session query boundary: exposes wake/replay/slice/checkpoint semantics for higher-level harness code
- capability descriptors: provider/model-resolved runtime capabilities consumed by planner and rendering paths
- trace projection surfaces: keep session/capability states operator-visible

## Interfaces

- session lifecycle: wake, replay, resume from checkpoint, interrogate slice
- capability lifecycle: resolve at boot, carry through request handling, surface through diagnostics
- trace visibility: project session/capability state into existing runtime events and forensic/transit views

## Data Flow

Boot resolves recorder policy and capability descriptors. During a turn, runtime events append into the session spine, later slices replay from that durable record, and planner/render logic reads capability descriptors instead of branching on provider strings.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Recorder unavailable | capability/boot check fails | fail closed or degrade honestly with explicit trace diagnostics | swap to bounded fallback only where policy allows |
| Session slice missing | replay/query returns no record | return explicit missing-session error | re-anchor from last checkpoint or full replay |
| Capability mismatch | provider/model does not satisfy requested behavior | reject or degrade via negotiated fallback | update capability descriptors rather than forking controller logic |
