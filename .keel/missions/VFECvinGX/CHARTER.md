# Replace Heuristic Routing With Model-Directed Action Selection - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace heuristic top-level turn routing with a model-directed next-action contract where interpretation context from `AGENTS.md` and linked foundational docs informs the first bounded action, and the controller enforces validation, budgets, and safe execution. | board: VFECyWLL6 |

## Constraints

- Do not encode Keel or any other repository domain as a first-class runtime intent. The harness must stay general-purpose across workspace evidence domains.
- The first non-trivial routing decision should come from a model-selected bounded action schema, not from controller string heuristics.
- `AGENTS.md` and linked foundational docs must be available before first action selection, not merely later prompt decoration for synthesis.
- The controller must remain authoritative for schema validation, resource budgets, allowlists, and fail-closed local-first behavior.
- Foundational docs must distinguish the target model-directed contract from the current transitional implementation until the epic is delivered.

## Halting Rules

- DO NOT halt while epic `VFECyWLL6` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFECyWLL6` is verified and `paddles` chooses first bounded actions through a constrained model contract informed by interpretation context instead of heuristic top-level routing.
- YIELD if removing heuristic top-level routing would weaken local-first guarantees, observability, or safe bounded execution.
