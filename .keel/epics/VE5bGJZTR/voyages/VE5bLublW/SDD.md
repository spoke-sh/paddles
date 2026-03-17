# Dogma and Bias Calibration - SDD

## Architecture Overview

Extending the `BootContext` and `Constitution` to include bias offsets and religious dogma validation.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `Cli` | Parse `--biases` and environmental flags | `clap` args |
| `Dogma` | Encapsulates immutable invariants | `Dogma::validate()` |
| `BootContext` | Aggregates weights, biases, and dogma state | N/A |

## Data Flows

1. `main.rs` parses credits, weights, and biases.
2. `BootContext` initializes and calls `Constitution::validate(weight)`.
3. `BootContext` calls `Dogma::validate(context)`.
4. If dogma is violated (e.g. reality > simulation), boot fails with "Unclean Boot".
