# Multi Provider Model Support - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Model provider is selectable at runtime via CLI flag (--provider local/openai/anthropic/google/moonshot) | board: epic delivering provider abstraction and CLI routing |
| MG-02 | OpenAI-compatible API provider works as both planner and synthesizer | board: epic delivering OpenAI adapter with chat completions |
| MG-03 | Anthropic Claude provider works as both planner and synthesizer | board: epic delivering Anthropic adapter with messages API |
| MG-04 | Google Gemini provider works as both planner and synthesizer | board: epic delivering Google adapter with generateContent API |
| MG-05 | Moonshot Kimi provider works as both planner and synthesizer | board: epic delivering Moonshot adapter (OpenAI-compatible) |
| MG-06 | Local Qwen provider continues to work unchanged as the default | board: existing tests pass with no regression |
| MG-07 | Ollama provider routes to OpenAI-compatible adapter with localhost:11434/v1 default | board: epic delivering Ollama provider variant |
| MG-08 | mistral.rs replaces Qwen-specific Candle code for native multi-architecture local inference | board: epic delivering mistral.rs integration |

## Constraints

- New providers are infrastructure adapters implementing existing domain ports (SynthesizerEngine, RecursivePlanner)
- No domain or application layer changes required beyond the provider routing
- API keys are provided via environment variables, never hardcoded
- The same prompt templates and action schemas work across all providers
- Local-first remains the default when no --provider flag is specified

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
