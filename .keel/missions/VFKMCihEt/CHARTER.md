# Multi Provider Model Support - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Model provider is selectable at runtime via CLI flag | board: VFKMjf28V |
| MG-02 | OpenAI-compatible API provider works as both planner and synthesizer (also covers Moonshot) | board: VFKMlkWBt |
| MG-03 | Anthropic Claude provider works as both planner and synthesizer | board: VFKMmuJFY |
| MG-04 | Google Gemini provider works as both planner and synthesizer | board: VFKMo6YJb |
| MG-05 | Ollama provider routes to OpenAI-compatible adapter with localhost:11434/v1 default | board: VFMC4fdIO |
| MG-06 | mistral.rs available as a separate local inference provider alongside sift | board: VFMC5HaQY |

## Constraints

- New providers are infrastructure adapters implementing existing domain ports (SynthesizerEngine, RecursivePlanner)
- No domain or application layer changes required beyond the provider routing
- API keys are provided via environment variables, never hardcoded
- The same prompt templates and action schemas work across all providers
- Local-first remains the default when no --provider flag is specified

## Halting Rules

- HALT when all epics (VFKMjf28V, VFKMlkWBt, VFKMmuJFY, VFKMo6YJb, VFMC4fdIO, VFMC5HaQY) are verified
- YIELD to human if any provider requires architectural changes beyond the infrastructure layer
