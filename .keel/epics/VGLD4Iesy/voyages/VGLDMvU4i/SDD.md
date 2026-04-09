# Add Adaptive Harness Profiles And Specialist Brains - Software Design Description

> Replace stale provider-shaped heuristics with explicit harness profiles, session-queryable context, and optional specialist brains that preserve the recursive core.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the harness adaptive without giving up its recursive identity. Instead of hiding more heuristics in controller code, the runtime should carry explicit profiles and optional specialist-brain roles that operate against the same session/capability contracts.

## Context & Boundaries

In scope are controller profile semantics, session-queryable context access, and auxiliary brains that still honor the core recursive planner loop. Out of scope are hosted orchestration and wholesale replacement of the existing planner/controller model.

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
| durable session spine | internal boundary | replay, rewind, and context slices | voyage one output |
| capability contracts | internal boundary | decide which profile/brain behaviors are legal | voyage one output |
| hand diagnostics | internal boundary | keep adaptive actions observable | voyage two output |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Profile model | Make steering/compaction/recovery policy explicit and versionable | Stale harness assumptions should be reviewable and replaceable |
| Session access | Query slices from the durable session instead of relying only on destructive compaction | Future models may need different retrieval/replay behavior |
| Specialist brains | Treat auxiliary brains as session-scoped capabilities, not alternate architectures | Preserve the recursive context harness essence |

## Architecture

Profiles sit above the capability/session spines and shape how the controller compacts, retries, or recovers. Optional specialist brains consume the same session and capability contracts and return bounded outputs back into the same recursive runtime.

## Components

- harness-profile contract and resolver
- session-slice query surface
- optional specialist-brain registry/boundary
- verification/docs layer that explains when and why a profile or specialist brain is active

## Interfaces

- profile selection and reporting
- session slice / rewind / replay queries
- specialist-brain invocation contract bounded by existing session/capability interfaces

## Data Flow

During a turn, the controller resolves the active profile from session and capability state, queries the session when it needs replay or compaction input, and optionally invokes specialist brains as bounded helpers that still feed results back into the main recursive loop.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Profile mismatch | active capability/session state cannot satisfy profile | fall back to a safer baseline profile | surface explicit profile downgrade |
| Session slice unavailable | slice query cannot resolve requested context | degrade to broader replay or bounded direct context | record recovery in trace |
| Specialist brain unavailable | capability or invocation fails | keep work inside primary recursive planner | emit explicit fallback event |
