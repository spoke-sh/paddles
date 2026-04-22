---
# system-managed
id: VHUS8h4jI
status: done
created_at: 2026-04-21T21:19:17
updated_at: 2026-04-21T21:51:12
# authored
title: Prove Live And Replay Render Convergence
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 3
started_at: 2026-04-21T21:49:05
completed_at: 2026-04-21T21:51:12
---

# Prove Live And Replay Render Convergence

## Summary

Add contract tests that compare live emitted render/projection state with
replayed transcript state for the same completed turn so stream rendering drift
fails fast.

## Acceptance Criteria

- [x] Automated tests compare live turn render/projection output with replayed transcript state for the same completed turn. [SRS-05/AC-01] <!-- verify: cargo test live_projection_updates_converge_with_replayed_transcript_render_state -- --nocapture, SRS-05:start:end, proof: ac-1.log-->
- [x] The convergence suite covers render blocks, response mode, and citation/grounding metadata. [SRS-NFR-01/AC-02] <!-- verify: cargo test live_projection_updates_converge_with_replayed_transcript_render_state -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->
