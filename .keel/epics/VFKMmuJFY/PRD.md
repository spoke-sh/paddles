# Anthropic Claude Provider - Product Requirements

## Problem Statement

No Anthropic adapter exists. A Claude adapter needs to implement SynthesizerEngine and RecursivePlanner using the Anthropic messages API with proper role mapping (system prompt via top-level parameter, not message role).

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Anthropic messages API adapter implements SynthesizerEngine and RecursivePlanner. | paddles processes a prompt end-to-end with --provider anthropic | Passing end-to-end run |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Person running paddles with --provider anthropic --model claude-sonnet-4-20250514 | Send prompts through Anthropic Claude for synthesis and planning |

## Scope

### In Scope

- [SCOPE-01] Anthropic messages API HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON responses
- [SCOPE-03] API key via ANTHROPIC_API_KEY environment variable
- [SCOPE-04] Model ID passed through (e.g. claude-sonnet-4-20250514)
- [SCOPE-05] Proper role mapping (system prompt via system parameter, not message role)

### Out of Scope

- [SCOPE-06] Streaming responses
- [SCOPE-07] Tool use via Anthropic tool_use blocks

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | HTTP adapter sends messages API requests with proper content blocks. | GOAL-01 | must | Core transport for all LLM interactions through the Anthropic messages endpoint. |
| FR-02 | System prompt uses Anthropic's top-level system parameter. | GOAL-01 | must | Anthropic requires system prompts outside the messages array; incorrect placement causes errors. |
| FR-03 | Response parsing extracts structured JSON for planner decisions. | GOAL-01 | must | RecursivePlanner needs machine-readable decisions from model output. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Adapter handles Anthropic-specific headers (x-api-key, anthropic-version). | GOAL-01 | must | Anthropic authenticates via x-api-key header and requires an anthropic-version header on every request. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Anthropic end-to-end | CLI run with --provider anthropic --model claude-sonnet-4-20250514 | Successful prompt processing output |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Anthropic messages API schema is stable. | Version pin in anthropic-version header. | Monitor Anthropic API changelog. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| None at this time. | - | - |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider anthropic --model claude-sonnet-4-20250514 processes a prompt end-to-end
<!-- END SUCCESS_CRITERIA -->
