---
# system-managed
id: VHkhz3Y8v
status: done
created_at: 2026-04-24T16:02:35
updated_at: 2026-04-24T19:20:27
# authored
title: Document Harness Capability Configuration
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 3
started_at: 2026-04-24T19:11:48
submitted_at: 2026-04-24T19:20:23
completed_at: 2026-04-24T19:20:27
---

# Document Harness Capability Configuration

## Summary

Document capability configuration, eval usage, and local-first boundaries for operators adopting the upgraded harness.

## Acceptance Criteria

- [x] Operator docs explain configuring external capabilities, execution policy, evals, and provider posture. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Runtime entrypoint smoke checks confirm the documented surfaces expose the new harness posture consistently. [SRS-04/AC-01] <!-- verify: cargo test runtime_entrypoint_smoke -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
