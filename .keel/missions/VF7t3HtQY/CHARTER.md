# Sift-Native Tool Runtime - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace wonopcode-owned core orchestration with a Paddles-controlled Sift runtime that retains turns, tool outputs, and bounded workspace evidence as first-class state. | board: VF7t633ux |

## Constraints

- Preserve the boot sequence contract and existing single-prompt plus interactive CLI entrypoints.
- Keep the execution path local-first: no new network dependency may be introduced for prompt handling or tool execution.
- Make common local tools simple to invoke through the runtime contract, with immediate support for search, file, shell, and edit/diff operations.
- Cut over hard: remove wonopcode-core/provider/tools from core runtime modules in the same implementation slice rather than dual-running both controllers.

## Halting Rules

- DO NOT halt while epic `VF7t633ux` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VF7t633ux` is verified and CLI proofs show prompt, interactive, and tool-assisted flows running through the Sift-native runtime.
- YIELD if upstream Sift interfaces prove insufficient for local tool context retention or generative session control without additional upstream design changes.
