---
# system-managed
id: VHkhk8Xmt
status: backlog
created_at: 2026-04-24T16:01:38
updated_at: 2026-04-24T16:04:59
# authored
title: Govern External Capability Invocation Results
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgG2aro
index: 2
---

# Govern External Capability Invocation Results

## Summary

Route external capability invocations through governance and return typed results for allowed, denied, unavailable, degraded, and malformed outcomes.

## Acceptance Criteria

- [ ] External capability calls consult governance before executing side effects. [SRS-02/AC-01] <!-- verify: cargo test external_capability_governance -- --nocapture, SRS-02:start:end -->
- [ ] Denied and degraded external calls return typed evidence rather than opaque errors. [SRS-02/AC-02] <!-- verify: cargo test external_capability_result_states -- --nocapture, SRS-02:start:end -->
