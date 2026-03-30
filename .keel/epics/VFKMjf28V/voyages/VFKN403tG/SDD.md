# Provider Routing And CLI Flags - Software Design Description

> Deliver CLI provider flag and factory routing to support multiple model backends

**SRS:** [SRS.md](SRS.md)

## Overview

The provider abstraction is purely a composition-root concern. A new `--provider` CLI flag (clap enum) selects which adapter constructors the factory closures use. The existing SynthesizerFactory and PlannerFactory type signatures remain unchanged — they already accept `(&Path, &str) -> Result<Arc<dyn Trait>>`. The routing logic in main.rs matches on the provider enum and calls the appropriate adapter constructor.

For API-based providers, reqwest is added as a dependency for HTTP client calls. API keys are resolved from environment variables named by convention (OPENAI_API_KEY, ANTHROPIC_API_KEY, GOOGLE_API_KEY, MOONSHOT_API_KEY).

## Architecture

```
main.rs (composition root)
  ├── parses --provider flag
  ├── match provider {
  │     Local => SiftAgentAdapter (existing)
  │     OpenAI => OpenAiAdapter (new)
  │     Anthropic => AnthropicAdapter (new)
  │     Google => GeminiAdapter (new)
  │     Moonshot => OpenAiAdapter with custom base URL (new)
  │   }
  └── constructs Arc<dyn SynthesizerEngine> + Arc<dyn RecursivePlanner>
```

No application or domain layer changes needed.

## Components

### ModelProvider enum (in main.rs or application layer)
Values: Local, OpenAI, Anthropic, Google, Moonshot. Derives clap::ValueEnum.

### Provider-aware factory construction
A function that takes the ModelProvider enum and returns the appropriate SynthesizerFactory and PlannerFactory closures.

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Provider routing location | main.rs composition root | Hexagonal architecture: adapter selection is wiring, not application logic |
| API key source | Environment variables | Standard practice, never hardcoded, works with .env files |
| Moonshot handling | Reuse OpenAI adapter with different base URL | Moonshot is OpenAI-compatible, avoid duplicate code |
| HTTP client | reqwest | Already used transitively, async-compatible with tokio |

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| reqwest | crate | HTTP client for API providers | 0.12 (already transitive) |
| clap | crate | CLI enum for --provider flag | 4.5 (existing) |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Missing API key | Env var lookup returns None | Error at boot before lane preparation | User sets env var |
| Unknown provider value | clap rejects invalid enum variant | CLI help message | User corrects flag |
| API provider selected but network unavailable | reqwest timeout | Error during turn processing | User switches to local or fixes network |
