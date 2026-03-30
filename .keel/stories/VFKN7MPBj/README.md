---
# system-managed
id: VFKN7MPBj
status: icebox
created_at: 2026-03-29T22:58:52
updated_at: 2026-03-29T22:58:52
# authored
title: OpenAI Compatible HTTP Adapter
type: feat
operator-signal:
scope: VFKMlkWBt/VFKN5cgzb
index: 1
---

# OpenAI Compatible HTTP Adapter

## Summary

Deliver a reqwest-based OpenAI chat completions adapter that implements SynthesizerEngine and RecursivePlanner with configurable base URL and env-var API key.

## Acceptance Criteria

- [x] OpenAI adapter implements SynthesizerEngine <!-- verify: manual, SRS-01:AC-01 -->
- [x] OpenAI adapter implements RecursivePlanner <!-- verify: manual, SRS-02:AC-02 -->
- [x] Base URL is configurable <!-- verify: manual, SRS-03:AC-03 -->
- [x] API key from env var <!-- verify: manual, SRS-04:AC-04 -->
