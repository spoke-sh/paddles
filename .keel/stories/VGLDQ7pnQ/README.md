---
# system-managed
id: VGLDQ7pnQ
status: in-progress
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:17
# authored
title: Define Session Wake Slice And Checkpoint Contract
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuE5W
index: 1
started_at: 2026-04-09T16:58:17
---

# Define Session Wake Slice And Checkpoint Contract

## Summary

Define the durable session contract that later runtime code can depend on. This story should make wake, replay, checkpoint, and selective slice semantics explicit so the session becomes a stable object outside any particular model context window.

## Acceptance Criteria

- [ ] The session contract names how a harness wakes a prior session, replays it, and resumes from checkpoints without relying on ad hoc prompt summaries [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Selective event-slice interrogation is explicit enough that later context and recovery stories can consume it without redefining replay semantics [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
