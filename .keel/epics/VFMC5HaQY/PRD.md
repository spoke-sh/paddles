# Mistral RS Native Inference - Product Requirements

## Problem Statement

The current local inference is hardcoded to Qwen via Candle. mistral.rs is a pure-Rust inference engine built on Candle that supports many architectures (Llama, Mistral, Gemma, Phi, Qwen) with quantization and CUDA. Replacing the Qwen-specific runtime with mistral.rs would give paddles native multi-architecture local inference without an external server process.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | mistral.rs replaces Qwen-specific Candle code as the local inference backend, supporting multiple model architectures | paddles --provider local --model mistralai/Mistral-7B-Instruct processes a prompt via mistral.rs | CLI proof |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local inference user | Developer running models locally without external API dependencies | Multi-architecture model support beyond Qwen |

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

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | LocalInferenceAdapter loads models via mistral.rs and implements SynthesizerEngine | GOAL-01 | must | Core adapter for local inference |
| FR-02 | Same adapter implements RecursivePlanner via structured JSON parsing | GOAL-01 | must | Planning requires structured output from local models |
| FR-03 | --provider local --model <any-supported-arch> works with mistral.rs backend | GOAL-01 | must | CLI entry point for multi-architecture local inference |
| FR-04 | GGUF quantized models are supported for memory-constrained environments | GOAL-01 | should | Enables running larger models on consumer hardware |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Existing SiftAgentAdapter continues to work as --provider sift for backwards compat | GOAL-01 | must | No regression for existing users |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Multi-arch inference | Manual CLI proof | paddles --provider local --model mistralai/Mistral-7B-Instruct processes a prompt |
| Backwards compat | Existing tests | --provider sift path continues to work |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| mistral.rs API is stable enough for library embedding | Pin to specific version | Review mistral.rs release cadence |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether to keep SiftAgentAdapter as a separate provider or fully replace it | Epic owner | Resolved - coexist as --provider sift vs --provider local |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider local --model mistralai/Mistral-7B-Instruct processes a prompt via mistral.rs
- [ ] Existing --provider sift path continues to work unchanged
<!-- END SUCCESS_CRITERIA -->
