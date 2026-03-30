# OpenAI Chat Completions Adapter - Software Design Description

> Deliver OpenAI-compatible HTTP adapter implementing SynthesizerEngine and RecursivePlanner

**SRS:** [SRS.md](SRS.md)

## Overview

Reqwest-based HTTP adapter in `src/infrastructure/adapters/openai_provider.rs`. Implements `SynthesizerEngine` by building chat messages from prompts and evidence, sending to `/v1/chat/completions`, and parsing the response. Implements planner methods by parsing structured JSON from model output using the same prompt templates as the local adapter. The `ConversationFactory` pattern is not needed since each API call is stateless.

## Context & Boundaries

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │      OpenAiProvider               │  │
│  │  (SynthesizerEngine + Planner)    │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
        ↑               ↑
   [paddles core]   [OpenAI API]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| reqwest | Library | HTTP client | latest |
| OpenAI API | Service | Chat completions endpoint | v1 |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Stateless calls | No ConversationFactory | Each API request is independent; no local model state to manage |
| Configurable base URL | --provider-url flag | Supports OpenAI-compatible providers (Azure, local proxies) |

## Architecture

`OpenAiProvider` struct holds a `reqwest::Client`, base URL, and API key. It implements both `SynthesizerEngine` and `RecursivePlanner` traits by constructing JSON request bodies and parsing JSON responses.

## Components

**OpenAiProvider** -- Builds chat completion requests with system/user/assistant message arrays, sends POST to `{base_url}/v1/chat/completions`, extracts `choices[0].message.content` from the response.

## Interfaces

Request: `POST /v1/chat/completions` with `Authorization: Bearer {api_key}` header. Body contains `model`, `messages` array, and optional parameters.

Response: JSON with `choices[0].message.content` containing either synthesized text or structured JSON for planner actions.

## Data Flow

1. Paddles core calls `SynthesizerEngine` or `RecursivePlanner` method
2. Adapter builds messages array from prompt templates and evidence
3. Adapter sends POST request to OpenAI-compatible endpoint
4. Response JSON parsed; content extracted from choices array
5. For planner: content parsed as structured JSON into action types

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing API key | Env var not set | Error at construction | User sets env var |
| HTTP error | Non-2xx status | Propagate error with status code | Retry or user corrects config |
| Malformed response | JSON parse failure | Error with context | Log response body for debugging |
