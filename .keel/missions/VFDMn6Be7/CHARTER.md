# Make Paddles Evidence-First And Observable - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Make `paddles` answer repository questions through an explicit evidence-gathering path by default, produce grounded answers with file citations by default, and render a Codex-style action stream as the default REPL experience. | board: VFDMnu8k9 |

## Constraints

- Keep the controller local-first and preserve direct casual chat plus deterministic tool execution as low-friction paths.
- Treat repository-question retrieval as an explicit gatherer responsibility; do not hide primary repo-answer retrieval inside synthesizer-private context assembly.
- Final repository answers must cite source files by default and admit insufficient evidence instead of improvising unsupported claims.
- The Codex-style action stream is the default REPL surface; do not add a quiet flag as part of this mission.
- Observability must be implemented as typed runtime events or equally reusable renderer-facing structures so the stream stays consistent across gatherers, planners, tools, and synthesis.

## Halting Rules

- DO NOT halt while epic `VFDMnu8k9` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFDMnu8k9` is verified and `paddles` defaults to explicit evidence gathering for repo questions, grounded cited synthesis, and a default action stream.
- YIELD if the current local synthesizer lane cannot satisfy the grounded-citation contract without a stronger default model or a product change to the response format.
