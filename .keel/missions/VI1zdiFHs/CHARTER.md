# Separate Runtime Planner Executor And Capability Boundaries - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Separate planner orchestration from executor-side runtime helpers and external-capability adapter execution without changing runtime behavior. | board: VI1zeXMOr |

## Constraints

- Preserve planner, executor, and external-capability behavior exactly; this mission is refactor-only.
- Keep local-first governance and execution-policy checks in force.
- Move cohesive execution helpers behind application-layer module boundaries instead of adding shallow wrappers.
- Prove the slice with focused regression coverage and the standard repository checks.

## Halting Rules

- DO NOT halt while epic `VI1zeXMOr` has draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VI1zeXMOr` is complete and linked evidence proves behavior stayed stable after extraction.
- YIELD to the human if the remaining work requires changing planner, executor, or external-capability product semantics.
