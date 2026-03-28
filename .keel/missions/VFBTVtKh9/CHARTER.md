# Context-Gathering Subagent Routing - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Build a proper context-gathering subagent lane so Paddles can route retrieval-heavy requests to specialized models such as Chroma Context-1 without replacing the default answer runtime. | board: VFBTXlHli |

## Constraints

- Preserve the current local-first answer path and existing Qwen-powered chat/tool execution as the default runtime.
- Treat Context-1 as a specialized context-gathering model only; keep context gathering and final answer synthesis as separate roles.
- Any Context-1 or remote gatherer integration must be explicit, capability-gated, and fail closed when the expected harness/runtime is unavailable.
- Keep the boot sequence, CLI entrypoints, and current operator-facing behavior stable while the new lane is introduced.

## Halting Rules

- DO NOT halt while epic `VFBTXlHli` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFBTXlHli` is verified and Paddles can route retrieval-heavy requests through a documented context-gathering lane with an explicit Context-1 integration boundary.
- YIELD if honest Context-1 integration requires non-public harness behavior that cannot be represented safely inside Paddles.
