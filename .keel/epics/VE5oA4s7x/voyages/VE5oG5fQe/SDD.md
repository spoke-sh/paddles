# Interactive Loop Integration - SDD

## Architecture Overview

Adding an asynchronous `while` loop to `main.rs` that reads from `stdin` when the `prompt` CLI argument is missing.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `main.rs` | Interactive loop orchestrator | `tokio::io::stdin()` |
| `PromptLoop` | Executing each turn | `run()` |

## Data Flows

1. `main.rs` detects empty `prompt` arg.
2. `main.rs` starts a loop.
3. `main.rs` waits for `stdin` line.
4. `main.rs` passes line to `PromptLoop::run()` with the *same* session.
5. Result is printed.
6. Loop continues until "exit" or EOF.
