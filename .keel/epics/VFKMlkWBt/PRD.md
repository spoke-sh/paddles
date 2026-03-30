# OpenAI Compatible Provider - Product Requirements

## Problem Statement

No HTTP API provider exists. An OpenAI-compatible adapter needs to implement SynthesizerEngine and RecursivePlanner using the chat completions API, supporting OpenAI, Moonshot Kimi, and any other OpenAI-compatible endpoint via configurable base URL.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | OpenAI chat completions adapter implements SynthesizerEngine and RecursivePlanner. | paddles processes a prompt end-to-end with --provider openai | Passing end-to-end run |
| GOAL-02 | Configurable base URL supports OpenAI, Moonshot Kimi, and any compatible endpoint. | paddles processes a prompt via --provider moonshot with custom base URL | Passing end-to-end run |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Person running paddles with --provider openai --model gpt-4o | Send prompts through OpenAI-compatible LLM APIs for synthesis and planning |

## Scope

### In Scope

- [SCOPE-01] OpenAI chat completions HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON responses
- [SCOPE-03] Configurable base URL for OpenAI-compatible providers (Moonshot, etc.)
- [SCOPE-04] API key via environment variable (OPENAI_API_KEY, MOONSHOT_API_KEY)
- [SCOPE-05] Model ID passed through to the API (e.g. gpt-4o, kimi-2.5)

### Out of Scope

- [SCOPE-06] Streaming token-by-token responses (future enhancement)
- [SCOPE-07] Function calling / tool use via OpenAI tools API

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | HTTP adapter sends chat completion requests with system/user/assistant messages. | GOAL-01 | must | Core transport for all LLM interactions through the OpenAI completions endpoint. |
| FR-02 | Response parsing extracts structured JSON for planner action decisions. | GOAL-01 | must | RecursivePlanner needs machine-readable decisions from model output. |
| FR-03 | Base URL is configurable via --provider-url flag or env var. | GOAL-02 | must | Enables the same adapter to target OpenAI, Moonshot, or any compatible endpoint. |
| FR-04 | API key is read from environment variable based on provider name. | GOAL-01 | must | Authenticates requests without hardcoding credentials. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Adapter uses reqwest for HTTP with timeout and error handling. | GOAL-01 | must | Consistent HTTP client with proper failure modes and connection management. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| OpenAI end-to-end | CLI run with --provider openai --model gpt-4o | Successful prompt processing output |
| Moonshot end-to-end | CLI run with --provider moonshot --model kimi-2.5 | Successful prompt processing via custom base URL |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| OpenAI-compatible APIs accept the same chat completions JSON schema. | Moonshot/others may need minor request adjustments. | Test against Moonshot endpoint during development. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether to use streaming or non-streaming completions endpoint. | Epic owner | Resolved - non-streaming first |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider openai --model gpt-4o processes a prompt end-to-end
- [ ] paddles --provider moonshot --model kimi-2.5 processes a prompt via configurable base URL
<!-- END SUCCESS_CRITERIA -->
