# Substitute Residual Reasoning Heuristics With Model-Judged Interpretation And Retrieval - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace the remaining reasoning-heavy controller heuristics in `paddles` with constrained model-judged interpretation, fallback, and retrieval decisions so the recursive harness stops thinking on behalf of the model while preserving controller-owned safety constraints, budgets, validation, and fail-closed behavior. | board: VFJ5rdPZP |

## Constraints

- Preserve controller ownership of safety rails: allowlists, budgets, deterministic execution, path validation, and fail-closed behavior stay in Rust.
- Do not introduce new repository-specific intents or hardcoded board-specific routing to replace the heuristics being removed.
- Keep `AGENTS.md` as the only hardcoded interpretation root; additional guidance should remain model-derived from the referenced document graph.
- Prefer constrained structured model passes over free-form prose when replacing routing, fallback, interpretation, or retrieval heuristics.
- Keep the runtime local-first and bounded. If a model decision is invalid, recovery must remain explicit and observable rather than silently pretending success.
- Update foundational docs so the controller-versus-model boundary is explicit and consistent with the delivered code.

## Halting Rules

- DO NOT halt while epic `VFJ5rdPZP` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFJ5rdPZP` is verified and the remaining reasoning-heavy heuristics on the primary harness path have been replaced with model-judged equivalents without weakening controller safety constraints.
- YIELD if removing a heuristic would require weakening local-first guarantees, controller validation, safe command boundaries, or deterministic fail-closed behavior.
