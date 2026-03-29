# Define Transit-Aligned Trace Recorder Boundary - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Define a `paddles`-owned trace contract and recorder boundary aligned with `transit` lineage semantics so recursive turns, branches, tool activity, checkpoints, and future graph traces can be durably recorded locally without depending on UI prose or a networked trace service. | board: VFH4BXH4F |

## Constraints

- Keep the runtime generic. Do not make Keel or any other repository domain first-class in the trace model.
- Keep the trace contract `paddles`-owned. Align with `transit` semantics without leaking `transit` core types through the domain boundary.
- Preserve embedded-first local recording. The first durable path must work with embedded `transit-core` and must not require a running `transit` server.
- Keep `TurnEventSink` and operator transcript rendering distinct from durable trace recording. UI rows are a projection, not the source of truth.
- Preserve local-first bounded runtime behavior. Recording must fail closed or degrade honestly without destabilizing turn execution.
- Foundational docs must explain how the recorder boundary relates to the planner loop, gatherer boundary, artifact envelopes, and future replay.

## Halting Rules

- DO NOT halt while epic `VFH4BXH4F` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFH4BXH4F` is verified and `paddles` can project recursive turns into a stable recorder boundary with an embedded `transit-core` proof path.
- YIELD if recorder integration would weaken local-first guarantees, force a network dependency, or couple durable traces to UI-only strings.
