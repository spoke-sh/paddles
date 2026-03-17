# Lattice Structure Transition - SDD

## Architecture Overview

Applying DDD and Hexagonal Architecture to `paddles`.

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| `domain::model` | Domain | Core logic: BootContext, Constitution, Dogma |
| `domain::ports` | Domain | Interfaces for external dependencies |
| `application` | Application | Use case orchestration: Booting, Prompting |
| `infrastructure::adapters` | Infrastructure | Implementations of ports (Candle) |
| `infrastructure::cli` | Infrastructure | CLI entry point and argument parsing |

## Data Flows

1. `main.rs` (Infrastructure) parses args.
2. `main.rs` calls Application Use Case.
3. Application Use Case coordinates Domain objects.
4. Application Use Case invokes Ports (implemented by Adapters).
5. Result returned to `main.rs` for display.
