# Registry Implementation - SDD

## Architecture Overview

Introducing a `Registry` port to handle model acquisition, and updating the `CandleAdapter` to use these real assets.

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| `ModelRegistry` | Domain Port | Interface for model discovery and acquisition |
| `HFHubAdapter` | Infrastructure | Implementation of `ModelRegistry` using `hf-hub` |
| `CandleAdapter` | Infrastructure | Loads and executes model files |

## Data Flows

1. User specifies `--model gemma-2b`.
2. `BootSystem` use case asks `ModelRegistry` for paths to weights/config.
3. `HFHubAdapter` downloads/locates files on disk.
4. Paths are passed to `CandleAdapter`.
5. `CandleAdapter` initializes `ModelWeights` and starts the loop.
