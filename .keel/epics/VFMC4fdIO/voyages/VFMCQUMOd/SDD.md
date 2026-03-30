# Ollama Provider Variant - Software Design Description

> Deliver --provider ollama routing to OpenAI adapter with Ollama defaults

**SRS:** [SRS.md](SRS.md)

## Overview

Zero new adapter code. The Ollama provider is a new enum variant in the CLI provider routing (main.rs) that constructs the existing OpenAI-compatible adapter with http://localhost:11434/v1 as the base URL and passes the --model flag value through unchanged. If OLLAMA_HOST is set, its value replaces the default base URL.

## Context & Boundaries

```
┌──────────────────────────────────────┐
│            main.rs routing           │
│                                      │
│  --provider ollama                   │
│    └── OpenAiAdapter::new(base_url)  │
│         base_url = OLLAMA_HOST       │
│         or http://localhost:11434/v1 │
│         model_id = --model flag      │
└──────────────────────────────────────┘
        ↓
   [Ollama server at localhost:11434]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Ollama | external | Local model server with OpenAI-compatible API | /v1/chat/completions |
| OpenAI adapter | internal | Existing adapter reused with different base URL | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Adapter reuse | Use OpenAI adapter as-is | Ollama's /v1 endpoint is OpenAI-compatible; no new code needed |
| Base URL source | OLLAMA_HOST env var with http://localhost:11434/v1 fallback | Matches Ollama's own OLLAMA_HOST convention |
| Model passthrough | Use --model value directly | Ollama identifies models by name (llama3, qwen2.5) same as OpenAI |

## Architecture

The change is confined to the provider match arm in main.rs. No new modules or files.

```
match provider {
    ...
    Provider::Ollama => {
        let base_url = env::var("OLLAMA_HOST")
            .map(|h| format!("{}/v1", h))
            .unwrap_or("http://localhost:11434/v1".into());
        OpenAiAdapter::new(base_url, model_id, None)
    }
}
```

## Data Flow

1. User runs `paddles --provider ollama --model llama3`
2. CLI parses --provider as `Provider::Ollama`
3. Routing constructs `OpenAiAdapter` with Ollama base URL
4. All subsequent planner/synthesizer calls go through the OpenAI adapter to Ollama's /v1/chat/completions
5. Ollama returns standard OpenAI-format responses

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Ollama not running | Connection refused from adapter | Error message: "Cannot connect to Ollama at {base_url}" | User starts Ollama |
| Model not pulled | Ollama returns 404 | Error propagated from OpenAI adapter | User runs `ollama pull <model>` |
