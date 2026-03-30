# Gemini GenerateContent Adapter - Software Design Description

> Deliver Google Gemini adapter implementing SynthesizerEngine and RecursivePlanner

**SRS:** [SRS.md](SRS.md)

## Overview

Reqwest-based HTTP adapter in `src/infrastructure/adapters/gemini_provider.rs`. Maps prompts to Gemini `contents`/`parts` format. API key passed as `?key=` query parameter. Parses `candidates[0].content.parts[0].text` for the response.

## Context & Boundaries

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │       GeminiProvider              │  │
│  │  (SynthesizerEngine + Planner)    │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
        ↑               ↑
   [paddles core]   [Gemini API]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| reqwest | Library | HTTP client | latest |
| Gemini API | Service | generateContent endpoint | v1beta/v1 |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Auth mechanism | API key as query param | Gemini convention; no Authorization header needed |
| Content mapping | contents/parts arrays | Gemini uses nested parts structure rather than flat messages |

## Architecture

`GeminiProvider` struct holds a `reqwest::Client`, API key, and model name. It implements both `SynthesizerEngine` and `RecursivePlanner` by constructing generateContent requests and parsing candidates from the response.

## Components

**GeminiProvider** -- Builds generateContent requests with `contents` array containing `parts` objects. Sends POST to `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={api_key}`. Extracts `candidates[0].content.parts[0].text` from the response.

## Interfaces

Request: `POST /v1beta/models/{model}:generateContent?key={api_key}`. Body contains `contents` array with role and parts.

Response: JSON with `candidates[0].content.parts[0].text` containing synthesized text or structured JSON for planner actions.

## Data Flow

1. Paddles core calls `SynthesizerEngine` or `RecursivePlanner` method
2. Adapter maps prompts to Gemini contents/parts format
3. Adapter sends POST request with API key as query parameter
4. Response JSON parsed; text extracted from candidates array
5. For planner: text parsed as structured JSON into action types

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing API key | GOOGLE_API_KEY not set | Error at construction | User sets env var |
| HTTP error | Non-2xx status | Propagate error with status code | Retry or user corrects config |
| Malformed response | JSON parse failure or empty candidates | Error with context | Log response body for debugging |
