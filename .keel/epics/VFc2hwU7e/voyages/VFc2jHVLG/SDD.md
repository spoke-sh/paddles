# Plan Inception Provider Delivery - Software Design Description

> Deliver first-class Inception Labs provider support in staged slices: core
> Mercury-2 chat compatibility first, then optional diffusion streaming and
> edit-native capabilities.

**SRS:** [SRS.md](SRS.md)

## Overview

The voyage should treat Inception as a remote-provider extension of paddles’
existing HTTP adapter architecture, not as a special-case runtime. The core
slice adds a new provider identity, credentials, defaults, and model catalog
entry for `mercury-2`, then routes it through the already-existing
OpenAI-compatible `chat/completions` path with structured outputs and forensic
exchange capture. Two explicit follow-on slices remain separate: one for
streaming/diffusion visualization, and one for edit-native endpoint support.
This preserves execution pressure on the smallest useful delivery path while
keeping the more provider-specific work visible.

## Context & Boundaries

- In scope:
  - `ModelProvider` catalog additions for `Inception`
  - `INCEPTION_API_KEY` credential handling and provider availability reporting
  - `mercury-2` model catalog/defaults and OpenAI-compatible HTTP adapter routing
  - docs/operator guidance for supported setup and selection
  - planned follow-on slices for streaming/diffusion visualization and edit-native endpoints
- Out of scope:
  - upstream `sift` changes
  - local inference/runtime work for Inception models
  - bundling streaming or edit-native execution into the core Mercury-2 slice

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌──────────────┐  ┌──────────────┐    │
│  │ Provider     │  │ HTTP Adapter │    │
│  │ Catalog/Auth │  │ Mercury-2    │    │
│  └──────────────┘  └──────────────┘    │
│          │                  │          │
│  ┌──────────────┐  ┌──────────────┐    │
│  │ Docs/Defaults│  │ Future Native│    │
│  │              │  │ Slices       │    │
│  └──────────────┘  └──────────────┘    │
└─────────────────────────────────────────┘
        ↑               ↑
   [Inception API]  [Paddles Remote Lanes]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Inception API | external service | OpenAI-compatible `chat/completions` endpoint for `mercury-2` | `https://api.inceptionlabs.ai/v1` |
| Existing HTTP adapter | internal runtime | Reuses request/response, rendering, and forensic capture machinery | current `HttpProviderAdapter` |
| Credential store and provider catalog | internal runtime | Authenticate and expose Inception in `/login` and `/model` | current CLI/TUI/provider surfaces |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Core transport | Reuse `ApiFormat::OpenAi` for Mercury-2 | Inception advertises an OpenAI-compatible `chat/completions` surface, so a provider shim is lower-risk than a new adapter |
| Core model target | Start with `mercury-2` | It fits the current chat-completions contract better than edit-native endpoints |
| Optional slices | Keep streaming/diffusion and edit-native support as separate stories | These are different UX/protocol problems and should not block basic provider bring-up |
| Architecture boundary | Keep this work in `paddles`, not `sift` | This is a remote-provider integration, not a local runtime/model-loading concern |

## Architecture

The core architecture change is additive:

1. `providers.rs` gains a new `ModelProvider::Inception` with base URL, env var,
   known model list, and display metadata.
2. `credentials.rs`, CLI/TUI surfaces, and config normalization treat Inception
   like the other authenticated remote providers.
3. `main.rs` maps Inception to `ApiFormat::OpenAi`.
4. `rendering.rs` resolves Inception to `OpenAiJsonSchema` if the API proves
   schema-compatible; otherwise it can fall back to `PromptEnvelope` without
   changing the provider catalog shape.
5. Follow-on stories can extend the web/UI or remote adapter surfaces for
   streaming/diffusion and edit-native endpoints without reopening the core seam.

## Components

- `ModelProvider::Inception`
  Purpose: identify the provider in configuration, model selection, and auth.
  Behavior: advertises base URL, credential env var, and known core model ids.

- `CredentialStore`
  Purpose: resolve `INCEPTION_API_KEY` or stored credentials.
  Behavior: fail closed when missing, while leaving other providers unaffected.

- `HttpProviderAdapter`
  Purpose: execute Mercury-2 through the existing OpenAI-compatible flow.
  Behavior: send chat-completions requests, capture exact provider exchange
  artifacts, and normalize structured final answers.

- Documentation / operator guidance
  Purpose: explain how to use Inception in paddles and what is not yet supported.
  Behavior: call out the difference between core compatibility and future native slices.

## Interfaces

- `INCEPTION_API_KEY` environment variable and `/login inception`
- `ModelProvider::Inception` in config and `/model`
- `https://api.inceptionlabs.ai/v1/chat/completions` through the OpenAI-style adapter contract
- future: streaming/diffusion and edit-native endpoint contracts as dedicated follow-on interfaces

## Data Flow

1. Operator authenticates Inception through environment or local credential store.
2. Provider availability marks Inception enabled in `/model`.
3. Runtime preparation resolves `Inception + mercury-2` into the OpenAI-compatible HTTP adapter.
4. The adapter sends the assembled planner/synthesizer context to Inception,
   captures request/response artifacts, and returns normalized output.
5. Optional later slices attach streaming/diffusion visualization or edit-native
   endpoints to this established provider identity.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Inception credentials missing | provider availability / runtime preparation | fail closed with login-required guidance | operator authenticates and retries |
| Inception schema compatibility differs from current OpenAI JSON-schema assumptions | adapter tests or integration proof fail | fall back to prompt-envelope rendering in the core slice if needed | tighten support in a focused adapter follow-up |
| Optional slices expand scope pressure on core support | planning review / story pull order | leave streaming/edit work as separate backlog slices | pull core stories first and defer native extras |
