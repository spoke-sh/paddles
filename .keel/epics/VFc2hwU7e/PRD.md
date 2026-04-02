# Integrate Inception Labs Provider - Product Requirements

## Problem Statement

Paddles can already route OpenAI-compatible remote providers, but it cannot
authenticate, select, or operate Inception Labs models. We need a mission-contained
plan that lands core Mercury-2 chat support first and then stages the
provider-native diffusion and edit features behind explicit later slices.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Operators can authenticate Inception Labs and run `mercury-2` through the normal planner/synthesizer remote-provider path. | `Inception` appears as an enabled provider in `/model` after login and can complete a turn through the HTTP adapter | Core delivery slice |
| GOAL-02 | The board separates core provider compatibility from Inception-native diffusion and edit work so delivery pressure stays on the smallest useful slice. | Optional streaming/edit slices are explicitly decomposed and do not block GOAL-01 | Planned voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles locally with multiple remote providers | Authenticate, select, and use Inception models without bespoke setup or adapter hacks |
| Harness Maintainer | Contributor extending paddles provider capabilities | Land Inception support without destabilizing other providers or the local-first runtime |

## Scope

### In Scope

- [SCOPE-01] Provider catalog, credential, and `/model` surface changes required to recognize Inception as a first-class remote provider
- [SCOPE-02] Mercury-2 chat support through the existing OpenAI-compatible HTTP adapter, including structured outputs and forensic exchange capture
- [SCOPE-03] Documentation and operator defaults for selecting and reasoning about Inception support in paddles
- [SCOPE-04] Optional streaming/diffusion visualization support layered on top of the core provider path
- [SCOPE-05] Optional edit-native endpoint support for future Inception coder/edit models

### Out of Scope

- [SCOPE-06] Upstream `sift` changes as a prerequisite for onboarding the remote provider
- [SCOPE-07] Local Inception inference or non-HTTP transport work
- [SCOPE-08] Changing the global default provider/model as part of the initial provider-enablement slice

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must treat Inception as a first-class authenticated provider in provider selection, credentials, and `/model` visibility. | GOAL-01 | must | Operators need to log in and select the provider before any runtime work matters. |
| FR-02 | Paddles must run `mercury-2` through the existing OpenAI-compatible HTTP adapter, including structured answer normalization and forensic request/response capture. | GOAL-01 | must | Core usable support should reuse the already-proven remote lane. |
| FR-03 | Paddles documentation and operator guidance must explain the supported Inception setup, recommended model, and how core compatibility differs from optional native capabilities. | GOAL-01, GOAL-02 | should | Prevents operator confusion and keeps scope boundaries explicit. |
| FR-04 | The roadmap must include an explicit follow-on slice for streaming/diffusion visualization rather than hiding it inside the initial provider adapter. | GOAL-02 | should | Keeps the core delivery path small while preserving the product direction. |
| FR-05 | The roadmap must include an explicit follow-on slice for edit-native endpoints separate from chat-completions support. | GOAL-02 | should | Edit-native behavior is a different transport/contract and should not be smuggled into the core slice. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Authentication and provider selection for Inception must fail closed when credentials are missing, without breaking existing providers. | GOAL-01 | must | Remote-provider onboarding should not make runtime selection ambiguous or unsafe. |
| NFR-02 | Core Inception support must preserve the existing remote-provider architecture, observability, and forensic capture contracts. | GOAL-01 | must | The provider should fit the current harness instead of creating a one-off side path. |
| NFR-03 | Optional Inception-native slices must not block the Mercury-2 compatibility slice from entering delivery. | GOAL-02 | must | Execution pressure should stay on the smallest useful change. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Provider onboarding | Cargo tests for provider catalog, credentials, runtime preparation, and `/model` behavior | Story-level automated proof from provider/auth slices |
| Mercury-2 execution | Mock-server HTTP adapter tests plus full-turn structured output verification | Story-level automated proof from adapter slice |
| Optional native capabilities | Manual or focused integration verification after the core path is accepted | Follow-on story evidence only when those slices are pulled |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Inception’s current `chat/completions` surface remains OpenAI-compatible enough for the existing adapter shape. | The core slice would require a bespoke transport adapter instead of a provider shim. | Validate during the Mercury-2 adapter story against current docs and mock traces. |
| `mercury-2` is the correct first model target for provider bring-up. | The first slice could chase an edit-native endpoint that is harder to integrate and verify. | Keep edit-native support as a later slice and confirm docs at implementation time. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Whether Inception’s JSON-schema response format matches the exact shape paddles expects for `OpenAiJsonSchema` mode | Epic owner | Open |
| Whether streaming/diffusion visualization belongs in the web UI only or should later appear in the TUI as well | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Operators can authenticate Inception and select `mercury-2` as a provider-backed model in paddles.
- [ ] A planned voyage exists that separates core Mercury-2 support from optional streaming/diffusion and edit-native follow-on slices.
- [ ] The board can pull core Inception stories immediately without requiring upstream `sift` work first.
<!-- END SUCCESS_CRITERIA -->
