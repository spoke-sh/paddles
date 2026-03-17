# Auth and Default Stabilization - SDD

## Architecture Overview

Updating the boot sequence and registry adapter to handle authentication.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `Cli` | Parses `--hf-token` | `clap` args |
| `MechSuitService` | Orchestrates token passing | `boot()` |
| `HFHubAdapter` | Uses token for API calls | `ApiBuilder` |

## Data Flows

1. `main.rs` captures token from `--hf-token` or `HF_TOKEN` env.
2. Token passed to `MechSuitService`.
3. `HFHubAdapter` instantiated with token.
4. `HFHubAdapter` uses `ApiBuilder::new().with_token(token).build()`.
