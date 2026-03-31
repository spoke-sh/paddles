# Real Chord Integration - Software Design Document

## Architecture Overview

Transitioning from a mock simulation to a direct integration with the `legacy-core` crate.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `Instance` | Managing project state and providers | `Instance::new(path)` |
| `PromptLoop` | Orchestrating the agentic interaction loop | `PromptLoop::run()` |
| `main.rs` | CLI entry point and lifecycle coordinator | N/A |

## Data Flows

1. `main.rs` initializes `legacy_core::Instance`.
2. `main.rs` creates a new `legacy_core::Session`.
3. `main.rs` constructs a `PromptLoop`.
4. The user prompt is passed to `PromptLoop::run`.
5. The result is printed to the terminal.
