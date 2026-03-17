# Local Inference Integration - Product Requirements

## Problem Statement

The `CandleProvider` currently uses a hardcoded mock response. To build actual capacity for local agentic execution, we must integrate `candle-transformers` to load and run models (e.g. Llama or Qwen) locally.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Integrate Candle core and transformers | System loads a quantized model file | 100% |
| GOAL-02 | Real token generation | `CandleProvider` generates text from model logits | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | User requiring air-gapped execution | Local, verifiable model inference |

## Scope

### In Scope

- [SCOPE-01] Model loading logic (weights, tokenizer, config).
- [SCOPE-02] Inference loop implementation in `CandleProvider`.
- [SCOPE-03] Support for GGUF/Safetensors quantized models.

### Out of Scope

- [SCOPE-04] Distributed inference.
- [SCOPE-05] Training or fine-tuning capabilities.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `CandleProvider` must load a model from a local path. | GOAL-01 | must | Core capacity requirement. |
| FR-02 | `CandleProvider` must generate tokens based on user input. | GOAL-02 | must | Required for execution. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Model loading time must be logged. | GOAL-01 | must | For performance tracking. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- CLI proof: `paddles --prompt "Who are you?"` returns a model-generated (non-mock) response.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| A-01 | Small GGUF models are compatible with current candle-transformers versions. | Necessary for CPU-bound inference. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | Memory constraints on large models | Target < 8B parameter models for initial capacity. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles` generates text using a real local model.
<!-- END SUCCESS_CRITERIA -->
