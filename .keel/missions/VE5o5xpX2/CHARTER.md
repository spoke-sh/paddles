# The Interactive TUI - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Implement interactive prompt loop in `main.rs` | board: VE5oA4s7x |
| MG-02 | Execute `just paddles` to open the interactive interface | board: VE5oA4s7x |

## Constraints

- Must maintain existing non-interactive `--prompt` capability.
- Must use `wonopcode-core` for the underlying agentic loop.

## Halting Rules

- HALT when the interactive prompt is functional and accepts multiple turns.
