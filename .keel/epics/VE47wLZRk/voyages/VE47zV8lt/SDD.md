# Chord Wiring - Software Design Document

## Architecture Overview

Integrating `legacy-engine` (chord) as the core execution engine for the `paddles` CLI.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `src/main.rs` | CLI entry, prompt parsing, delegating to chord | `main()` |
| `legacy-engine` | Agentic coding engine | `Engine::run()` (TBD) |

## Data Flows

1. User provides `--prompt "message"`.
2. `main.rs` parses prompt.
3. `main.rs` initializes `legacy-engine` engine.
4. `main.rs` passes prompt to `legacy-engine`.
5. `legacy-engine` executes (read/write files as needed).
6. Result/logs output to user.

## Design Decisions

| ADR | Decision | Rationale |
|-----|----------|-----------|
| N/A | Direct CLI Integration | Simplest first step for the "mech suit" metaphor. |

## Verification Strategy

- Story-level tests for `legacy-engine` integration.
- Manual verification of `--prompt` output.
