# Provider Routing And CLI Flags - SRS

## Summary

Epic: VFKMjf28V
Goal: Deliver CLI provider flag and factory routing to support multiple model backends

## Scope

### In Scope

- [SCOPE-01] CLI --provider flag accepting local, openai, anthropic, google, moonshot
- [SCOPE-02] Provider-aware factory closures routing to correct adapter
- [SCOPE-03] API key resolution from provider-specific environment variables
- [SCOPE-04] Optional --provider-url flag for custom API endpoints
- [SCOPE-05] reqwest dependency for shared HTTP client infrastructure

### Out of Scope

- [SCOPE-06] Concrete provider adapter implementations (separate epics)
- [SCOPE-07] Provider-specific streaming or tool use features

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | --provider CLI flag accepts local, openai, anthropic, google, moonshot values | SCOPE-01 | FR-01 | manual |
| SRS-02 | Factory closures dispatch to correct adapter constructor based on provider value | SCOPE-02 | FR-02 | manual |
| SRS-03 | API key resolved from provider-specific env var (OPENAI_API_KEY, ANTHROPIC_API_KEY, etc.) | SCOPE-03 | FR-03 | manual |
| SRS-04 | --provider-url overrides default API base URL for any provider | SCOPE-04 | FR-04 | manual |
| SRS-05 | Local provider is default when --provider is not specified | SCOPE-01 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | All existing tests pass unchanged | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Provider routing lives in main.rs composition root | SCOPE-02 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
