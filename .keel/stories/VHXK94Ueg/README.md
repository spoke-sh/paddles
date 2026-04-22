---
# system-managed
id: VHXK94Ueg
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T10:41:14
# authored
title: Define Normalized Deliberation Signals
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJiqMD1
index: 3
started_at: 2026-04-22T10:36:41
completed_at: 2026-04-22T10:41:14
---

# Define Normalized Deliberation Signals

## Summary

Define the application-owned `DeliberationSignals` contract and the extraction
rules that turn provider-native reasoning artifacts into bounded,
provider-agnostic hints the recursive harness can understand.

## Acceptance Criteria

- [x] The application layer defines a provider-agnostic deliberation signal contract for continuation, uncertainty, evidence gaps, branching, stop confidence, and risk hints. [SRS-01/AC-01] <!-- verify: cargo test application::deliberation::tests:: -- --nocapture, SRS-01:start:end -->
- [x] Providers can emit zero or more normalized signals, with explicit safe semantics for `none` or `unknown`. [SRS-02/AC-02] <!-- verify: cargo test application::deliberation::tests:: -- --nocapture && cargo test provider_turn_request_and_response_keep_deliberation_state_separate_from_content -- --nocapture && cargo test capability_surface_ -- --nocapture, SRS-02:start:end -->
