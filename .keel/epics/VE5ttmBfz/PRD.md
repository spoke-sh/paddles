# Hugging Face Model Integration - Product Requirements

## Problem Statement

Paddles currently uses simulated inference within its `CandleAdapter`. To provide real value as a "mech suit" for AI assistants, it must be able to pull real, high-performance weights from the Hugging Face registry and execute them locally using `candle`.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Hugging Face Hub Integration | Successful download and caching of model files | 100% |
| GOAL-02 | Gemma/Qwen Support | Real token generation from pulled weights | 100% |
| GOAL-03 | CLI Model Parameter | `--model` flag allows selecting between supported families | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Main user of Paddles | Access to state-of-the-art local models |

## Scope

### In Scope

- [SCOPE-01] Integration of `hf-hub` crate.
- [SCOPE-02] Implementation of Gemma or Qwen loading logic in `CandleAdapter`.
- [SCOPE-03] Model caching strategy.
- [SCOPE-04] CLI argument for model selection.

### Out of Scope

- [SCOPE-05] Support for every model on Hugging Face (focusing on Gemma/Qwen).
- [SCOPE-06] Fine-tuning or model uploads.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | System must download model files if not present locally. | GOAL-01 | must | Required for registry connectivity. |
| FR-02 | System must execute inference using pulled weights. | GOAL-02 | must | Core functionality. |
| FR-03 | System must support selecting a model ID via CLI. | GOAL-03 | must | Flexibility. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Progress of downloads must be visible. | GOAL-01 | must | UX for large file downloads. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- CLI Proof: `paddles --prompt "hello" --model gemma` returns a non-simulated response.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| A-01 | hf-hub crate is compatible with our Nix environment. | Essential for automated downloads. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | Model download size | Limit initial support to < 3B models for testing. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles` pull a real model from Hugging Face.
- [ ] `paddles` generates text from that model.
<!-- END SUCCESS_CRITERIA -->
