# Adopt HTTP-Only Inference Decision - Software Design Description

> Record the HTTP-only inference ADR, compatibility policy, and explicit decision that legacy Sift model-provider config hard-fails with an Ollama HTTP migration hint.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage records the architectural reversal before runtime code is deleted.
It adds the ADR and compatibility policy that make the later implementation
slices safe: paddles no longer owns model loading for inference, local-first
models run through HTTP services such as Ollama, and legacy Sift model-provider
configuration fails explicitly instead of being silently remapped.

## Context & Boundaries

The voyage is limited to decision and compatibility-policy groundwork. It may
add the first failing compatibility tests and doc checks, but it does not remove
runtime adapters yet. Sift retrieval/indexing remains outside this decision.

```
┌─────────────────────────────────────────┐
│      HTTP-Only Inference Decision       │
│                                         │
│  ADR -> config failure policy -> docs   │
│                 |                       │
│                 v                       │
│        Later HTTP-only factories        │
└─────────────────────────────────────────┘
        ↑               ↑
  Research map     Human decisions
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `.keel/adrs/` | board artifact | Store the binding architecture decision | repo-local |
| `CONFIGURATION.md` | project doc | Own provider config and migration hint language | repo-local |
| `ARCHITECTURE.md` | project doc | Own runtime boundary narrative | repo-local |
| Existing config tests | Rust tests | Prove old Sift provider selections fail deterministically | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Inference boundary | HTTP model clients only | Model loading, residency, batching, and hardware placement are separate concerns from the paddles turn loop. |
| Local-first path | `ollama:<model>` style HTTP providers | Local execution remains possible without paddles loading models in-process. |
| Legacy Sift provider | Hard failure with migration hint | Silent remapping would hide provider, model, latency, and deployment changes. |
| Sift retrieval | Out of scope | Retrieval/indexing has a different blast radius and remains a later decision. |

## Architecture

The ADR is the authoritative source for the migration direction. Runtime code
will later enforce that direction by removing Sift inference branches; this
voyage only creates the policy and first guardrails.

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| ADR | Bind the architecture decision | States HTTP-only inference and Sift retrieval separation |
| Compatibility tests | Pin legacy config behavior | Prove `sift` inference config fails before runtime construction |
| Docs | Explain operator migration | Point users toward local HTTP services and `ollama:<model>` |

## Interfaces

The migration hint should be suitable for CLI/config errors:

`provider 'sift' no longer performs model inference; run a local HTTP model service such as Ollama and select 'ollama:<model>'.`

## Data Flow

1. Operator supplies legacy Sift model-provider config.
2. Config parsing preserves enough legacy shape to identify the old provider.
3. Runtime preparation rejects the provider before model construction.
4. The error points to `ollama:<model>` and the ADR-backed docs.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Legacy Sift model provider selected | Config/provider validation | Return deterministic migration error | Configure an HTTP provider such as `ollama:<model>` |
| Sift retrieval mistaken for Sift inference | Retrieval boundary tests | Keep retrieval path green and unchanged | Defer retrieval decision to later mission |
| Docs imply in-process model loading remains supported | Doc check or review | Update owning docs in this voyage | Point to HTTP-only ADR |
