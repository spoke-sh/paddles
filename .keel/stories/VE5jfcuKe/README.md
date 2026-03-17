---
id: VE5jfcuKe
title: Implement Model Loading
type: feat
status: backlog
created_at: 2026-03-16T20:27:15
updated_at: 2026-03-16T20:25:44
operator-signal: 
scope: VE5jWMShq/VE5jbmios
index: 1
---

# Implement Model Loading

## Summary

Implement the logic to load model weights, tokenizer, and config from local paths using `candle`.

## Acceptance Criteria

- [ ] `CandleProvider` successfully loads a model from disk. [SRS-12/AC-01] <!-- verify: manual, SRS-12:start:end -->
- [ ] Loading completion is traced with timing. [SRS-NFR-06/AC-01] <!-- verify: manual, SRS-NFR-06:start:end -->
