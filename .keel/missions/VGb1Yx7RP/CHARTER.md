# Add Mode-Aware Review And Planning Loops - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Introduce explicit collaboration modes and a first-class review lane so Paddles can plan, execute, and inspect changes with different bounded behaviors while keeping one recursive harness underneath. | board: VGb1c1pAR |

## Constraints

- Modes must steer behavior and permissions, not create separate product silos with divergent runtime semantics.
- Review output must stay findings-first and evidence-backed rather than drifting into generic chat summaries.
- Structured user-input requests must be bounded, explicit, and mode-aware so the harness does not fall back to vague conversational clarification loops.
- Mode changes must be visible in traces and operator surfaces so replay explains why the harness behaved differently.

## Halting Rules

- DO NOT halt while planning, execution, and review still share one undifferentiated behavioral contract.
- DO NOT halt while review remains an informal prompt style instead of a first-class workflow with findings-oriented output contracts.
- DO NOT halt while structured clarification requests are unavailable or unconstrained.
- HALT when epic `VGb1c1pAR` is terminal and mode-aware behavior is proven in docs, tests, and operator-facing transcripts.
- YIELD to human only if the desired UX tradeoffs between autonomy and interruption need a product decision.
