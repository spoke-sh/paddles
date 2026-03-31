# Core Loop Implementation - SDD

## Architecture Overview

Wiring the `PromptLoop` from `legacy-core` into `paddles` CLI.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `Instance` | Provides dependencies (provider, tools, sessions) | `instance.session_repo()`, etc. |
| `PromptLoop` | Executes the agentic loop | `PromptLoop::new(...)`, `run()` |

## Data Flows

1. `main.rs` builds `Instance`.
2. `main.rs` extracts required `Arc` components from `Instance`.
3. `main.rs` constructs `PromptLoop`.
4. `main.rs` calls `loop.run()`.
5. `PromptResult` is rendered to stdout.
