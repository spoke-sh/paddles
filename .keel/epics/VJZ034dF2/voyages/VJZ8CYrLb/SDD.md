# Route Runtime Inference Through HTTP Model Clients - Software Design Description

> Make action-selection and final-rendering model inference resolve exclusively through HTTP model clients while preserving Sift retrieval independence.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the ADR into runtime construction behavior. Action selection
and final rendering must be backed by HTTP model clients only. Sift retrieval
remains available through its own retrieval/indexing path, but Sift no longer
participates as an inference provider.

## Context & Boundaries

The current runtime factory code branches between Sift inference adapters and
HTTP provider adapters. This voyage removes that branch for inference while
leaving retrieval-specific Sift code alone. Deleting the now-unused adapter files
and dependencies is intentionally left to the next voyage so tests can prove the
HTTP-only boundary first.

```
┌─────────────────────────────────────────┐
│        Turn Runtime Construction        │
│                                         │
│  Action Selection -> HTTP Model Client  │
│  Final Rendering  -> HTTP Model Client  │
│  Retrieval        -> Retrieval Provider │
└─────────────────────────────────────────┘
        ↑               ↑
 Provider config   Credential store
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `HttpProviderAdapter` | Rust adapter | Execute final-rendering HTTP model calls | repo-local |
| `HttpPlannerAdapter` or renamed successor | Rust adapter | Execute action-selection HTTP model calls | repo-local |
| `ModelCapabilitySurface` | Rust domain contract | Negotiate provider transport/schema support | repo-local |
| Sift retrieval adapter | Rust adapter | Preserve retrieval/indexing behavior | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Runtime inference | HTTP-only | There should be one model-client boundary for action selection and final rendering. |
| Local model paths | Removed from inference preparation | Model loading belongs to local HTTP services, not paddles. |
| Retrieval | Preserve and test | Sift retrieval is not the same concern as Sift model inference. |
| Deletion timing | Delete adapter files later | Prove construction behavior first, then remove dead code in the next voyage. |

## Architecture

Runtime preparation should produce turn runtime components from provider
configuration without fetching local model paths for inference. The action
selection and final rendering model clients may still have different prompts,
schemas, and provider options, but both use HTTP transport. Retrieval remains a
separate prepared component.

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| Action-selection model client | Produce tool/action decisions | Built from HTTP provider config |
| Final-rendering model client | Produce terminal responses | Built from HTTP provider config |
| Retrieval provider | Gather context/evidence | May still use Sift retrieval/indexing |
| Provider capability surface | Validate provider support | Reject unsupported HTTP format before runtime |

## Interfaces

Provider selection should accept HTTP-backed model references such as
`ollama:<model>`, `openai:<model>`, or other supported HTTP provider forms.
Legacy `sift` as a model provider is rejected by the compatibility policy.

## Data Flow

1. Runtime config is read and normalized.
2. Model-provider selections are validated against HTTP provider capabilities.
3. Action-selection and final-rendering clients are constructed from HTTP
   provider adapters.
4. Retrieval providers are prepared independently.
5. The turn loop receives model clients, retrieval, tool execution, evidence,
   and final-rendering components without local inference model paths.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Legacy `sift` inference provider selected | Provider validation | Fail with `ollama:<model>` migration hint | Select an HTTP provider |
| HTTP provider lacks required schema/transport support | Capability negotiation | Fail before runtime construction | Select a compatible model/provider |
| Retrieval path depends on deleted inference state | Retrieval tests | Stop the slice and split work | Preserve or refactor retrieval separately |
