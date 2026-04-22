---
# system-managed
id: VHUS8h4jI
status: backlog
created_at: 2026-04-21T21:19:17
updated_at: 2026-04-21T21:22:51
# authored
title: Prove Live And Replay Render Convergence
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 3
---

# Prove Live And Replay Render Convergence

## Summary

Add contract tests that compare live emitted render/projection state with
replayed transcript state for the same completed turn so stream rendering drift
fails fast.

## Acceptance Criteria

- [ ] Automated tests compare live turn render/projection output with replayed transcript state for the same completed turn. [SRS-05/AC-01] <!-- verify: test, SRS-05:start:end -->
- [ ] The convergence suite covers render blocks, response mode, and citation/grounding metadata. [SRS-NFR-01/AC-02] <!-- verify: test, SRS-NFR-01:start:end -->
