# Define Narrative Machine Model And Shared Projection - Software Design Description

> Define the simplified Rube Goldberg machine mental model, moment projection, and interaction contract so transit and forensic views share one causal vocabulary.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines the abstraction layer that the later UI work depends on. Instead of rendering trace nodes and forensic records directly, the runtime will project them into “machine moments” that describe what the ball did at that point in the turn: entered, was inspected, got diverted, jammed, replanned, acted on a tool, or exited into an outcome.

## Context & Boundaries

In scope is the semantic model and selection contract shared by transit and forensic surfaces. Out of scope is the full visual rewrite itself. The design should let both routes share one mental model before either route is rebuilt.

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
| `trace_graph` projection | Internal runtime data | Supplies temporal node ordering and branch/diverter structure | existing |
| `forensics.turns` projection | Internal runtime data | Supplies record-level payloads, signals, and artifact lineage | existing |
| Route tests and selector tests | Internal verification | Guard the new shared machine-moment contract | existing |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Shared abstraction | Introduce “machine moments” as the primary UI model | This is the smallest concept that tells a coherent story without exposing raw storage primitives directly. |
| Shared selection state | Standardize on selected turn, selected moment, and optional internals mode | The current multi-axis selection model is a large source of operator confusion. |
| Raw evidence access | Preserve raw ids/payloads as detail metadata, not default geometry | Operators still need internals, but they should not dominate the first-read narrative. |

## Architecture

`trace_graph` + `forensics.turns` -> shared projection/selectors -> route-specific machine stages -> optional internals drawers.

## Components

- Shared machine-moment selectors: derive moments, kinds, labels, temporal order, and raw evidence back-links.
- Shared vocabulary/copy contract: defines labels like diverter, jam, spring return, output bin, and force.
- Shared selection model: determines how both routes select and inspect a moment without route-specific mode sprawl.

## Interfaces

- Machine moment:
  Purpose: the operator-facing unit rendered by transit and forensic routes.
  Required fields: moment id, turn id, temporal index, machine kind, summary label, narrative detail, linked raw trace ids, and optional steering-force summary.
- Selection contract:
  Required fields: selected turn id, selected moment id, show internals flag.

## Data Flow

1. Runtime store exposes current `trace_graph` and `forensics.turns`.
2. Shared selectors normalize those structures into ordered machine moments.
3. Transit and forensic routes consume the same moments.
4. A selected moment opens route-specific detail, which can still reference raw record ids/payloads.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| A trace node or forensic record does not map cleanly into a machine moment | Selector contract tests or fallback mapping logic | Render a generic “unknown machine part” moment with raw ids intact | Refine the projection contract without losing operator visibility |
| Route surfaces diverge in vocabulary or moment semantics | Route tests and shared-copy contract tests | Fail closed in tests before shipping inconsistent language | Keep all labels and mappings in shared helpers |
