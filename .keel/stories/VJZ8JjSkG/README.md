---
# system-managed
id: VJZ8JjSkG
status: backlog
created_at: 2026-05-13T21:29:41
updated_at: 2026-05-13T21:36:09
# authored
title: Resolve Action Selection Through HTTP Model Clients
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 1
---

# Resolve Action Selection Through HTTP Model Clients

## Summary

Move action-selection model construction to the HTTP model-client boundary.
This story removes Sift model-path preparation from the action-selection path
while preserving provider capability negotiation.

## Acceptance Criteria

- [ ] A failing test is added first proving action-selection runtime construction never receives local `ModelPaths`. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] Action-selection clients are built through HTTP provider configuration and capability negotiation. [SRS-01/AC-02] <!-- verify: automated, SRS-01:start:end -->
- [ ] Legacy Sift action-selection provider config fails with the approved `ollama:<model>` migration hint. [SRS-01/AC-03] <!-- verify: automated, SRS-01:start:end -->
