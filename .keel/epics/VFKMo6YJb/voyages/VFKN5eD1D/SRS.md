# Gemini GenerateContent Adapter - SRS

## Summary

Epic: VFKMo6YJb
Goal: Deliver Google Gemini adapter implementing SynthesizerEngine and RecursivePlanner

## Scope

### In Scope

- [SCOPE-01] Gemini generateContent HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON
- [SCOPE-03] API key from GOOGLE_API_KEY env var
- [SCOPE-04] Content parts mapping to Gemini format

### Out of Scope

- [SCOPE-05] Streaming responses
- [SCOPE-06] Gemini function calling

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | HTTP adapter sends generateContent requests with parts structure | SCOPE-01 | FR-01 | manual |
| SRS-02 | Response parsing extracts text from candidates array | SCOPE-01 | FR-02 | manual |
| SRS-03 | Structured JSON parsing for planner actions | SCOPE-02 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | API key as query parameter per Gemini convention | SCOPE-03 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
