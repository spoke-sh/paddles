# Ollama Provider Variant - SRS

## Summary

Epic: VFMC4fdIO
Goal: Deliver --provider ollama routing to OpenAI adapter with Ollama defaults

## Scope

### In Scope

- [SCOPE-01] --provider ollama enum variant routing to OpenAI-compatible adapter
- [SCOPE-02] Default base URL of http://localhost:11434/v1
- [SCOPE-03] OLLAMA_HOST env var overrides base URL
- [SCOPE-04] Model ID passed through to Ollama API (e.g. llama3, qwen2.5)

### Out of Scope

- [SCOPE-05] Ollama-specific APIs beyond OpenAI compatibility (pull, create, etc.)
- [SCOPE-06] Ollama installation or model management

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | --provider ollama accepted as CLI flag value | SCOPE-01 | FR-01 | manual |
| SRS-02 | Ollama variant constructs OpenAI adapter with http://localhost:11434/v1 as default base URL | SCOPE-02 | FR-02 | manual |
| SRS-03 | OLLAMA_HOST env var overrides the default base URL when set | SCOPE-03 | FR-03 | manual |
| SRS-04 | Model ID from --model flag passed through to Ollama API unchanged | SCOPE-04 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | No new adapter code; reuses existing OpenAI-compatible adapter | SCOPE-01 | NFR-01 | code review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
