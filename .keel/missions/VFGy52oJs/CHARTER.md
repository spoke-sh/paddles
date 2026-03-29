# Integrate Sift Graph Search Into The Gatherer Boundary - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Pull the latest Sift graph/branching runtime into `paddles` and expose it through the generic gatherer boundary so the recursive planning harness can use bounded graph-mode retrieval with preserved branch/frontier evidence, visible graph telemetry, and local-first fail-closed behavior. | board: VFGy53NJt |

## Constraints

- Keep graph search behind the gatherer boundary. Do not introduce Keel-specific, board-specific, or other repository-specific top-level runtime intents.
- Update the pinned `sift` dependency as part of the same slice so `paddles` uses the real upstream graph/branching API rather than a speculative local shim.
- Preserve local-first bounded behavior. Graph mode must degrade honestly when unavailable or invalid rather than silently pretending to have richer context than it does.
- Preserve controller ownership of validation, budgets, deterministic execution, and safe command boundaries even when graph-mode gatherers get richer planner traces.
- Preserve stable machine-readable graph lineage metadata so a future durable recorder can persist graph-capable turns without reverse-engineering UI prose.
- Keep the integration compatible with a future embedded `transit-core` recorder boundary. Do not assume graph-mode evidence or telemetry will only ever live in an interactive terminal surface, and do not require a networked `transit` server.
- Foundational docs must explain graph-mode gatherers as a generic recursive-context capability, not as a product-specific special case.

## Halting Rules

- DO NOT halt while epic `VFGy53NJt` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFGy53NJt` is verified and `paddles` can route graph-capable gatherer work through the recursive harness with typed graph evidence and operator-visible telemetry.
- YIELD if pulling the upstream graph runtime would weaken local-first guarantees, bounded execution, or the generic harness boundary.
