---
# system-managed
id: VGLDQAMpx
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:07
# authored
title: Adapt Workspace And Terminal Execution To Hand Contracts
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuu5X
index: 2
---

# Adapt Workspace And Terminal Execution To Hand Contracts

## Summary

Adapt the local workspace editor and terminal runner to the shared hand contract. This story should preserve current authored-workspace and shell semantics while shifting them onto the common execution interface.

## Acceptance Criteria

- [ ] Workspace editing and terminal execution run through the shared hand contract without breaking current local-first operator behavior [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Hand execution remains observable through the existing runtime trace and diagnostics surfaces after the adapter migration [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
