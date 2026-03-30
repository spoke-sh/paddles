# Google Gemini Provider - Product Requirements

## Problem Statement

No Google adapter exists. A Gemini adapter needs to implement SynthesizerEngine and RecursivePlanner using the Gemini generateContent API with proper content part mapping.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Gemini generateContent API adapter implements SynthesizerEngine and RecursivePlanner. | paddles processes a prompt end-to-end with --provider google | Passing end-to-end run |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Person running paddles with --provider google --model gemini-2.5-flash | Send prompts through Google Gemini for synthesis and planning |

## Scope

### In Scope

- [SCOPE-01] Gemini generateContent HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON responses
- [SCOPE-03] API key via GOOGLE_API_KEY environment variable
- [SCOPE-04] Model ID passed through (e.g. gemini-2.5-flash)
- [SCOPE-05] Content parts mapping (text parts in Gemini format)

### Out of Scope

- [SCOPE-06] Streaming responses
- [SCOPE-07] Gemini function calling

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | HTTP adapter sends generateContent requests with proper content structure. | GOAL-01 | must | Core transport for all LLM interactions through the Gemini REST endpoint. |
| FR-02 | Response parsing extracts text from candidates array. | GOAL-01 | must | Gemini returns completions inside a candidates array with content parts. |
| FR-03 | Structured JSON parsing for planner decisions from model output. | GOAL-01 | must | RecursivePlanner needs machine-readable decisions from model output. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Adapter uses Gemini REST API with API key as query parameter. | GOAL-01 | must | Gemini authenticates via ?key= query parameter rather than a header. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Gemini end-to-end | CLI run with --provider google --model gemini-2.5-flash | Successful prompt processing output |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Gemini REST API v1 is stable. | Version pin needed. | Monitor Google AI API changelog. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| None at this time. | - | - |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] paddles --provider google --model gemini-2.5-flash processes a prompt end-to-end
<!-- END SUCCESS_CRITERIA -->
