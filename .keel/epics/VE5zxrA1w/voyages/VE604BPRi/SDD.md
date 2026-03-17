# Sift Implementation Transition - SDD

## Architecture Overview

Replacing custom infrastructure adapters with `sift`'s internal adapters.

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| `SiftRegistryAdapter` | Infrastructure | Wraps `sift` model loading |
| `SiftInferenceAdapter` | Infrastructure | Wraps `sift` `GenerativeModel` |

## Data Flows

1. `main.rs` identifies requested model.
2. `SiftRegistryAdapter` calls `QwenReranker::load()` or equivalent.
3. The resulting model is wrapped in `SiftInferenceAdapter`.
4. `PromptLoop` uses `SiftInferenceAdapter` for token generation.
