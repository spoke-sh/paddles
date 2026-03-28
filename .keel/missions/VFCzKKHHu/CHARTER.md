# Integrate Sift Autonomous Retrieval Planning - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Add a local autonomous retrieval-planning lane so Paddles can use Sift's bounded planner to decompose multi-hop repository investigation into iterative evidence gathering before final synthesis. | board: VFCzL9KKd |

## Constraints

- Preserve the current synthesizer lane as the default response path for casual chat, direct coding help, and deterministic workspace actions.
- Treat Sift autonomous planning as an evidence-first gatherer lane, not a replacement for the final answer synthesizer.
- Default the autonomous gatherer to a local heuristic planner strategy first; model-driven planner profiles must stay optional and capability-gated.
- Surface planner trace, stop reasons, retained artifacts, and fallback causes in typed runtime outputs or verbose logs so operators can audit routing decisions.
- Keep the existing context-gathering architecture extensible so autonomous Sift planning complements, rather than replaces, the experimental `context-1` boundary.

## Halting Rules

- DO NOT halt while epic `VFCzL9KKd` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFCzL9KKd` is verified and Paddles can route decomposition-worthy prompts through a local autonomous gatherer lane with observable planner trace and safe fallback.
- YIELD if upstream Sift autonomous APIs require a planner/model contract that cannot be represented behind Paddles' typed gatherer boundary without weakening local-first or fail-closed guarantees.
