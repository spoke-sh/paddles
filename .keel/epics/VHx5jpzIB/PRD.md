# OpenAI GPT-5.5 Catalog Support - Product Requirements

## Problem Statement

The OpenAI provider catalog does not list the newly available `gpt-5.5` and `gpt-5.5-pro` models, and existing OpenAI text/reasoning `*-pro` models are not exposed consistently through the supported model surfaces. Operators cannot reliably select the current OpenAI frontier and pro model IDs through `/model` even when the HTTP adapter can route them.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Expose current OpenAI GPT-5.5 and supported text/reasoning pro model IDs through the provider catalog. | Catalog tests enumerate the supported OpenAI model IDs. | `gpt-5.5`, `gpt-5.5-pro`, `gpt-5.4-pro`, `gpt-5.2-pro`, `gpt-5-pro`, `o3-pro`, and `o1-pro` are accepted. |
| GOAL-02 | Preserve correct chat-completions and Responses routing for the new model IDs. | Capability-surface tests cover normal GPT-5.5 and Responses-only pro paths. | Pro paths use prompt-envelope Responses continuity; GPT-5.5 thinking uses Responses when enabled. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | A Paddles user configuring remote OpenAI lanes from the CLI, TUI, or persisted runtime preferences. | A selectable model list that reflects currently available OpenAI coding and reasoning models without bypassing validation. |

## Scope

### In Scope

- [SCOPE-01] OpenAI provider catalog entries for `gpt-5.5`, `gpt-5.5-pro`, and current OpenAI text/reasoning `*-pro` model IDs.
- [SCOPE-02] Thinking-mode and capability-surface behavior for the newly exposed model IDs.
- [SCOPE-03] Configuration documentation for the updated representative OpenAI capability paths.

### Out of Scope

- [SCOPE-04] Video/media generation models such as Sora Pro that are not usable through the existing coding-harness chat/responses adapter.
- [SCOPE-05] Changing default configured models or authored user lane preferences.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Publish current OpenAI GPT-5.5 and supported text/reasoning pro model IDs from `ModelProvider::Openai.known_model_ids()`. | GOAL-01 | must | The model selector and validation path consume the provider catalog. |
| FR-02 | Classify `gpt-5.5` thinking modes and pro model routing through the negotiated capability surface. | GOAL-02 | must | The harness needs accurate Responses routing for reasoning and pro paths. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Keep model catalog changes covered by focused automated provider tests. | GOAL-01, GOAL-02 | must | Provider catalog regressions should fail before runtime selection breaks. |
| NFR-02 | Keep the configuration documentation synchronized with generated provider capability matrix output. | GOAL-02 | must | Operators need docs that match runtime capability negotiation. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Catalog outcome | Rust provider catalog tests | Story evidence for `openai_provider_exposes_additional_model_ids` |
| Routing outcome | Rust capability/thinking tests | Story evidence for OpenAI thinking and Responses-only pro paths |
| Documentation outcome | Generated matrix embedding test | Story evidence for `configuration_docs_embed_current_provider_capability_matrix` |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Official OpenAI model documentation is the source for available text/reasoning pro IDs. | Paddles may expose stale or unsupported IDs. | Verify against OpenAI's model catalog before implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should Sora Pro be exposed through this provider catalog? | Epic owner | Closed: out of scope until a media-generation adapter exists. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `ModelProvider::Openai` accepts the current OpenAI GPT-5.5 family and supported text/reasoning pro model IDs.
- [ ] Capability-surface tests prove pro models use the Responses-oriented prompt-envelope path.
- [ ] Configuration docs embed the current generated provider capability matrix.
<!-- END SUCCESS_CRITERIA -->
