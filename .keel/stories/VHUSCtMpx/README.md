---
# system-managed
id: VHUSCtMpx
status: backlog
created_at: 2026-04-21T21:19:34
updated_at: 2026-04-21T21:24:11
# authored
title: Move Conversation Projections Into An Application Read Model
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS6H0Kd
index: 2
---

# Move Conversation Projections Into An Application Read Model

## Summary

Move transcript, forensics, manifold, and related conversation projections out
of `domain/model` into an application-owned read-model boundary while
preserving replay and update behavior.

## Acceptance Criteria

- [ ] Conversation transcript, forensics, manifold, and trace graph projections are owned by an application read-model boundary rather than `domain/model`. [SRS-03/AC-01] <!-- verify: review, SRS-03:start:end -->
- [ ] Replay and projection update paths continue to produce equivalent conversation-scoped outputs through the new ownership boundary. [SRS-NFR-01/AC-02] <!-- verify: test, SRS-NFR-01:start:end -->
