# Anthropic Messages API Adapter - SRS

## Summary

Epic: VFKMmuJFY
Goal: Deliver Anthropic Claude adapter implementing SynthesizerEngine and RecursivePlanner

## Scope

### In Scope

- [SCOPE-01] Anthropic messages API HTTP adapter implementing SynthesizerEngine
- [SCOPE-02] Same adapter implementing RecursivePlanner via structured JSON
- [SCOPE-03] API key from ANTHROPIC_API_KEY env var
- [SCOPE-04] System prompt via top-level system parameter

### Out of Scope

- [SCOPE-05] Streaming responses
- [SCOPE-06] Tool use blocks

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | HTTP adapter sends messages API requests with content blocks | SCOPE-01 | FR-01 | manual |
| SRS-02 | System prompt uses Anthropic top-level system parameter | SCOPE-04 | FR-02 | manual |
| SRS-03 | Response parsing extracts structured JSON for planner | SCOPE-02 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Anthropic-specific headers (x-api-key, anthropic-version) | SCOPE-03 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
