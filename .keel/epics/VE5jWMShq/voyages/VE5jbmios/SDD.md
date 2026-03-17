# Candle Logic Implementation - SDD

## Architecture Overview

Integrating `candle-transformers` into `CandleProvider`.

## Components

| Component | Responsibility | Interface |
|-----------|----------------|-----------|
| `ModelLoader` | Fetches/Loads weights and tokenizer | `load()` |
| `InferenceLoop` | Manages token-by-token generation | `generate()` |

## Data Flows

1. `CandleProvider` receives `generate()` call.
2. `ModelLoader` ensures weights are in memory.
3. User prompt is tokenized.
4. `InferenceLoop` runs until EOS or limit.
5. Tokens are detokenized and streamed to `StreamChunk`.
