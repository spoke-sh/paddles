---
id: VE5u5D5YM
title: Execute Real Model Inference
type: feat
status: done
created_at: 2026-03-16T21:10:15
started_at: 2026-03-16T21:20:00
updated_at: 2026-03-16T21:11:24
operator-signal: 
scope: VE5ttmBfz/VE5tzQyo5
index: 3
submitted_at: 2026-03-16T21:11:19
completed_at: 2026-03-16T21:11:24
---

# Execute Real Model Inference

## Summary

Update `CandleAdapter` to use real weights and support model selection via CLI.

## Acceptance Criteria

- [x] CLI supports `--model` argument. [SRS-22/AC-01] <!-- verify: manual, SRS-22:start:end -->
- [x] `CandleAdapter` loads real Gemma or Qwen weights. [SRS-21/AC-01] <!-- verify: manual, SRS-21:start:end -->
- [x] `paddles` generates text from the real model. [SRS-21/AC-02] <!-- verify: manual, SRS-21:start:end -->
