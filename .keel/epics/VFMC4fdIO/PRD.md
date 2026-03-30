# Ollama Provider - Product Requirements

## Problem Statement

Ollama exposes an OpenAI-compatible API at localhost:11434/v1. A named --provider ollama should route to the OpenAI-compatible adapter with Ollama's default base URL, giving users access to any model Ollama serves without additional configuration.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | --provider ollama routes to OpenAI adapter with Ollama default base URL | paddles --provider ollama --model llama3 processes a prompt via local Ollama | CLI proof |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local inference user | Developer running Ollama locally for private model serving | Named --provider ollama flag that works out of the box with default Ollama URL |

## Scope

### In Scope

- [SCOPE-01] --provider ollama enum variant routing to OpenAI-compatible adapter
- [SCOPE-02] Default base URL of http://localhost:11434/v1
- [SCOPE-03] OLLAMA_HOST env var overrides base URL
- [SCOPE-04] Model ID passed through to Ollama API (e.g. llama3, qwen2.5)

### Out of Scope

- [SCOPE-05] Ollama-specific APIs beyond OpenAI compatibility (pull, create, etc.)
- [SCOPE-06] Ollama installation or model management

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | --provider ollama accepted as CLI flag value | GOAL-01 | must | Entry point for the Ollama provider variant |
| FR-02 | Routes to OpenAI-compatible adapter with http://localhost:11434/v1 base URL | GOAL-01 | must | Ollama's default OpenAI-compatible endpoint |
| FR-03 | OLLAMA_HOST env var overrides the default base URL | GOAL-01 | must | Supports non-default Ollama configurations |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | No new adapter code needed, reuses OpenAI-compatible adapter | GOAL-01 | must | Keeps implementation minimal and leverages existing tested code |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| CLI routing | Manual CLI proof | paddles --provider ollama --model llama3 processes a prompt |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Ollama's OpenAI compatibility layer handles the same chat completions schema | Minor request adjustments needed | Test against running Ollama instance |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| None | - | Design is straightforward |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider ollama --model llama3 processes a prompt via local Ollama
<!-- END SUCCESS_CRITERIA -->
