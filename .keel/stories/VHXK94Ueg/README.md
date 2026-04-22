---
# system-managed
id: VHXK94Ueg
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Define Normalized Deliberation Signals
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJiqMD1
index: 3
---

# Define Normalized Deliberation Signals

## Summary

Define the application-owned `DeliberationSignals` contract and the extraction
rules that turn provider-native reasoning artifacts into bounded,
provider-agnostic hints the recursive harness can understand.

## Acceptance Criteria

- [ ] The application layer defines a provider-agnostic deliberation signal contract for continuation, uncertainty, evidence gaps, branching, stop confidence, and risk hints. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Providers can emit zero or more normalized signals, with explicit safe semantics for `none` or `unknown`. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
