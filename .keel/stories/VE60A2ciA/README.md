---
id: VE60A2ciA
title: Migrate Inference to Sift
type: feat
status: backlog
created_at: 2026-03-16T21:45:15
updated_at: 2026-03-16T21:31:17
operator-signal: 
scope: VE5zxrA1w/VE604BPRi
index: 2
---

# Migrate Inference to Sift

## Summary

Wrap `sift::GenerativeModel` into `wonopcode_provider::LanguageModel` for use in the `PromptLoop`.

## Acceptance Criteria

- [ ] `SiftInferenceAdapter` implements `InferenceEngine` by wrapping `sift`. [SRS-27/AC-01] <!-- verify: manual, SRS-27:start:end -->
- [ ] CLI executes `just paddles` using the `sift` backend. [SRS-28/AC-01] <!-- verify: manual, SRS-28:start:end -->
