---
id: VE5u5D5YM
title: Execute Real Model Inference
type: feat
status: backlog
created_at: 2026-03-16T21:10:15
updated_at: 2026-03-16T21:07:02
operator-signal: 
scope: VE5ttmBfz/VE5tzQyo5
index: 3
---

# Execute Real Model Inference

## Summary

Update `CandleAdapter` to use real weights and support model selection via CLI.

## Acceptance Criteria

- [ ] CLI supports `--model` argument. [SRS-22/AC-01] <!-- verify: manual, SRS-22:start:end -->
- [ ] `CandleAdapter` loads real Gemma or Qwen weights. [SRS-21/AC-01] <!-- verify: manual, SRS-21:start:end -->
- [ ] `paddles` generates text from the real model. [SRS-21/AC-02] <!-- verify: manual, SRS-21:start:end -->
