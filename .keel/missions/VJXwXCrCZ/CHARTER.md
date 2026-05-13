# Unify Recursive Agent Loop Action Selection - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace the pre-loop InitialAction routing split with a unified recursive agent-loop action decision contract, so the first model choice and later choices use the same bounded action vocabulary; preserve direct answers, edit obligations, candidate-file hints, capability-manifest gating, provider support, and documentation while removing planner-as-separate-phase nomenclature from the architecture. | board: VJXwbmekZ |

## Constraints
- Preserve local-first execution and existing provider compatibility; no new network dependency; keep action schema and capability manifest shared across lanes; migrate in TDD slices with parity tests before renames; update foundational docs in the same mission.
## Halting Rules
- Do not halt while the runtime still has a separate InitialAction pre-loop route or docs describe planning as a phase outside the recursive agent loop. Halt when the unified action contract is implemented, all planner lanes use it for first and recursive decisions, edit/direct-answer behavior remains covered by tests, and docs describe model reasoning as bounded recursive agent action selection.
