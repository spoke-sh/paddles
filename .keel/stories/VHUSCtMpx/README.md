---
# system-managed
id: VHUSCtMpx
status: done
created_at: 2026-04-21T21:19:34
updated_at: 2026-04-21T22:58:24
# authored
title: Move Conversation Projections Into An Application Read Model
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS6H0Kd
index: 2
started_at: 2026-04-21T22:48:52
completed_at: 2026-04-21T22:58:24
---

# Move Conversation Projections Into An Application Read Model

## Summary

Move transcript, forensics, manifold, and related conversation projections out
of `domain/model` into an application-owned read-model boundary while
preserving replay and update behavior.

## Acceptance Criteria

- [x] Conversation transcript, forensics, manifold, and trace graph projections are owned by an application read-model boundary rather than `domain/model`. [SRS-03/AC-01] <!-- verify: /home/alex/workspace/spoke-sh/paddles/.keel/stories/VHUSCtMpx/EVIDENCE/verify-review.sh, SRS-03:start:end, proof: review.md -->
- [x] Replay and projection update paths continue to produce equivalent conversation-scoped outputs through the new ownership boundary. [SRS-NFR-01/AC-02] <!-- verify: /home/alex/workspace/spoke-sh/paddles/.keel/stories/VHUSCtMpx/EVIDENCE/verify-ac-2.sh, SRS-NFR-01:start:end, proof: read-model-tests.log, proof: application-tests.log -->
