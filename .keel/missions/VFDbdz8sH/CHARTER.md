# Codex-Style Interactive TUI For Paddles - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace the plain interactive REPL with a Codex-style terminal UI that makes paddles feel like a live transcript-oriented coding agent while preserving the existing local-first controller/runtime behavior. | board: VFDbdzqtU |

## Constraints

- Keep `--prompt` one-shot execution plain and script-friendly.
- Keep the default interactive path local-first with no new mandatory remote dependencies.
- Preserve the existing controller semantics for routing, citations, and event emission while changing the presentation layer.
- Prefer a small `paddles`-owned TUI architecture inspired by Codex over importing a large external UI framework wholesale.

## Halting Rules

- DO NOT halt while the interactive loop still uses the legacy plain stdin/stdout presentation.
- DO NOT halt while user/assistant/action turns are not visually distinct in the interactive transcript.
- DO NOT halt while live turn events or final assistant answers cannot be observed cleanly inside the TUI.
- HALT when the epic linked from MG-01 is terminal and the interactive TUI is proven by tests and transcript artifacts.
- YIELD to human only if terminal behavior differs across environments in a way that requires a product decision rather than an implementation fix.
