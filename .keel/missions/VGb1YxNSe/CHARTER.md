# Generalize Subagents Into A Collaborative Multi-Agent Runtime - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Generalize Paddles' existing subagent and threading foundations into a collaborative multi-agent runtime with explicit delegation roles, ownership boundaries, and wait/close semantics that remain auditable inside the recursive harness. | board: VGb1c2DBj |

## Constraints

- Build on the existing thread-lineage and specialist-brain foundations instead of introducing hidden out-of-band worker state.
- Delegation must stay role-based and contract-based rather than provider-specific or purely prompt-conventional.
- Each delegated agent must use the same execution governance, evidence, and trace surfaces as the parent harness.
- Parent turns retain integration ownership; workers may advance slices, but they do not silently rewrite global state.

## Halting Rules

- DO NOT halt while delegated work lacks explicit lifecycle operations such as spawn, follow-up, wait, resume, and close.
- DO NOT halt while parallel agent work can collide on ownership or mutate shared state without visibility.
- DO NOT halt while subagent outputs are not captured as traceable artifacts that a parent turn can inspect and integrate.
- HALT when epic `VGb1c2DBj` is terminal and role-based delegation is proven through tests, transcripts, and docs.
- YIELD to human only if delegation authority, ownership rules, or role taxonomy require a product decision.
