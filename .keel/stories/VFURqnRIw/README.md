---
# system-managed
id: VFURqnRIw
status: done
created_at: 2026-03-31T16:20:23
updated_at: 2026-03-31T16:34:39
# authored
title: Emit RefinementApplied TurnEvent In Trace Stream
type: feat
operator-signal:
scope: VFUNJz9zT/VFURjeU0t
index: 3
started_at: 2026-03-31T16:31:16
submitted_at: 2026-03-31T16:34:31
completed_at: 2026-03-31T16:34:39
---

# Emit RefinementApplied TurnEvent In Trace Stream

## Summary

Emit a trace-level `RefinementApplied` event when a refinement is accepted so execution, diagnostics, and replay can consume the context mutation.

## Acceptance Criteria

- [x] Emit `RefinementApplied` as a structured turn event and stream it through trace output with the refinement reason and updated context summary. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
