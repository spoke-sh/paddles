# Make Paddles A Recursive In-Context Planning Harness - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Reframe `paddles` around a recursive in-context planning harness where operator memory influences first-pass interpretation, a planner model owns a bounded search/refine loop, and a downstream synthesizer answers from the resulting evidence trace. | board: VFDv1i61H |

## Constraints

- Do not make Keel board semantics first-class in the runtime. Keel remains one workspace evidence domain among many, not a hardcoded turn type.
- Treat `AGENTS.md` and linked foundational docs as interpretation-time context, not merely late-stage prompt decoration.
- The recursive loop must remain bounded by explicit resource budgets, typed or validated actions, and fail-closed local-first behavior.
- Preserve separable planner and synthesizer roles so `context-1`, Sift autonomous planning, and local Qwen variants can be routed independently.
- Foundational docs must distinguish the intended backbone architecture from the current implementation snapshot until the mission is delivered.

## Halting Rules

- DO NOT halt while epic `VFDv1i61H` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFDv1i61H` is verified and `paddles` has a bounded model-owned search/refine loop whose interpretation context includes operator memory and whose final answers are synthesized from recursive evidence.
- YIELD if the recursive planner cannot be expressed behind a safe bounded-action contract without weakening local-first guarantees or making domain-specific board logic first-class.
