# Registry Auth and Defaults - Product Requirements

## Problem Statement

The current default model (Gemma-2B) is gated on Hugging Face, requiring users to have an account and a token just to run `just paddles` for the first time. This creates a high barrier to entry. Additionally, the system provides no mechanism to supply an authentication token for gated models.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Smooth Out-of-the-box Experience | `just paddles` succeeds without manual token config | 100% |
| GOAL-02 | Gated Model Support | Users can supply an HF token to access models like Gemma | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| New User | First-time pilot of the mech suit | Immediate success running the default model |
| Power User | Advanced pilot | Ability to use their own models and tokens |

## Scope

### In Scope

- [SCOPE-01] Changing default model to `qwen-1.5b`.
- [SCOPE-02] Adding `--hf-token` CLI argument.
- [SCOPE-03] Supporting `HF_TOKEN` environment variable.

### Out of Scope

- [SCOPE-04] Managing multiple Hugging Face accounts.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Default model must be a non-gated model family (e.g. Qwen). | GOAL-01 | must | Eliminates 401 errors for new users. |
| FR-02 | `HFHubAdapter` must accept an optional token. | GOAL-02 | must | Enables gated model access. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Tokens must not be logged or printed. | GOAL-02 | must | Security invariant. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- CLI Proof: `just paddles` succeeds (after clearing local cache if needed).
- CLI Proof: `paddles --hf-token <token> --model gemma-2b` succeeds.

## Assumptions

- Qwen-1.5B remains non-gated on the Hugging Face Hub.

## Open Questions & Risks

- Risk: Rate limiting on HF Hub without a token even for public models.

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `just paddles` boots and syncs without error by default.
- [ ] Gated models are accessible when a token is provided.
<!-- END SUCCESS_CRITERIA -->
