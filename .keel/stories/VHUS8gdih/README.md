---
# system-managed
id: VHUS8gdih
status: backlog
created_at: 2026-04-21T21:19:17
updated_at: 2026-04-21T21:22:51
# authored
title: Make Projection Updates Reducer-Driven And Versioned
type: feat
operator-signal:
scope: VHURpL4nG/VHUS4nctz
index: 2
---

# Make Projection Updates Reducer-Driven And Versioned

## Summary

Define one canonical live projection update contract with deterministic ordering
or version semantics so stream consumers can reconcile transcript/render state
from the same source replay uses.

## Acceptance Criteria

- [ ] Live projection updates expose deterministic reducer or version semantics for canonical transcript/render reconciliation. [SRS-03/AC-01] <!-- verify: test, SRS-03:start:end -->
- [ ] Stream consumers can detect stale state and rebuild from authoritative projection state rather than UI-local render repair heuristics. [SRS-04/AC-02] <!-- verify: test, SRS-04:start:end -->
