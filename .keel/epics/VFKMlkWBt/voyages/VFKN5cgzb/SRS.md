# OpenAI Chat Completions Adapter - SRS

## Summary

Epic: VFKMlkWBt
Goal: Deliver OpenAI-compatible HTTP adapter implementing SynthesizerEngine and RecursivePlanner

## Scope

### In Scope

- [SCOPE-01] OpenAI chat completions HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON
- [SCOPE-03] Configurable base URL for OpenAI-compatible providers
- [SCOPE-04] API key from environment variable

### Out of Scope

- [SCOPE-05] Streaming token responses
- [SCOPE-06] OpenAI function calling / tools API

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | HTTP adapter sends chat completion requests with system/user/assistant messages | SCOPE-01 | FR-01 | manual |
| SRS-02 | Response parsing extracts structured JSON for planner actions | SCOPE-02 | FR-02 | manual |
| SRS-03 | Base URL configurable via --provider-url flag | SCOPE-03 | FR-03 | manual |
| SRS-04 | API key read from provider-specific env var | SCOPE-04 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
