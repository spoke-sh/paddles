---
# system-managed
id: VHkhz3Y8v
status: icebox
created_at: 2026-04-24T16:02:35
updated_at: 2026-04-24T16:06:50
# authored
title: Document Harness Capability Configuration
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 3
---

# Document Harness Capability Configuration

## Summary

Document capability configuration, eval usage, and local-first boundaries for operators adopting the upgraded harness.

## Acceptance Criteria

- [ ] Operator docs explain configuring external capabilities, execution policy, evals, and provider posture. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Runtime entrypoint smoke checks confirm the documented surfaces expose the new harness posture consistently. [SRS-04/AC-01] <!-- verify: cargo test runtime_entrypoint_smoke -- --nocapture, SRS-04:start:end -->
