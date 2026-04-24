---
# system-managed
id: VHkhmMMtW
status: icebox
created_at: 2026-04-24T16:01:46
updated_at: 2026-04-24T16:06:07
# authored
title: Integrate Policy Gate With Local Hands
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgKreip
index: 2
---

# Integrate Policy Gate With Local Hands

## Summary

Integrate the execution policy evaluator beneath the existing permission gate for shell, edit, patch, and external capability actions.

## Acceptance Criteria

- [ ] Shell, edit, patch, and external capability call sites consult the execution policy evaluator before execution. [SRS-02/AC-01] <!-- verify: cargo test execution_policy_gate -- --nocapture, SRS-02:start:end -->
- [ ] Conservative defaults preserve local-first behavior and fail closed when policy is invalid. [SRS-NFR-01/AC-01] <!-- verify: cargo test execution_policy_defaults -- --nocapture, SRS-NFR-01:start:end -->
