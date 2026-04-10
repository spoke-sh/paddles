---
# system-managed
id: VGLDQBYqH
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T18:39:17
# authored
title: Expose Session-Queryable Context Slices For Adaptive Replay
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMvU4i
index: 2
started_at: 2026-04-09T18:30:59
completed_at: 2026-04-09T18:39:17
---

# Expose Session-Queryable Context Slices For Adaptive Replay

## Summary

Expose queryable session slices that adaptive replay and compaction code can use without destructive prompt-only summarization. This story should turn the durable session into a real context object for the harness.

## Acceptance Criteria

- [x] Session-queryable context slices support adaptive replay, rewind, or compaction-oriented access outside the live prompt window [SRS-02/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test trace_recording::tests:: -- --nocapture && cargo test recent_turn_summaries_prefer_session_context_slice_before_history_or_synth_fallback -- --nocapture', SRS-02:start:end, proof: ac-1.log -->
- [x] Slice semantics are explicit enough that later adaptive-profile work can reuse them without redefining replay behavior [SRS-02/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "query_session_context|AdaptiveReplay|CompactionWindow|Rewind" README.md ARCHITECTURE.md CONFIGURATION.md src/domain/ports/trace_recording.rs src/application/mod.rs', SRS-02:start:end, proof: ac-2.log -->
