# Signal Manifold Route And Chamber Projection - Software Design Description

> Project transit-backed steering signals into a dedicated web route where chambers, conduits, and opacity reveal how signal influence accumulates, interacts, and changes over time.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds an expressive steering-signal manifold route beside the existing precise forensic inspector. The manifold route does not invent a new source of truth. Instead, it projects transit-backed influence snapshots, lineage anchors, and artifact lifecycle state into an operator-facing topology of chambers, conduits, valves, and reservoirs whose opacity and fill change over time. The browser treats that topology as a metaphorical view over exact data and preserves drilldown back to the forensic route.

## Context & Boundaries

### In Scope

- manifold replay/live projection derived from stored transit artifacts
- a dedicated web route and layout for the manifold view
- chamber/conduit state mapping from steering signal families and lineage transitions
- time controls and live update behavior
- source drilldown and route-to-route navigation
- documentation describing the metaphor and its evidence basis

### Out of Scope

- replacing the precise forensic inspector
- TUI parity
- hosted telemetry or remote rendering services
- decorative physics with no evidence anchor

```
┌─────────────────────────────────────────────────────────────────────┐
│                            This Voyage                             │
│                                                                     │
│ transit forensics ──> manifold projection ──> manifold route        │
│         │                     │                    │                 │
│         │                     ├─ signal states     ├─ chambers       │
│         │                     ├─ lineage anchors   ├─ conduits       │
│         │                     └─ live deltas       └─ time controls  │
│                                                                     │
│      precise forensic route <──── route-to-route source drilldown   │
└─────────────────────────────────────────────────────────────────────┘
          ↑                                           ↑
   stored transit artifacts                    web operator surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Transit forensic artifacts and influence snapshots | internal | Authoritative replay/live source for manifold state | current |
| Application service/web projection layer | internal | Builds manifold-friendly replay/live payloads | current |
| Existing static web UI | internal | Hosts the dedicated route and source drilldown affordances | current |
| Optional local visualization helper | local dependency | Helps render the manifold view if SVG/canvas alone is insufficient | TBD |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Source of truth | Reuse transit-backed forensic artifacts and influence snapshots | Keeps the manifold faithful to the system rather than inventing detached visual state |
| Route model | Add a dedicated route instead of replacing the precise inspector | The manifold is expressive and systemic; the inspector remains the exact surface |
| Visual metaphor | Chambers, conduits, reservoirs, and valves encode signal families and state transitions | Matches the desired Rube Goldberg / manifold feel without claiming literal physics |
| Temporal semantics | Accumulation, stabilization, supersession, and bleed-off derive from influence snapshots and lifecycle state | Gives the metaphor real meaning over time |
| Accountability | Every manifold state links back to exact sources and inspector context | Prevents the visualization from becoming decorative or deceptive |
| Dependency posture | Prefer locally served SVG/canvas; only add a local helper if necessary | Preserves local-first constraints and keeps the route lightweight |

## Architecture

1. The application/web layer replays transit-backed influence snapshots and lineage markers into a manifold projection model.
2. The projection model groups steering signals into chamber and conduit state keyed by turn, step, lineage, and time.
3. The web adapter exposes that projection to a dedicated manifold route alongside live update streams for active turns.
4. The browser renders the manifold topology, time controls, and a detail/source pane for the current selection.
5. The detail pane links back to the precise forensic route for exact artifact inspection.

## Components

`ManifoldProjectionBuilder`
: Projects replay/live transit data into time-ordered manifold states, including chamber accumulation, conduit activity, and bleed-off transitions.

`SignalTopologyMapper`
: Defines how steering signal families, lineage branches, and lifecycle markers map onto chambers, conduits, reservoirs, and valves.

`ManifoldRouteShell`
: Dedicated web route that owns the layout for the manifold canvas, timeline controls, selection pane, and navigation back to the precise inspector.

`ManifoldTimelineController`
: Coordinates replay, pause, scrub, and active-turn progression across the manifold state history.

`ManifoldSourceDrilldown`
: Displays the exact influence snapshot and source anchors for the selected state and routes the operator into the forensic inspector when needed.

## Interfaces

Candidate internal interfaces:

- `replay_manifold_projection(conversation_id, turn_filter) -> ManifoldProjection`
- `subscribe_manifold_updates(conversation_id) -> stream`
- `project_signal_topology(signal_snapshot, lineage_anchor) -> ManifoldNodeState`

Candidate web contracts:

- dedicated manifold route in the static web app
- replay endpoint(s) for manifold projection payloads
- SSE event(s) for provisional/final manifold state updates
- payload fields for:
  - time-ordered state frames
  - chamber/conduit topology
  - current and previous influence snapshots
  - selection-to-source anchors
  - lifecycle flags (`provisional`, `superseded`, `final`)

## Data Flow

1. Transit stores or already exposes exact influence snapshots, lifecycle state, and lineage anchors for the turn.
2. The manifold projection builder folds those records into time-ordered state frames.
3. The topology mapper assigns state frames to chambers and conduits representing steering signal families and lineage transitions.
4. The route shell loads replay state, subscribes to live updates, and hands the frames to the timeline controller.
5. The timeline controller advances or scrubs the visible frame and updates chamber opacity, fill, and conduit activity.
6. Selecting a chamber or conduit opens its source drilldown and offers navigation to the precise forensic inspector for exact artifact review.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Transit data is insufficient to build truthful manifold frames | Projection tests fail or source drilldown cannot resolve exact anchors | Block full route rollout and extend projection/capture seams first | Rebuild from richer replay once the missing fields exist |
| Live updates drift from replay state | Frame signatures differ or a provisional/final transition is missing | Mark the route stale and trigger replay rebuild | Recover from replay without relying on browser-local repair |
| Visual metaphor becomes too decorative | Review/manual verification cannot map a rendered state back to evidence | Simplify topology or strengthen drilldown affordances | Keep the exact inspector as the source-of-truth escape hatch |
| Rendering cost grows too high on long conversations | Manual verification or tests show sluggish scrub or repaint behavior | Clamp frame density, virtualize detail lists, and reduce derived animation work | Rebuild frames incrementally and fall back to simpler rendering primitives |
