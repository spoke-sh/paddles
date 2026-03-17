---
id: VE5xHxg36
title: Stabilize Registry Access
type: feat
status: backlog
created_at: 2026-03-16T21:30:15
updated_at: 2026-03-16T21:19:48
operator-signal: 
scope: VE5x7tOAX/VE5xDYVO9
index: 1
---

# Stabilize Registry Access

## Summary

Switch the default model to a non-gated one and add support for Hugging Face authentication tokens.

## Acceptance Criteria

- [ ] Default model is set to `qwen-1.5b`. [SRS-23/AC-01] <!-- verify: manual, SRS-23:start:end -->
- [ ] CLI accepts `--hf-token` argument. [SRS-24/AC-01] <!-- verify: manual, SRS-24:start:end -->
- [ ] `HFHubAdapter` uses the provided token for requests. [SRS-25/AC-01] <!-- verify: manual, SRS-25:start:end -->
- [ ] Token is never printed to logs or stdout. [SRS-NFR-10/AC-01] <!-- verify: manual, SRS-NFR-10:start:end -->
