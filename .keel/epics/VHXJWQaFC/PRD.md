# Provider Reasoning Integration And Rationale Boundaries - Product Requirements

## Problem Statement

Paddles does not yet treat provider-native reasoning as a first-class execution substrate across adapters. As a result, multi-step reasoning continuity, uncertainty/evidence signals, and rationale boundaries are inconsistent across providers, and the recursive harness cannot benefit from native provider deliberation without risking architectural leakage into paddles-owned control semantics.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Provider-native deliberation substrate: every supported provider/model path advertises an explicit reasoning capability and can preserve native deliberation state or a safe no-op equivalent across recursive turns. | `ModelProvider::ALL` maps to an explicit deliberation capability and at least one provider-native continuation path is verified end-to-end | Voyage 1 and Voyage 2 |
| GOAL-02 | Harness-safe normalization: provider-native reasoning is translated into provider-agnostic deliberation signals the recursive harness can use for better branch, refine, retry, and stop decisions. | Recursive planner decisions can consume normalized signals without matching on provider-specific payloads | Voyage 3 |
| GOAL-03 | Rationale boundary integrity: paddles continues to emit its own concise, auditable `rationale` while raw provider reasoning remains adapter-scoped or debug-scoped. | Canonical transcript/render/projection state never depends on raw provider reasoning content | Voyage 1 and Voyage 3 |
| GOAL-04 | Cross-provider operational coherence: docs, configuration, thinking-mode selection, tests, and fallback behavior are explicit for `sift`, `openai`, `inception`, `anthropic`, `google`, `moonshot`, and `ollama`. | Maintainers can inspect one capability matrix, one thinking-mode surface, and one contract-test suite to understand reasoning behavior for every provider | Voyage 2 |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles through the TUI, web runtime, or CLI while watching recursive execution live. | Better recursive behavior without opaque provider-specific surprises or leaking raw thought traces into normal output. |
| Runtime Maintainer | Engineer evolving adapters, planner loops, and surface projections. | A stable contract separating provider-native reasoning mechanics from paddles-owned rationale and turn state. |
| Provider Integrator | Engineer adding or updating a model provider. | One explicit place to declare what reasoning continuity, signal extraction, and fallbacks the provider supports. |

## Scope

### In Scope

- [SCOPE-01] Provider capability negotiation for native reasoning/continuation support, summary-only support, or explicit unsupported/no-op behavior
- [SCOPE-02] Adapter-owned deliberation state that can carry provider-native reasoning continuity across tool/result turns
- [SCOPE-03] Provider-specific continuation implementations for supported providers and explicit fallback semantics for unsupported ones
- [SCOPE-04] Application-owned normalized deliberation signals derived from provider-native reasoning artifacts
- [SCOPE-05] Recursive harness changes that use normalized deliberation signals while preserving paddles-authored rationale
- [SCOPE-06] Debug/forensic boundaries for provider-native reasoning artifacts that avoid contaminating canonical transcript/render state
- [SCOPE-07] Cross-provider tests, capability matrices, thinking-mode catalogs, configuration guidance, and architectural documentation

### Out of Scope

- [SCOPE-08] Exposing raw chain-of-thought or provider-native reasoning text in default user-facing transcript/stream surfaces
- [SCOPE-09] Replacing paddles rationale with provider-native explanations
- [SCOPE-10] New hosted orchestration or remote session services to compensate for local adapter limitations
- [SCOPE-11] Broad prompt-tuning work unrelated to reasoning continuity, signal extraction, or rationale boundaries

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Every supported provider/model path must advertise an explicit deliberation capability that tells the harness whether native reasoning continuity is available, summary-only, toggle-only, or unsupported. | GOAL-01, GOAL-04 | must | Eliminates implicit provider behavior and gives the runtime one capability surface instead of provider-name conditionals. |
| FR-02 | Provider-native reasoning state must be carried as adapter-owned or application-normalized deliberation state, separate from `AuthoredResponse`, transcript projection, and paddles `rationale`. | GOAL-01, GOAL-03 | must | Keeps provider thought-state useful for execution without polluting canonical turn truth. |
| FR-03 | Providers that expose reusable reasoning artifacts must preserve them across tool/result turns using the provider-correct transport or payload contract. | GOAL-01 | must | Native reasoning continuity is only valuable if the adapter can feed the right substrate back during recursive execution. |
| FR-04 | Providers without reusable reasoning continuity must expose explicit limited or no-op behavior rather than silently discarding the distinction. | GOAL-01, GOAL-04 | must | Safe degradation is part of the contract; unsupported providers should still be predictable. |
| FR-05 | The application layer must translate provider-native reasoning artifacts into a normalized deliberation-signal contract the recursive harness can consume without matching on provider payload shapes. | GOAL-02 | must | Preserves hexagonal boundaries and keeps control logic provider-agnostic. |
| FR-06 | The recursive harness must be able to use normalized deliberation signals to improve branch, refine, retry, and stop decisions. | GOAL-02 | should | Provider-native reasoning should improve the harness, not merely be stored. |
| FR-07 | Paddles must continue to emit its own concise `rationale` derived from chosen actions, evidence, and normalized signals rather than raw provider reasoning content. | GOAL-03 | must | `rationale` is the auditable control artifact and must remain stable across providers. |
| FR-08 | Every supported provider/model path must expose supported thinking modes or an explicit `none`/unsupported result through provider catalogs, configuration, and runtime negotiation. | GOAL-01, GOAL-04 | must | Thinking-mode selection is part of the provider reasoning contract and must not remain hard-coded to only a subset of providers. |
| FR-09 | Operator docs, configuration guidance, and contract tests must describe the reasoning behavior, thinking modes, and fallback mode for every supported provider. | GOAL-04 | must | Cross-provider reasoning support is otherwise too easy to misinterpret or regress. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The integration must preserve local-first execution and add no new hosted dependency for reasoning continuity, signal extraction, or rationale correctness. | GOAL-01, GOAL-02, GOAL-03 | must | Architectural cleanup cannot violate the local-first runtime contract. |
| NFR-02 | Raw provider reasoning must never become the canonical transcript/render/replay artifact; any persisted provider reasoning must be clearly debug-scoped, bounded, and redactable. | GOAL-03 | must | Prevents privacy, storage, and replay correctness problems. |
| NFR-03 | Unsupported or partially supported providers must degrade explicitly and safely, with tests covering fallback semantics. | GOAL-01, GOAL-04 | must | The capability matrix is only useful if degraded paths are deliberate and observable. |
| NFR-04 | The cross-provider contract must be testable through deterministic adapter tests and harness-level decision tests that cover both native and no-op reasoning providers. | GOAL-02, GOAL-04 | must | Prevents regressions in the exact area this epic is trying to standardize. |
| NFR-05 | Provider-specific artifacts and semantics must remain behind adapter or application-normalization seams rather than leaking into domain or UI presentation layers. | GOAL-01, GOAL-02, GOAL-03 | must | Preserves DDD/hexagonal coherence while adding provider sophistication. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Provider substrate | Adapter contract tests plus capability-surface review | Story-level tests and review artifacts |
| Cross-provider rollout | Provider matrix tests, docs review, and fallback-path verification | Story-level test logs and review notes |
| Harness decisions | Recursive-loop tests comparing signal-aware decisions against no-signal baselines | Story-level decision-path evidence |
| Rationale boundary | Transcript/trace review plus tests proving canonical turn state excludes raw provider reasoning | Story-level review and targeted tests |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| At least some providers expose reusable reasoning artifacts or equivalent continuity handles that are worth carrying across tool turns. | The epic may devolve into only configuration flags and no meaningful substrate reuse. | Validate in Voyage 1 and Voyage 2. |
| The recursive harness can improve from normalized signals without depending on raw provider reasoning text. | The signal model may be too weak and require redesign. | Validate in Voyage 3. |
| Provider-native reasoning can remain outside canonical transcript/render state without losing operator value. | We may need a richer debug/forensic surface than currently planned. | Validate in Voyage 1 and Voyage 3. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should OpenAI reasoning support remain limited to currently supported transports, or should the epic include a Responses API migration for reasoning-capable models? | Provider/runtime owner | Open |
| Which normalized deliberation signals are reliable enough to drive branch/refine/stop decisions without overfitting to one provider? | Harness owner | Open |
| How much debug visibility into provider-native reasoning is useful before it becomes noisy or policy-sensitive? | Runtime UX owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Every supported provider in `ModelProvider::ALL` advertises an explicit deliberation capability, fallback mode, and thinking-mode surface.
- [ ] At least one provider-native reasoning continuity path and one explicit no-op path are verified end-to-end through the recursive harness.
- [ ] The recursive harness can consume normalized deliberation signals without depending on provider-specific payload shapes.
- [ ] Canonical transcript/render/projection state remains free of raw provider reasoning while paddles `rationale` stays concise and auditable.
- [ ] At least one active implementation slice exists with story-level verification paths for provider substrate, cross-provider rollout, thinking-mode coverage, and rationale-boundary behavior.
<!-- END SUCCESS_CRITERIA -->
