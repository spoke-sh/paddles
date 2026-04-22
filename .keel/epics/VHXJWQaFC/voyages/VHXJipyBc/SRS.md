# Cross-Provider Deliberation Rollout And Verification - SRS

## Summary

Epic: VHXJWQaFC
Goal: Roll the deliberation substrate and signal model out across every supported provider with explicit capability negotiation, fallback semantics, tracing boundaries, and operator documentation.

## Scope

### In Scope

- [SCOPE-01] Native continuation implementations for OpenAI reasoning-capable transport, Anthropic extended/interleaved thinking, and Gemini thought signatures
- [SCOPE-02] Explicit summary-only, toggle-only, or unsupported/no-op semantics for Inception, Ollama, and Sift
- [SCOPE-03] Provider capability negotiation and fallback policies across `ModelProvider::ALL`
- [SCOPE-07] Cross-provider contract tests, thinking-mode catalogs, and operator/configuration documentation

### Out of Scope

- [SCOPE-08] Recursive harness policy changes that consume normalized deliberation signals
- [SCOPE-09] Default user-facing display of raw provider reasoning artifacts
- [SCOPE-10] New hosted orchestration or remote state services

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Every provider in `ModelProvider::ALL` must negotiate an explicit deliberation support level and continuation policy. | SCOPE-03 | FR-01 | test |
| SRS-02 | OpenAI reasoning-capable paths must preserve reusable reasoning state when the selected transport supports it, and must expose explicit limited behavior when the active transport/model cannot do so. | SCOPE-01, SCOPE-03 | FR-03 | test |
| SRS-03 | Anthropic extended thinking paths must preserve required thinking blocks and interleaved-thinking behavior across tool/result turns when enabled. | SCOPE-01 | FR-03 | test |
| SRS-04 | Gemini thinking paths must preserve required thought signatures or equivalent continuity handles across tool/function turns when thinking is enabled. | SCOPE-01 | FR-03 | test |
| SRS-05 | Inception, Ollama, and Sift must expose explicit summary-only, toggle-only, or unsupported/no-op semantics instead of pretending to provide reusable native reasoning continuity. | SCOPE-02, SCOPE-03 | FR-04 | test |
| SRS-06 | The repository must include a provider capability matrix, contract tests, and operator/configuration docs covering Moonshot, OpenAI, Anthropic, Gemini, Inception, Ollama, and Sift. | SCOPE-07 | FR-09 | review |
| SRS-07 | Every supported provider/model path must expose supported thinking modes or an explicit `none`/unsupported result through provider catalogs and runtime configuration surfaces. | SCOPE-03, SCOPE-07 | FR-08 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unsupported providers and unsupported model/provider combinations must fail soft with explicit capability reporting instead of hidden behavior changes. | SCOPE-02, SCOPE-03, SCOPE-07 | NFR-03 | test |
| SRS-NFR-02 | Provider-specific continuation semantics must remain behind adapter or application-normalization seams. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-05 | review |
| SRS-NFR-03 | Cross-provider rollout docs, configuration guidance, and thinking-mode catalogs must stay synchronized with the actual capability surface. | SCOPE-07 | NFR-04 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
