---
# system-managed
id: VFEDDtAFz
status: done
created_at: 2026-03-28T21:41:55
updated_at: 2026-03-29T08:38:20
# authored
title: Document And Prove Model-Directed Routing
type: docs
operator-signal:
scope: VFECyWLL6/VFED2RjSu
index: 4
started_at: 2026-03-29T08:36:20
submitted_at: 2026-03-29T08:38:15
completed_at: 2026-03-29T08:38:20
---

# Document And Prove Model-Directed Routing

## Summary

Update the foundational docs and proof artifacts so operators can see the new
model-directed routing contract, the recursive loop behavior, and the remaining
transitional gaps without reverse-engineering the runtime from code.

## Acceptance Criteria

- [x] README and companion architecture docs describe model-directed top-level action selection and how it fits into the recursive harness. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-1.log-->
- [x] Operator guidance documents explain that the model owns bounded action selection while the controller owns validation and budgets. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] Execution proofs show before/after routing behavior for at least one turn that previously depended on heuristic top-level classification. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
