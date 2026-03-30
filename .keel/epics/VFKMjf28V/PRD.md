# Provider Abstraction And Routing - Product Requirements

## Problem Statement

The model layer is hardcoded to local Qwen via SiftAgentAdapter. A provider abstraction needs to route model IDs to the correct adapter at runtime, with a CLI flag to select provider and shared HTTP client infrastructure for API-based providers.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | CLI --provider flag selects which model backend to use | paddles --provider openai --model gpt-4o routes to OpenAI adapter | First voyage |
| GOAL-02 | Factory closures in main.rs route to the correct adapter based on provider | Each provider produces valid Arc<dyn SynthesizerEngine> and Arc<dyn RecursivePlanner> | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles | Choose between local models and API providers based on task requirements |

## Scope

### In Scope

- [SCOPE-01] CLI --provider flag with values: local, openai, anthropic, google, moonshot
- [SCOPE-02] Provider-aware factory closures that route model_id to the correct adapter
- [SCOPE-03] Shared HTTP client configuration (reqwest) for API-based providers
- [SCOPE-04] API key resolution from environment variables per provider
- [SCOPE-05] Optional --provider-url flag for custom API endpoints
- [SCOPE-06] ModelRegistry becomes optional for API providers (no local weight download needed)

### Out of Scope

- [SCOPE-07] Concrete provider adapter implementations (separate epics)
- [SCOPE-08] Provider-specific streaming or tool use features

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | --provider CLI flag accepts local, openai, anthropic, google, moonshot | GOAL-01 | must | Entry point for provider selection |
| FR-02 | Factory closures dispatch to the correct adapter constructor based on provider | GOAL-02 | must | Core routing mechanism |
| FR-03 | API key is resolved from a provider-specific environment variable | GOAL-01 | must | Secure credential handling |
| FR-04 | --provider-url overrides the default API base URL for any provider | GOAL-01 | should | Enables custom endpoints and Moonshot reuse of OpenAI adapter |
| FR-05 | Local provider remains the default when --provider is not specified | GOAL-01 | must | Backwards compatibility |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | All 90 existing tests pass unchanged with the provider abstraction | GOAL-02 | must | No regression to local model path |
| NFR-02 | Provider routing lives in main.rs (composition root), not in the application layer | GOAL-02 | must | Respects hexagonal architecture |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Provider routing | paddles --provider openai --model gpt-4o --help shows correct configuration | CLI flag accepted |
| Backwards compat | cargo nextest run passes | All 90 tests green |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| All API providers can be abstracted behind SynthesizerEngine and RecursivePlanner traits | Would need trait changes | The traits are generic enough: prompt in, response out, action selection |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should provider config live in a file or purely CLI flags | operator | Resolved: CLI flags with env var fallbacks |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] --provider flag is accepted and routes to different adapter constructors
- [ ] Local provider works unchanged as default
- [ ] All existing tests pass
<!-- END SUCCESS_CRITERIA -->
