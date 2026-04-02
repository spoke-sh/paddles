# Build Steering Signal Manifold Visualization - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver a mission-scoped epic that defines a dedicated web route for a steering signal manifold view, using transit-backed influence snapshots to show signal accumulation, bleed-off, and chamber opacity over time. | board: VFes0Rhaj |
| MG-02 | Keep the manifold route expressive but accountable by linking every rendered chamber or conduit state back to inspectable forensic sources instead of inventing decorative physics disconnected from evidence. | board: VFes0Rhaj |
| MG-03 | Decompose the work into planned implementation slices that separate projection, route shell, chamber-state modeling, live/replay behavior, and documentation. | manual: epic VFes0Rhaj has a planned voyage with backlog stories covering each boundary |

## Constraints

- The manifold view must be a distinct web route and must not replace the existing precise forensic inspector.
- Transit-backed forensic artifacts and influence snapshots remain the source of truth; the manifold may reinterpret them visually, but it must not fabricate unseen signal state.
- Steering signals are metaphorical guidance, not literal pressure physics. The visualization may use chambers, conduits, opacity, and flow as metaphors, but the UI must preserve source drilldown and exact lineage context.
- The first slice must remain local-first and avoid mandatory hosted telemetry, remote render services, or externally served visualization dependencies.
- The route should support both completed-turn replay and active-turn live updates so operators can use it for forensics and live observation.
- The manifold should stay legible over long conversations and avoid turning the page into a single giant scrolling canvas.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epic VFes0Rhaj is verified and only follow-on visual polish or optional rendering experiments remain
- YIELD to human when route-default placement, metaphor intensity, or major visualization-library tradeoffs need product input
