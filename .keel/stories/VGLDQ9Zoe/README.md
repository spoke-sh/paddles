---
# system-managed
id: VGLDQ9Zoe
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:07
# authored
title: Define Shared Hand Lifecycle And Diagnostics Surface
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuu5X
index: 1
---

# Define Shared Hand Lifecycle And Diagnostics Surface

## Summary

Define the common execution-hand contract that local action surfaces should share. This story should name the lifecycle, provisioning, execution, recovery, and diagnostics vocabulary before adapters are migrated onto it.

## Acceptance Criteria

- [ ] The runtime defines a shared hand lifecycle and diagnostics surface that covers local execution boundaries consistently [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The hand vocabulary is explicit enough that later workspace, terminal, and transport stories can adopt it without inventing new state names [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
