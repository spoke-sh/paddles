# Trace DAG Visualization - Software Design Description

> Deliver trace graph endpoint and hexagonal railroad SVG visualization

**SRS:** [SRS.md](SRS.md)

## Overview

An SVG railroad visualization is embedded in the chat page, rendering hexagonal
nodes from TurnEvents with edges showing lineage flow. A new
`GET /sessions/:id/trace/graph` endpoint converts the internal `TraceReplay`
structure into a flat JSON representation of nodes, edges, and branches. The
browser-side JS fetches this graph and renders it as an inline SVG with
vertically-flowing hexagonal nodes, color-coded by `TraceRecordKind`. Branch
divergence is shown as parallel swimlanes splitting from the mainline, and merge
records converge lanes back together. As new TraceRecords arrive via SSE, the
SVG updates incrementally.

## Context & Boundaries

- In scope:
  - trace graph JSON endpoint
  - SVG rendering of hexagonal DAG nodes
  - color/label encoding of TraceRecordKind
  - branch swimlane divergence and merge visualization
  - real-time SSE-driven updates
- Out of scope:
  - historical trace browsing
  - interactive editing or annotation
  - 3D or physics layouts

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌───────────┐  ┌──────────┐  ┌──────┐ │
│  │ /trace/   │  │ SVG hex  │  │ SSE  │ │
│  │  graph EP │  │ renderer │  │ feed │ │
│  └───────────┘  └──────────┘  └──────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [TraceReplay]   [chat page host]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| TraceReplay | internal data | Source of trace records, branches, and lineage | current Rust runtime |
| axum server | internal runtime | Hosts the graph endpoint | current Rust runtime |
| Chat page (VFKDlUda0) | sibling voyage | SVG visualization is embedded within the chat HTML | same binary |
| SSE endpoint | internal runtime | Streams TraceRecord events for real-time updates | existing /sessions/:id/events |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Hexagonal SVG nodes | Inline SVG polygons with 6 vertices | Visually distinct from rectangular DOM elements, conveys "turnstep" metaphor |
| Railroad vertical flow | Top-to-bottom layout with swimlanes | Natural reading direction for sequential trace records |
| Color-coded by TraceRecordKind | Distinct fill colors per kind (root, action, tool, checkpoint, merge) | Immediate visual semantics without reading labels |
| Embedded in chat page | Same HTML file, togglable panel | No separate route or build artifact needed |

## Components

### Trace graph handler

Purpose: converts TraceReplay into `{ nodes, edges, branches }` JSON.

### SVG renderer (inline JS)

Purpose: reads graph JSON, computes layout, and renders hexagonal SVG nodes
with edge paths and swimlane positioning.

### Color mapping

| Event Kind | Color | Hex |
|-----------|-------|-----|
| Root | Blue | #58a6ff |
| Planner action | Purple | #bc8cff |
| Tool call | Orange | #d29922 |
| Checkpoint | Green | #3fb950 |
| Merge | Pink | #f778ba |

### SSE listener (JS)

Purpose: on each TraceRecord event, re-fetches the graph endpoint and
triggers an incremental SVG update.

## Interfaces

- `GET /sessions/:id/trace/graph` -- returns JSON:
  ```json
  {
    "nodes": [{ "id": "...", "kind": "action", "label": "...", "branch": 0 }],
    "edges": [{ "from": "...", "to": "..." }],
    "branches": [{ "id": 0, "parent": null }]
  }
  ```
- SSE TraceRecord events -- consumed by the JS listener to trigger re-render.

## Data Flow

1. Browser loads chat page containing the SVG visualization panel.
2. JS fetches `GET /sessions/:id/trace/graph` and receives graph JSON.
3. SVG renderer lays out hexagonal nodes top-to-bottom, assigning swimlane
   X-offsets based on branch metadata.
4. Edges are drawn as SVG path elements connecting node centers.
5. As SSE delivers new TraceRecord events, JS re-fetches the graph and
   incrementally updates the SVG.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Trace graph endpoint returns empty graph | nodes array length == 0 | Display placeholder message in SVG panel | Re-fetch after next SSE event |
| Graph fetch fails | fetch response status != 2xx | Log error, keep existing SVG | Retry on next SSE event |
| Malformed graph JSON | JSON.parse throws | Log to console, skip render update | Continue listening for SSE events |
| SSE connection drops | EventSource onerror | SVG stops updating, display stale indicator | EventSource auto-reconnects |
