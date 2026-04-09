---
# system-managed
id: VGLDQBYqH
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:07
# authored
title: Expose Session-Queryable Context Slices For Adaptive Replay
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMvU4i
index: 2
---

# Expose Session-Queryable Context Slices For Adaptive Replay

## Summary

Expose queryable session slices that adaptive replay and compaction code can use without destructive prompt-only summarization. This story should turn the durable session into a real context object for the harness.

## Acceptance Criteria

- [ ] Session-queryable context slices support adaptive replay, rewind, or compaction-oriented access outside the live prompt window [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Slice semantics are explicit enough that later adaptive-profile work can reuse them without redefining replay behavior [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
