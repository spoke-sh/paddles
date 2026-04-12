# Connect External Capability Fabrics To The Recursive Harness - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Extend the recursive harness with a typed external capability fabric for web search, MCP servers, and connector-backed apps so Paddles can reach Codex-class tool breadth without abandoning its evidence-first local core. | board: VGb1c1XAL |

## Constraints

- Preserve local-first behavior: external capability fabrics must be optional and explicitly negotiated, not mandatory for basic operation.
- Route every external capability through typed ports, evidence artifacts, and execution governance rather than embedding ad-hoc client logic in prompts.
- External results must become first-class evidence with source lineage, availability diagnostics, and honest degradation when a capability is absent or stale.
- Authentication and side-effecting external tools must compose with the same sandbox and approval model as local hands.

## Halting Rules

- DO NOT halt while external capabilities still require bespoke, surface-specific controller logic instead of one negotiated tool fabric.
- DO NOT halt while web, MCP, or connector-backed results bypass evidence capture and trace recording.
- DO NOT halt while unavailable external capabilities fail ambiguously instead of degrading with explicit operator-visible state.
- HALT when epic `VGb1c1XAL` is terminal and the external capability fabric is documented, exercised, and observable across surfaces.
- YIELD to human only if connector scope or external side-effect boundaries require explicit product policy decisions.
