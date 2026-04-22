# Deliberation Substrate And Continuation Contracts - SRS

## Summary

Epic: VHXJWQaFC
Goal: Establish provider-agnostic deliberation state contracts and adapter continuity boundaries so provider-native reasoning can be carried between steps without becoming canonical paddles rationale.

## Scope

### In Scope

- [SCOPE-01] Provider-agnostic deliberation capability negotiation for every provider/model path
- [SCOPE-02] Adapter-owned `DeliberationState` or equivalent opaque continuation artifact kept separate from transcript/render/rationale state
- [SCOPE-03] Initial native continuation implementation for Moonshot/Kimi reasoning continuity across tool/result turns
- [SCOPE-05] Contract tests proving both native continuation and explicit no-op behavior
- [SCOPE-06] Debug-scoped trace boundaries for provider-native reasoning artifacts

### Out of Scope

- [SCOPE-08] Recursive harness decision changes that consume normalized deliberation signals
- [SCOPE-09] Full provider rollout for OpenAI, Anthropic, Gemini, Inception, Ollama, and Sift
- [SCOPE-10] User-facing display of raw provider-native reasoning content

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The provider/model capability surface must classify deliberation support for every path as native-continuation, limited/summarized, toggle-only, or unsupported/no-op. | SCOPE-01 | FR-01 | review |
| SRS-02 | Adapter turn interfaces must be able to return and accept opaque `DeliberationState` separate from `AuthoredResponse`, transcript state, and paddles `rationale`. | SCOPE-02 | FR-02 | test |
| SRS-03 | The Moonshot/Kimi adapter must preserve required provider reasoning continuity across tool/result turns without emitting raw reasoning into canonical user-visible content. | SCOPE-03 | FR-03 | test |
| SRS-04 | Any recorded provider-native reasoning artifacts must be stored on a debug-scoped or forensic path that is separate from canonical transcript/render persistence. | SCOPE-06 | FR-02 | review |
| SRS-05 | The repository must include contract tests covering one native-continuation provider path and one explicit unsupported/no-op provider path. | SCOPE-05 | FR-03 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Deliberation state must remain opaque and provider-scoped outside adapter/application seams so provider payload shapes do not leak into the domain model. | SCOPE-01, SCOPE-02 | NFR-05 | review |
| SRS-NFR-02 | Provider-native reasoning artifacts retained for debugging must be bounded and redactable rather than becoming an unbounded durable log. | SCOPE-06 | NFR-02 | review |
| SRS-NFR-03 | The initial substrate rollout must add no new hosted runtime dependency. | SCOPE-01, SCOPE-03, SCOPE-06 | NFR-01 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
