---
# system-managed
id: VHUS8gdih
status: done
created_at: 2026-04-21T21:19:17
updated_at: 2026-04-21T21:47:18
# authored
title: Make Projection Updates Reducer-Driven And Versioned
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 2
started_at: 2026-04-21T21:41:33
completed_at: 2026-04-21T21:47:18
---

# Make Projection Updates Reducer-Driven And Versioned

## Summary

Define one canonical live projection update contract with deterministic ordering
or version semantics so stream consumers can reconcile transcript/render state
from the same source replay uses.

## Acceptance Criteria

- [x] Live projection updates expose deterministic reducer or version semantics for canonical transcript/render reconciliation. [SRS-03/AC-01] <!-- verify: cargo test conversation_projection_updates_are_derived_from_authoritative_replay_state -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Stream consumers can detect stale state and rebuild from authoritative projection state rather than UI-local render repair heuristics. [SRS-04/AC-02] <!-- verify: npm --workspace @paddles/web run test -- projection-state.test.ts runtime-shell.test.tsx, SRS-04:start:end, proof: ac-2.log-->
