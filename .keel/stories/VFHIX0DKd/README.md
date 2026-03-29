---
# system-managed
id: VFHIX0DKd
status: done
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T11:58:34
# authored
title: Define Thread Decision And Merge Contract
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 1
started_at: 2026-03-29T11:56:48
submitted_at: 2026-03-29T11:58:33
completed_at: 2026-03-29T11:58:34
---

# Define Thread Decision And Merge Contract

## Summary

Define the paddles-owned contract for model-driven steering-prompt threading so
the runtime can distinguish between continuing the current thread, opening a
child thread, and reconciling work back to the mainline without falling back to
product-specific heuristics.

## Acceptance Criteria

- [x] A bounded thread decision contract exists for steering prompts and can express continue current thread, open child thread, and merge/reconcile outcomes with rationale and stable ids. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Merge/reconcile intent is part of the same bounded contract rather than an ad hoc later-stage escape hatch. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] The contract remains generic across evidence domains and does not encode Keel-specific thread types. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
