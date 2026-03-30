# Mistral RS Local Inference Backend - SRS

## Summary

Epic: VFMC5HaQY
Goal: Replace Qwen-specific Candle code with mistral.rs for multi-architecture local inference

## Scope

### In Scope

- [SCOPE-01] Add mistral.rs as a dependency replacing direct Candle model loading
- [SCOPE-02] New LocalInferenceAdapter implementing SynthesizerEngine using mistral.rs
- [SCOPE-03] Support for multiple architectures (Llama, Mistral, Gemma, Phi, Qwen) via mistral.rs
- [SCOPE-04] Quantized model support (GGUF) via mistral.rs
- [SCOPE-05] CUDA and CPU inference via mistral.rs device selection

### Out of Scope

- [SCOPE-06] Removing the existing SiftAgentAdapter (can coexist initially)
- [SCOPE-07] Vision or multimodal model support
- [SCOPE-08] Model download/management (use HF Hub or manual download)

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | LocalInferenceAdapter loads models via mistral.rs MistralRs struct | SCOPE-01, SCOPE-02 | FR-01 | test |
| SRS-02 | LocalInferenceAdapter implements SynthesizerEngine for text generation | SCOPE-02 | FR-01 | test |
| SRS-03 | LocalInferenceAdapter implements RecursivePlanner via structured JSON parsing | SCOPE-02 | FR-02 | test |
| SRS-04 | --provider local --model <id> routes to LocalInferenceAdapter with mistral.rs backend | SCOPE-02, SCOPE-03 | FR-03 | manual |
| SRS-05 | GGUF quantized models load and run via mistral.rs quantization support | SCOPE-04 | FR-04 | manual |
| SRS-06 | Device selection (CUDA/CPU) uses mistral.rs built-in detection | SCOPE-05 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Existing SiftAgentAdapter continues to work as --provider sift | SCOPE-01 | NFR-01 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
