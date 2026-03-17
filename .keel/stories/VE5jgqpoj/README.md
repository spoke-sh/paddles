---
id: VE5jgqpoj
title: Implement Real Inference Loop
type: feat
status: done
created_at: 2026-03-16T20:27:15
started_at: 2026-03-16T20:30:00
updated_at: 2026-03-16T20:28:47
operator-signal: 
scope: VE5jWMShq/VE5jbmios
index: 2
submitted_at: 2026-03-16T20:28:34
completed_at: 2026-03-16T20:28:47
---

# Implement Real Inference Loop

## Summary

Replace the mock response with a real token generation loop using `candle-transformers`.

## Acceptance Criteria

- [x] `CandleProvider` generates text from a real model. [SRS-13/AC-01] <!-- verify: manual, SRS-13:start:end -->
- [x] Text is streamed back to the `PromptLoop`. [SRS-13/AC-02] <!-- verify: manual, SRS-13:start:end -->
