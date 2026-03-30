# Mistral RS Native Inference - Product Requirements

## Problem Statement

The existing local provider (sift) is coupled to Qwen models. mistral.rs is a pure-Rust inference engine that supports many architectures (Llama, Mistral, Gemma, Phi, Qwen) with quantization and CUDA. Adding mistral.rs as a separate provider gives paddles a second local inference option that supports a broader range of models without an external server process.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | mistral.rs is available as a provider for native multi-architecture local inference alongside sift | paddles --provider mistralrs --model mistralai/Mistral-7B-Instruct processes a prompt | CLI proof |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local inference user | Developer running models locally without external API dependencies | Multi-architecture model support beyond what sift provides |

## Scope

### In Scope

- [SCOPE-01] Add mistral.rs as a dependency for a new provider adapter
- [SCOPE-02] MistralRsAdapter implementing SynthesizerEngine using mistral.rs inference
- [SCOPE-03] Same adapter implementing RecursivePlanner via structured JSON parsing
- [SCOPE-04] Support for multiple architectures (Llama, Mistral, Gemma, Phi, Qwen) via mistral.rs
- [SCOPE-05] GGUF quantized model support via mistral.rs

### Out of Scope

- [SCOPE-06] Modifying the existing sift provider or SiftAgentAdapter
- [SCOPE-07] Vision or multimodal model support
- [SCOPE-08] Model download or management (use HF Hub or manual download)

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | MistralRsAdapter loads models via mistral.rs and implements SynthesizerEngine | GOAL-01 | must | Core adapter for mistral.rs inference |
| FR-02 | Same adapter implements RecursivePlanner via structured JSON parsing | GOAL-01 | must | Planning requires structured output from local models |
| FR-03 | --provider mistralrs --model <hf-model-id> selects the mistral.rs backend | GOAL-01 | must | CLI entry point |
| FR-04 | GGUF quantized models are supported for memory-constrained environments | GOAL-01 | should | Enables running larger models on consumer hardware |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Existing sift provider continues to work unchanged as --provider sift | GOAL-01 | must | No regression for existing local inference path |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Multi-arch inference | Manual CLI proof | paddles --provider mistralrs --model mistralai/Mistral-7B-Instruct processes a prompt |
| Sift unchanged | Existing tests | All 90 tests pass, --provider sift works |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| mistral.rs API is stable enough for library embedding | Pin to specific version | Review mistral.rs release cadence |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether mistral.rs should be an optional feature flag to avoid heavy compile times | operator | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider mistralrs --model mistralai/Mistral-7B-Instruct processes a prompt via mistral.rs
- [ ] Existing sift provider continues to work unchanged
<!-- END SUCCESS_CRITERIA -->
