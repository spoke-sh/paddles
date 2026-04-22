# Cross-Provider Deliberation Rollout And Verification - Software Design Description

> Roll the deliberation substrate and signal model out across every supported provider with explicit capability negotiation, fallback semantics, tracing boundaries, and operator documentation.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage completes the provider matrix. Each supported provider advertises
its reasoning behavior and thinking modes explicitly, native-continuation
providers implement the correct continuity contract, and limited providers
document their safe fallback mode. The result is one capability surface across
`ModelProvider::ALL` instead of scattered provider assumptions.

## Context & Boundaries

- In scope:
  - OpenAI, Anthropic, and Gemini native reasoning continuity paths
  - explicit limited/no-op semantics for Inception, Ollama, and Sift
  - provider capability matrix, thinking-mode catalogs, tests, and docs
- Out of scope:
  - recursive harness decision policy
  - user-facing display of raw provider reasoning
  - new hosted state services

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│  OpenAI   Anthropic   Gemini   Moonshot -> native continuity      │
│  Inception/Ollama/Sift            -> limited or no-op             │
│                                                                    │
│  capability matrix + thinking modes -> tests -> docs/config       │
│                                     -> operator contract          │
└────────────────────────────────────────────────────────────────────┘
        ↑                                           ↑
  provider APIs/docs                            harness callers
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| OpenAI reasoning-capable transport | external API | Reusable reasoning state for supported OpenAI models | current |
| Anthropic Messages API thinking blocks | external API | Extended/interleaved thinking continuity | current |
| Gemini thought signatures | external API | Stateless continuity handle for thinking models | current |
| Moonshot/Kimi reasoning continuity | external API | Existing native proving path carried forward | current |
| Provider catalog/config docs | internal | Expose actual capability matrix and thinking-mode surface to operators | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Capability authority | `ModelProvider::ALL` must map to explicit deliberation behavior | Avoids silent drift when providers/models change |
| Thinking-mode authority | Every provider/model path must expose supported thinking modes or explicit none | Provider reasoning controls must be discoverable rather than hidden in provider-specific code |
| OpenAI support | Treat reusable reasoning state as transport-dependent rather than universally available | OpenAI reasoning continuity depends on the selected API path |
| Limited providers | Model summary/toggle-only providers explicitly rather than force-fitting native continuation | Some providers expose reasoning controls without reusable state |
| Verification | Ship docs and capability tests with adapter changes | Cross-provider behavior is otherwise too easy to misread |

## Architecture

1. Extend provider capability negotiation so every provider/model path returns an
   explicit deliberation contract and thinking-mode surface.
2. Implement native continuity paths for OpenAI, Anthropic, and Gemini using
   their provider-correct continuation substrate.
3. Preserve Moonshot continuity from Voyage 1 as part of the matrix.
4. Implement explicit limited/no-op handling for Inception, Ollama, and Sift.
5. Back the capability surface with contract tests and synchronized operator
   documentation, including provider thinking-mode catalogs.

## Components

`ProviderCapabilityMatrix`
: Canonical mapping from provider/model to reasoning support level and fallback
mode.

`ProviderThinkingModeCatalog`
: Canonical mapping from provider/model to supported thinking modes or explicit
absence of thinking controls.

`OpenAiReasoningBridge`
: Transport-aware OpenAI continuation layer for models/APIs that support
reusable reasoning state.

`AnthropicThinkingBridge`
: Preserves required `thinking` blocks and interleaved-thinking settings across
tool/result turns.

`GeminiThoughtSignatureBridge`
: Replays returned thought signatures on subsequent tool/function turns.

`LimitedReasoningModes`
: Encodes summary-only, toggle-only, or unsupported behavior for Inception,
Ollama, and Sift.

`ProviderCapabilityMatrixTests`
: Contract tests that verify capability negotiation, thinking-mode exposure,
native continuity, and fallback behavior.

## Interfaces

Candidate internal interfaces:

- `provider_deliberation_capability(provider, model_id, transport) -> Capability`
- `provider_thinking_modes(provider, model_id) -> &[ThinkingMode]`
- `supported_continuation(provider, previous_state, tool_results) -> RequestPatch`
- `limited_reasoning_mode(provider, model_id) -> LimitedReasoningBehavior`
- `render_provider_capability_matrix() -> docs/config artifact`

## Data Flow

1. Provider negotiation evaluates the current provider/model/transport.
2. Native providers produce or consume their continuity substrate:
   - OpenAI reasoning items or prior-response handles
   - Anthropic thinking blocks
   - Gemini thought signatures
   - Moonshot reasoning continuity
3. Limited providers surface explicit behavior instead:
   - Inception summary-only or effort control
   - Ollama think toggle or local model-specific limits
   - Sift unsupported/no-op
4. Thinking-mode catalogs are emitted from the same authority for all provider
   families, including explicit `none` where no thinking control exists.
5. Tests and docs are generated or updated from the same capability authority.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Provider API contract changes and invalidates the continuity bridge | Adapter tests fail or runtime capability checks detect mismatch | Downgrade to explicit limited/no-op behavior and surface the regression | Update the provider adapter and docs together |
| Docs drift from actual capability surface | Matrix review or docs test fails | Block verification until docs/config match behavior | Regenerate or rewrite the provider matrix artifact |
| Unsupported model/provider pair is treated as native-continuation | Capability tests catch incorrect classification | Fail closed and mark the path explicit limited/no-op | Correct negotiation rules before rollout |
| Thinking-mode catalogs drift from actual provider/runtime support | Catalog tests or docs review fail | Block the slice and regenerate or rewrite the thinking-mode surface | Keep mode catalogs driven from the same provider capability source |
