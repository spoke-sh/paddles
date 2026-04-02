# Steering Signal Manifold Route - Product Requirements

## Problem Statement

The current web UI has a precise forensic inspector, but it does not expose steering signals as a living manifold view that lets operators watch signal accumulation, bleed-off, chamber opacity, and non-linear interaction over time from transit-backed evidence.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make steering signals legible as a dedicated manifold route rather than only as lists or static panels. | Operators can replay a turn and immediately see which signal families accumulated, stabilized, or bled off over time | First voyage |
| GOAL-02 | Keep the expressive manifold tied to exact transit-backed evidence. | A selected chamber or conduit state can always be traced back to underlying influence snapshots, lineage anchors, and forensic artifacts | First voyage |
| GOAL-03 | Support live observation as well as replay. | Active turns update the manifold route without reload and recover correctly from replay if live updates are missed | First voyage |
| GOAL-04 | Preserve the current precise forensic inspector as the exact surface while adding a separate route for systemic visualization. | Operators can move between the manifold route and the forensic inspector without losing selection context | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer or maintainer using the web UI during active or completed turns. | A fast systemic view of how steering signals changed over time while still being able to drill into exact evidence. |
| Maintainer | Engineer debugging planner or synthesizer behavior in the recursive harness. | A route that shows whether signal accumulation actually supports the next judgement instead of hiding the system behind flat lists. |

## Scope

### In Scope

- [SCOPE-01] Transit-backed projection of time-ordered steering signal state suitable for a manifold visualization route
- [SCOPE-02] A dedicated web route and layout for the steering signal manifold, separate from the existing precise forensic inspector
- [SCOPE-03] A chamber/conduit/reservoir state model that maps steering signal families and lineage transitions into expressive visual primitives
- [SCOPE-04] Timeline replay, scrub, and pause controls so operators can inspect manifold state over time
- [SCOPE-05] Live active-turn manifold updates and replay-based recovery after missed events
- [SCOPE-06] Cross-linking from manifold selections back to exact forensic sources and route-to-route navigation
- [SCOPE-07] Foundational and public documentation describing the steering signal metaphor, its limits, and the manifold route

### Out of Scope

- [SCOPE-08] Replacing the existing precise forensic inspector
- [SCOPE-09] TUI parity for the manifold route
- [SCOPE-10] Decorative ambient visualizations that are not tied to inspectable transit-backed signal data
- [SCOPE-11] Hosted telemetry systems or remote visualization services
- [SCOPE-12] Full hypothetical simulation of alternative signals beyond a bounded replay/shadow comparison baseline

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The application/web layer must expose time-ordered manifold replay data derived from transit-backed steering signal snapshots, lineage anchors, and artifact lifecycle state. | GOAL-01, GOAL-02 | must | The route needs authoritative data instead of browser-local invention. |
| FR-02 | The web UI must expose a dedicated steering signal manifold route distinct from the precise forensic inspector. | GOAL-01, GOAL-04 | must | The user explicitly wants a separate route for the systemic visualization. |
| FR-03 | The manifold route must map steering signal families and lineage structure into chambers, conduits, reservoirs, or equivalent expressive visual primitives. | GOAL-01 | must | The route needs a meaningful topology rather than a flat chart. |
| FR-04 | Chamber and conduit visual state must change over time according to influence snapshots, including accumulation, stabilization, supersession, and bleed-off behavior. | GOAL-01, GOAL-03 | must | The route is valuable only if it shows how signals evolve. |
| FR-05 | Operators must be able to replay, pause, and scrub manifold state over time for a selected conversation or turn. | GOAL-01, GOAL-03 | should | Temporal inspection is central to the value of the route. |
| FR-06 | Active turns must stream provisional and final manifold updates without page reload and remain recoverable through replay. | GOAL-03 | must | The route should work during live debugging, not just after completion. |
| FR-07 | Selecting a chamber or conduit state must expose exact underlying sources and support navigation back to the precise forensic inspector. | GOAL-02, GOAL-04 | must | The metaphor must stay accountable to inspectable evidence. |
| FR-08 | Foundational and public docs must describe the manifold route, steering signal semantics, and the metaphorical limits of the visualization. | GOAL-02, GOAL-04 | should | The docs should accurately describe the system operators are using. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Replay data must be sufficient to rebuild manifold state after missed live updates without relying on browser-local repair heuristics. | GOAL-02, GOAL-03 | must | Transit-backed replay remains the authoritative recovery path. |
| NFR-02 | The manifold route must remain usable for long conversations and large turn histories through bounded local rendering and local scrolling behavior. | GOAL-01, GOAL-03 | should | Operators need to inspect long-running sessions without the route collapsing into noise. |
| NFR-03 | Any new visualization dependency must be served locally or vendored and must not introduce mandatory hosted services. | GOAL-01, GOAL-03 | must | Preserves the local-first contract. |
| NFR-04 | The visualization must preserve interpretability by providing a source drilldown for rendered states and avoiding decorative signals with no evidence anchor. | GOAL-02 | must | The metaphor should clarify the system, not lie about it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Manifold replay/live projection | Unit and integration tests around projection payloads, lifecycle state, and replay recovery | Story-level test evidence |
| Web route and interaction model | Browser/manual verification plus HTML contract tests where possible | Story-level proof and manual evidence |
| Documentation accuracy | Review against implementation and route semantics | Story-level review evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Existing forensic/transit data is rich enough to drive a manifold projection with limited new capture work. | We may need a deeper transit-extension slice before the route is truthful. | Validate in the first projection story. |
| The current web surface can host a separate route without forcing a frontend framework rewrite. | We may need to stage the route in a simpler shell first. | Validate during route-shell implementation. |
| A Rube Goldberg or manifold metaphor can stay interpretable if linked back to exact sources. | The route could become too decorative to be useful. | Validate with source drilldown and route-to-route navigation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which local rendering stack best fits the manifold route: SVG/canvas only, WebGL, or a small local 3D helper? | Web / architecture | Open |
| How much temporal smoothing or interpolation can the manifold use before it stops feeling evidence-faithful? | Product / web | Open |
| Should the route default to conversation-wide signal state or begin focused on the active turn? | UX / operator | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The web UI exposes a dedicated manifold route where steering signals accumulate and bleed off over time from transit-backed data
- [ ] A selected manifold state can always be traced back to exact forensic sources and route into the precise inspector
- [ ] Active turns update the manifold route live and recover from replay if updates are missed
- [ ] The documentation explains the manifold metaphor as an evidence-backed steering-signal view rather than literal system physics
<!-- END SUCCESS_CRITERIA -->
