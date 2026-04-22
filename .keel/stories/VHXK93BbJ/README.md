---
# system-managed
id: VHXK93BbJ
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Compile Explicit Paddles Rationale And Operator Evidence
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJiqMD1
index: 1
---

# Compile Explicit Paddles Rationale And Operator Evidence

## Summary

Compile the final paddles `rationale` from chosen actions, evidence, and
normalized signals, and present operator-facing signal summaries without
defaulting to raw provider-native reasoning content.

## Acceptance Criteria

- [ ] Final planner or synthesis decisions persist a concise paddles rationale derived from action, evidence, and normalized signals rather than raw provider reasoning. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] Transcript, manifold, and forensic/operator surfaces show rationale and signal summaries without raw provider-native reasoning by default. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] Decision-path tests cover at least one native-continuation provider and one explicit no-op provider. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
