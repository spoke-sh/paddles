# Unify Conversation Transcript Projection Across Interfaces - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Establish one canonical conversation-scoped transcript plane so prompt entry and final replies are projected from the same durable source across TUI, web, and CLI surfaces. | board: VFYbtfpVG |
| MG-02 | Remove the current dependence on progress events and UI-local repair hacks for cross-surface transcript visibility. | manual: prompts and replies appear across attached interfaces without reload, replay polling, or global trace scraping |
| MG-03 | Leave the architecture decomposed into backlog-ready implementation slices with explicit boundaries for transcript identity, projection, notifications, and surface adoption. | manual: epic VFYbtfpVG has a planned voyage with backlog stories mapped to the architecture |

## Constraints

- Preserve `process_prompt_in_session_with_sink(...)` as the canonical command path for turn execution.
- Treat transcript state and progress state as separate planes; `TurnEvent` remains telemetry, not transcript truth.
- Keep TUI, web, and CLI as peer clients of shared conversation state rather than interface-specific transcript owners.
- Preserve local-first runtime constraints and avoid introducing new network services or browser build dependencies.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epic VFYbtfpVG is verified and the remaining work is only manual validation of cross-surface behavior
- YIELD to human when product direction is needed for shared-conversation UX or attachment semantics
