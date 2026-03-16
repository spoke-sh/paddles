# Chord Wiring - Software Design Document

## Architecture Overview

Integrating `wonopcode` (chord) as the core execution engine for the `paddles` CLI.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `src/main.rs` | CLI entry, prompt parsing, delegating to chord | `main()` |
| `wonopcode` | Agentic coding engine | `Engine::run()` (TBD) |

## Data Flows

1. User provides `--prompt "message"`.
2. `main.rs` parses prompt.
3. `main.rs` initializes `wonopcode` engine.
4. `main.rs` passes prompt to `wonopcode`.
5. `wonopcode` executes (read/write files as needed).
6. Result/logs output to user.

## Design Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| N/A | Direct CLI Integration | Simplest first step for the "mech suit" metaphor. |

## Verification Strategy

- Story-level tests for `wonopcode` integration.
- Manual verification of `--prompt` output.
