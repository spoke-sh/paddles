# Mistral RS Local Inference Backend - Software Design Description

> Replace Qwen-specific Candle code with mistral.rs for multi-architecture local inference

**SRS:** [SRS.md](SRS.md)

## Overview

A new `LocalInferenceAdapter` in `src/infrastructure/adapters/mistralrs_provider.rs` replaces the Qwen-specific Candle code with mistral.rs for local inference. It uses the mistral.rs `MistralRs` struct for model loading and inference, implementing `SynthesizerEngine` by sending formatted prompts to the loaded model. The `ConversationFactory` pattern maps to mistral.rs's request/response API. Device selection (CUDA/CPU) uses mistral.rs's built-in detection.

## Context & Boundaries

```
┌─────────────────────────────────────────────┐
│              This Voyage                    │
│                                             │
│  ┌────────────────────┐  ┌───────────────┐ │
│  │ LocalInference     │  │ mistral.rs    │ │
│  │ Adapter            │──│ MistralRs     │ │
│  │ (SynthesizerEngine │  │ (model load + │ │
│  │  RecursivePlanner) │  │  inference)   │ │
│  └────────────────────┘  └───────────────┘ │
└─────────────────────────────────────────────┘
        ↑                        ↑
   [main.rs routing]        [HF Hub / local GGUF]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| mistral.rs | crate | Model loading, tokenization, inference | mistralrs 0.x (pin to release) |
| hf-hub | crate | Model weight download (already a dependency) | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Inference engine | mistral.rs | Pure Rust, Candle-based, supports Llama/Mistral/Gemma/Phi/Qwen natively |
| Coexistence | --provider local vs --provider sift | No breaking change; SiftAgentAdapter remains as-is |
| Quantization | GGUF via mistral.rs | Widely available quantized models on HF Hub |
| Device selection | mistral.rs built-in | Automatically uses CUDA if available, falls back to CPU |

## Architecture

```
src/infrastructure/adapters/
  ├── mistralrs_provider.rs   (new - LocalInferenceAdapter)
  ├── sift_agent_adapter.rs   (unchanged - legacy Qwen path)
  └── openai_adapter.rs       (unchanged)

main.rs routing:
  --provider local → LocalInferenceAdapter (mistral.rs)
  --provider sift  → SiftAgentAdapter (legacy Qwen/Candle)
```

## Components

### LocalInferenceAdapter
- Loads model via `MistralRs::new()` with architecture auto-detection from model config
- Implements `SynthesizerEngine::synthesize()` by formatting prompt as chat messages and calling mistral.rs inference
- Implements `RecursivePlanner::plan()` by appending JSON schema instructions to the system prompt and parsing structured output
- Holds loaded model in memory for reuse across turns

### Model Loading
- Accepts HF model ID or local path
- Uses mistral.rs model loader which handles tokenizer, weights, and architecture detection
- GGUF files detected by extension and loaded via quantized pipeline

## Data Flow

1. User runs `paddles --provider local --model mistralai/Mistral-7B-Instruct-v0.3`
2. CLI routes to `LocalInferenceAdapter::new(model_id)`
3. Adapter initializes mistral.rs pipeline (downloads weights if needed, selects device)
4. `synthesize()` calls format prompt into chat messages, run mistral.rs inference, return generated text
5. `plan()` calls append structured output instructions, parse JSON from generation

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Model not found | mistral.rs returns load error | "Failed to load model: {model_id}" | User downloads model or checks path |
| CUDA unavailable | mistral.rs device detection | Falls back to CPU automatically | None needed |
| OOM on model load | Allocation failure | "Model too large for available memory; try a quantized variant" | User selects GGUF model |
| Malformed planner JSON | serde parse failure | Retry with stricter prompt or return parse error | Fallback to unstructured response |
