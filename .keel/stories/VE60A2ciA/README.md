---
id: VE60A2ciA
title: Migrate Inference to Sift
type: feat
status: done
created_at: 2026-03-16T21:45:15
started_at: 2026-03-16T21:55:00
updated_at: 2026-03-16T21:41:48
operator-signal: 
scope: VE5zxrA1w/VE604BPRi
index: 2
submitted_at: 2026-03-16T21:41:43
completed_at: 2026-03-16T21:41:48
---

# Migrate Inference to Sift

## Summary

Wrap `sift::GenerativeModel` into `wonopcode_provider::LanguageModel` for use in the `PromptLoop`.

## Acceptance Criteria

- [x] `SiftInferenceAdapter` implements `InferenceEngine` by wrapping `sift`. [SRS-27/AC-01] <!-- verify: manual, SRS-27:start:end -->
- [x] CLI executes `just paddles` using the `sift` backend. [SRS-28/AC-01] <!-- verify: manual, SRS-28:start:end -->
