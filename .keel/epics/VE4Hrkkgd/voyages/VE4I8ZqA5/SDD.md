# Boot Sequence Mechanics - SDD

## Architecture Overview

Introducing a `BootContext` structure that orchestrates the initial loading of inherited credits, foundational weights, and constitutional bounds before passing control to the Wonopcode Engine.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `Config` / CLI args | Parse input values for inheritance and weights | `clap` args |
| `BootSequence` | Validates weights against constitution and initializes | `BootSequence::run()` |
| `main.rs` | Ties the boot sequence into the CLI flow | N/A |

## Data Flows

1. `paddles` invoked with `--credits <N>` and config for weights.
2. `BootSequence` instantiated.
3. Boot validation checks if weights are within constitution constraints.
4. On success, prints the calibrated environment.
5. Control continues to the agent prompt loop.
