---
# system-managed
id: VGdaSObFI
status: backlog
created_at: 2026-04-12T20:19:54
updated_at: 2026-04-12T20:23:48
# authored
title: Project Delegated Worker State Across Operator Surfaces
type: feat
operator-signal:
scope: VGb1c2DBj/VGdaQAncW
index: 3
---

# Project Delegated Worker State Across Operator Surfaces

## Summary

Project delegated worker state across transcript and operator surfaces so active
workers, roles, ownership, progress, and completion or integration state stay
legible while the parent turn coordinates parallel work.

## Acceptance Criteria

- [ ] Transcript, TUI, web, and API surfaces render one shared delegation vocabulary for active workers, roles, ownership, progress, and completion or integration state. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Operator-facing surfaces show delegated progress clearly enough to follow parent and worker responsibilities without inspecting raw trace internals. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] Degraded, conflicting, or unsupported delegation states render honestly across shared surfaces instead of disappearing behind optimistic UI summaries. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Shared delegation surfaces preserve one recursive-harness identity and do not present workers as an unrelated orchestration subsystem. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end -->
