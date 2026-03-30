---
# system-managed
id: VFMCtA3y2
status: icebox
created_at: 2026-03-30T06:30:46
updated_at: 2026-03-30T06:30:46
# authored
title: Ollama Provider Enum Variant
type: feat
operator-signal:
scope: VFMC4fdIO/VFMCQUMOd
index: 1
---

# Ollama Provider Enum Variant

## Summary

Add a `Provider::Ollama` enum variant that routes to the existing OpenAI-compatible adapter with `http://localhost:11434/v1` as the default base URL. Support `OLLAMA_HOST` env var override and pass `--model` through unchanged.

## Acceptance Criteria

- [ ] --provider ollama accepted as CLI flag value [SRS-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Ollama variant constructs OpenAI adapter with http://localhost:11434/v1 base URL [SRS-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] OLLAMA_HOST env var overrides default base URL when set [SRS-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] Model ID from --model flag passed through to Ollama API unchanged [SRS-04] <!-- verify: manual, SRS-04:start:end -->
- [ ] No new adapter code introduced; existing OpenAI adapter reused [SRS-NFR-01] <!-- verify: code review, SRS-NFR-01:start:end -->
