# Reimagine Trace Inspector As Narrative Machine - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that reimagines the transit trace and forensic inspector as one simpler narrative “turn machine”, so operators can understand how a turn moved through planning, steering, tools, and outcome without navigating multiple panes, modes, and raw record lists. | board: VGGIor3dC |

## Constraints

- Use the steering gate manifold as the interaction benchmark: a strong primary stage, time-first navigation, and detail-on-selection instead of dense always-on diagnostics.
- Preserve raw trace fidelity and internal debuggability behind an explicit internals mode or bounded drill-down path; simplification must not discard underlying evidence.
- Prefer one coherent machine metaphor across transit and forensic surfaces instead of parallel metaphors, duplicate selectors, or surface-specific jargon.
- Keep the runtime local-first and repo-owned. Do not introduce IDE-fed context, remote visualization services, or non-repo dependencies to support the redesign.
- Treat the current trace graph and forensic projections as inputs to reinterpret, not as the UI model that must be exposed directly.
- Maintain route, test, and fallback-shell integrity unless a later human decision explicitly approves reducing fallback parity.

## Halting Rules

- DO NOT halt while epic `VGGIor3dC` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VGGIor3dC` is complete and the narrative-machine trace surfaces are backed by board-linked implementation evidence.
- YIELD to the human if the remaining decision requires product direction on how much raw/internal trace detail should stay visible by default versus move behind an explicit internals mode.
