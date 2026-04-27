# Expose Current OpenAI Pro Models - Software Design Description

> Operators can select current OpenAI GPT-5.5 and pro model ids from the supported model catalog with correct transport capabilities.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage updates the OpenAI provider model catalog and the negotiated
capability surface that downstream CLI, TUI, persisted preferences, and HTTP
adapter paths already consume. The implementation keeps model availability,
thinking modes, and Responses-only behavior centralized in
`src/infrastructure/providers.rs`.

## Context & Boundaries

OpenAI's current model catalog lists GPT-5.5 as the flagship coding/reasoning
model and lists GPT-5.5 pro, GPT-5.4 pro, GPT-5.2 pro, GPT-5 pro, o3-pro, and
o1-pro as text/reasoning pro paths. Paddles should expose those IDs only where
the existing OpenAI chat/responses harness can use them.

Media-generation pro models remain out of scope because they do not fit the
coding harness' text chat/responses contract.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| OpenAI model docs | External reference | Source of current model IDs and reasoning effort support | API docs, verified during implementation |
| `ModelProvider` catalog | Internal module | Accepted model IDs, thinking modes, capability surface | `src/infrastructure/providers.rs` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Catalog source | Add text/reasoning OpenAI pro IDs to `OPENAI_MODELS` rather than only transport allow-lists. | `/model` validation and selectable IDs consume `known_model_ids()`. |
| Pro routing | Keep Responses-continuity pro IDs in an explicit OpenAI Responses-only/pro list. | Pro paths need prompt-envelope rendering/planning and opaque continuation state. |
| GPT-5.5 thinking | Use `none`, `low`, `medium`, `high`, `xhigh`. | Matches current OpenAI GPT-5.5 model documentation. |
| Media pro models | Exclude Sora Pro. | The existing adapter is a coding/reasoning chat harness, not a media generation API. |

## Architecture

The provider catalog is the domain-facing registry for remote model
selection. Application and UI surfaces call `ModelProvider::known_model_ids()`,
`selectable_model_ids()`, and `thinking_modes()` instead of hard-coding model
IDs. Runtime HTTP behavior calls `capability_surface()` to decide wire format,
rendering contract, planner tool-call strategy, and deliberation state.

## Components

- `OPENAI_MODELS`: exposed OpenAI model IDs accepted by validation and model
  selection.
- `OPENAI_RESPONSES_ONLY_MODELS`: OpenAI model IDs that should use
  prompt-envelope Responses behavior and native continuation state.
- `openai_thinking_modes`: maps selectable OpenAI model IDs to documented
  reasoning effort controls.
- `DOCUMENTED_PROVIDER_CAPABILITY_PATHS`: representative capability matrix rows
  rendered into `CONFIGURATION.md`.

## Interfaces

No public API shape changes. Existing selectors and runtime preference files may
now contain the newly accepted model IDs. Existing runtime IDs with
`@@thinking=<effort>` continue to represent selected OpenAI thinking modes.

## Data Flow

1. Operator selects a provider/model through `/model` or persisted runtime lane
   state.
2. Paddles validates the model with `ModelProvider::accepts_model()`.
3. Paddles resolves thinking modes and prepares runtime model IDs.
4. The HTTP adapter consumes `capability_surface()` to choose Chat Completions or
   Responses-compatible behavior.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| A new model ID is missing from the catalog | Provider catalog test fails | Add the ID to the catalog or document why it is out of scope | Re-run provider tests |
| A pro model uses the wrong render/planner path | Capability-surface test fails | Update the Responses/pro classification | Re-run capability and docs tests |
| Documentation matrix drifts from runtime behavior | Docs embedding test fails | Regenerate or update the marked matrix section | Re-run docs embedding test |
