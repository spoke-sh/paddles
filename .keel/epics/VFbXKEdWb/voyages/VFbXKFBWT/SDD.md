# Transit Artifact Capture And Inspector Projection - Software Design Description

> Make transit the exact source of truth for model exchange, context lineage, and force snapshots, then project that data into a dense web forensic inspector with a secondary interactive overview.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends transit from a trace-oriented execution log into an exact forensic artifact substrate for the web UI. The application layer records exact assembled context, redaction-safe provider envelopes, raw provider responses, rendered outputs, lineage edges, and force snapshots into transit. A web-facing projection layer then replays those artifacts by conversation/turn and streams provisional updates during active turns. The browser renders that projection in a dense context-lineage-first inspector, with a precise 2D artifact pane as the primary surface and a secondary overview for topology, force, and shadow comparison.

## Context & Boundaries

### In Scope

- richer transit artifact records and lineage metadata
- force snapshot and contribution capture
- web replay/live projection APIs for forensic inspection
- dense web inspector and secondary overview

### Out of Scope

- TUI inspector parity
- changes to planner or synthesis behavior beyond recording their artifacts
- hosted telemetry backends or remote collaboration
- purely decorative globe-style views disconnected from transit artifacts

```
┌─────────────────────────────────────────────────────────────────────┐
│                            This Voyage                             │
│                                                                     │
│  execution path ──> transit artifact recorder ──> forensic replay   │
│         │                     │                         │             │
│         │                     ├─ lineage/force data     ├─ live deltas│
│         │                     └─ raw/rendered artifacts └─ web views  │
│                                                                     │
│                  secondary overview  +  precise 2D inspector         │
└─────────────────────────────────────────────────────────────────────┘
          ↑                                         ↑
   model/provider adapters                     web forensic UI
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Transit trace model and recorder | internal | Durable storage for exact forensic artifacts and lineage metadata | current |
| Application service | internal | Canonical capture, replay, and live update seams | current |
| Web adapter and static UI | internal | Browser projection and rendering of forensic artifacts | current |
| Optional overview visualization library | local web dependency | Secondary topology/force visualization only if needed beyond SVG/canvas | TBD |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Source of truth | Transit becomes the exact forensic artifact substrate | Prevents the web UI from reverse-engineering raw prompts, forces, or provider envelopes from lossy projections |
| Primary navigation | Context lineage first | Matches the product goal and makes sequence understandable across turns, model calls, steps, and artifacts |
| Raw vs rendered inspection | Store both and let the UI toggle | Operators need exact payloads for forensics and readable views for comprehension |
| Secret handling | Redact auth headers and obvious secret patterns before browser projection | Preserves operator visibility without exposing credentials in the browser |
| Overview role | Secondary to the precise 2D inspector | Keeps exact debugging reliable while still allowing richer structural visualization |
| Shadow baseline | Previous artifact in lineage for v1 | Gives a deterministic comparison point without needing full hypothetical simulation on day one |
| Contribution estimates | Heuristic/controller-derived in v1 | Provides useful operator signal quickly without waiting for model-generated explanations |

## Architecture

1. The existing execution path emits richer transit artifact records at context assembly and provider exchange seams.
2. Transit stores raw artifacts, rendered artifacts, lineage edges, and force snapshots in coherent sequence.
3. A forensic projection layer replays those records by conversation/turn and emits live updates for provisional/final transitions.
4. The web adapter exposes projection endpoints/streams for the browser.
5. The browser renders a two-part inspector:
   - a precise 2D lineage inspector for exact navigation and raw/rendered inspection
   - a secondary overview for topology/force/shadow context

## Components

`ForensicTransitRecorder`
: Extends transit capture with exact model exchange artifacts, redacted provider envelopes, rendered outputs, lineage edges, and force snapshots.

`ForceContributionEstimator`
: Computes or packages source-contribution estimates for the applied forces at each relevant step.

`ForensicReplayProjection`
: Rebuilds an exact artifact/lineage view from transit records for a conversation or turn and supports recovery after missed live updates.

`ForensicUpdateStream`
: Emits provisional, superseded, and final artifact updates during active turns for the browser.

`WebForensicInspector`
: Dense context-lineage-first browser surface for exact/raw vs rendered inspection, force panels, and coherent sequence navigation.

`InspectorOverviewRenderer`
: Secondary visualization layer for topology, force shape, and shadow comparison. The rendering technology remains abstract so SVG/canvas or a local 3D library can be chosen later.

## Interfaces

Candidate internal interfaces:

- `record_forensic_artifact(turn_id, artifact_record)`
- `record_force_snapshot(turn_id, force_snapshot)`
- `replay_forensic_projection(conversation_id, turn_filter) -> ForensicProjection`
- `subscribe_forensic_updates(conversation_id) -> stream`

Candidate web contracts:

- `GET /sessions/{id}/forensics` for dense replay payloads
- `GET /sessions/{id}/forensics/turns/{turn_id}` for focused inspection
- SSE event(s) for provisional/final forensic artifact updates
- projection payloads with:
  - ordered artifact records
  - lineage edges
  - raw/rendered forms
  - force snapshots and contribution estimates
  - artifact lifecycle state (`provisional`, `superseded`, `final`)

## Data Flow

1. A turn begins and the application assembles context for planner or synthesizer/model exchange.
2. The recorder stores exact context artifacts and a redaction-safe provider request envelope in transit.
3. Provider responses arrive and transit stores raw response data plus normalized/rendered outputs.
4. In parallel, force snapshots and lineage edges are recorded for the same turn/model-call sequence.
5. The forensic projection layer replays the stored records into a browser-friendly lineage graph and emits live provisional/final deltas.
6. The web UI updates the dense 2D inspector and the secondary overview from the same projected data.
7. If the browser misses live updates, it rebuilds from forensic replay instead of DOM repair or heuristic re-synthesis.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Artifact capture misses an exact provider/request seam | Replay tests or adapter integration tests show missing or malformed records | Keep the existing UI path working, extend the recorder at the right seam, and block declaring transit authoritative | Add the missing transit record type and re-run replay validation |
| Secret redaction removes too much or too little | Redaction tests fail or manual inspection reveals missing fidelity or leaked headers | Tighten pattern rules and separate raw-capture from browser-projected views | Reproject from stored transit with corrected redaction policy if needed |
| Live browser updates race or drop provisional artifacts | Projection signature differs from replay state or sequence gaps appear | Mark the browser projection stale and trigger replay rebuild | Recover from conversation/turn-scoped forensic replay |
| Secondary overview library introduces too much complexity or local asset friction | Review shows bundle/serving mismatch with the local-first web stack | Fall back to SVG/canvas overview while preserving the same projection model | Keep the 2D inspector primary and treat the overview as replaceable |
