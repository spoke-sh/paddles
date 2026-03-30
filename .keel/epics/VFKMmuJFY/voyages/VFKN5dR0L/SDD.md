# Anthropic Messages API Adapter - Software Design Description

> Deliver Anthropic Claude adapter implementing SynthesizerEngine and RecursivePlanner

**SRS:** [SRS.md](SRS.md)

## Overview

Reqwest-based HTTP adapter in `src/infrastructure/adapters/anthropic_provider.rs`. Uses the Anthropic messages API with system prompt as a top-level parameter. Maps paddles prompt templates to user/assistant content blocks. Parses text response for structured JSON planner actions.

## Context & Boundaries

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │     AnthropicProvider             │  │
│  │  (SynthesizerEngine + Planner)    │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
        ↑               ↑
   [paddles core]   [Anthropic API]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| reqwest | Library | HTTP client | latest |
| Anthropic API | Service | Messages endpoint | 2023-06-01+ |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| System prompt placement | Top-level `system` parameter | Anthropic API convention; system is not a message role |
| Auth header | `x-api-key` header | Anthropic uses custom header rather than Bearer token |

## Architecture

`AnthropicProvider` struct holds a `reqwest::Client`, API key, and model name. It implements both `SynthesizerEngine` and `RecursivePlanner` by constructing messages API requests and parsing text responses.

## Components

**AnthropicProvider** -- Builds messages API requests with top-level `system` string and `messages` array of user/assistant content blocks. Sends POST to `https://api.anthropic.com/v1/messages`. Extracts `content[0].text` from the response.

## Interfaces

Request: `POST /v1/messages` with `x-api-key` and `anthropic-version` headers. Body contains `model`, `system`, `messages` array, and `max_tokens`.

Response: JSON with `content[0].text` containing synthesized text or structured JSON for planner actions.

## Data Flow

1. Paddles core calls `SynthesizerEngine` or `RecursivePlanner` method
2. Adapter maps prompts to system parameter and user/assistant content blocks
3. Adapter sends POST request with Anthropic-specific headers
4. Response JSON parsed; text extracted from content array
5. For planner: text parsed as structured JSON into action types

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing API key | ANTHROPIC_API_KEY not set | Error at construction | User sets env var |
| HTTP error | Non-2xx status | Propagate error with status and error type | Retry or user corrects config |
| Malformed response | JSON parse failure | Error with context | Log response body for debugging |
