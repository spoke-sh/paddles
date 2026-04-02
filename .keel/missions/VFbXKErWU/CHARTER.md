# Build Web Forensic Transit Inspector - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Make transit the exact source of truth for web forensic inspection, including exact assembled context, redaction-safe provider envelopes, raw model responses, rendered outputs, force snapshots, and lineage edges. | board: VFbXKEdWb |
| MG-02 | Deliver a context-lineage-first web inspector that lets operators navigate coherent raw and rendered artifacts, inspect forces by default, and observe provisional active-turn state. | board: VFbXKEdWb |
| MG-03 | Leave the work decomposed into planned implementation slices that separate transit capture, projection APIs, dense 2D inspection, secondary overview visualization, and live provisional streaming. | manual: epic VFbXKEdWb has a planned voyage with backlog stories covering each boundary |

## Constraints

- Transit must become the forensic source of truth; the web UI should project from stored transit artifacts instead of reconstructing raw content heuristically.
- This mission is web-only. Do not expand the forensic inspector to the TUI in the first slice.
- Preserve local-first operation and avoid introducing mandatory external services for artifact capture, replay, or visualization.
- Keep the precise 2D inspector primary and treat any polar/3D overview as a secondary visualization layer.
- Exact provider request envelopes may be captured, but authentication headers and obvious secrets must be redacted before browser exposure.
- Active-turn artifacts may be provisional and later superseded; the design must preserve both lineage and clear provisional/final labeling.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epic VFbXKEdWb is verified and only follow-on polish or optional visualization experiments remain
- YIELD to human when product direction is needed for default inspector placement, shadow-baseline semantics beyond previous-lineage comparison, or overview-library tradeoffs
