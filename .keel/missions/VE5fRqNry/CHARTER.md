# The Active Pulse - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Fully wire the real `PromptLoop` in `main.rs` | board: VE5fVmIs3 |
| MG-02 | Execute a non-trivial agentic task through the CLI | board: VE5fVmIs3 |

## Constraints

- Must use `legacy-core` primitives (`Instance`, `PromptLoop`).
- Must maintain 100% board integrity.

## Halting Rules

- HALT when `PromptLoop` is executing and verified.
- YIELD if the `legacy-core` API remains too opaque for safe wiring.
