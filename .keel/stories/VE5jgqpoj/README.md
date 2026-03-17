---
id: VE5jgqpoj
title: Implement Real Inference Loop
type: feat
status: backlog
created_at: 2026-03-16T20:27:15
updated_at: 2026-03-16T20:25:44
operator-signal: 
scope: VE5jWMShq/VE5jbmios
index: 2
---

# Implement Real Inference Loop

## Summary

Replace the mock response with a real token generation loop using `candle-transformers`.

## Acceptance Criteria

- [ ] `CandleProvider` generates text from a real model. [SRS-13/AC-01] <!-- verify: manual, SRS-13:start:end -->
- [ ] Text is streamed back to the `PromptLoop`. [SRS-13/AC-02] <!-- verify: manual, SRS-13:start:end -->
