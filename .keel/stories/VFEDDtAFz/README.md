---
# system-managed
id: VFEDDtAFz
status: backlog
created_at: 2026-03-28T21:41:55
updated_at: 2026-03-28T21:48:08
# authored
title: Document And Prove Model-Directed Routing
type: docs
operator-signal:
scope: VFECyWLL6/VFED2RjSu
index: 4
---

# Document And Prove Model-Directed Routing

## Summary

Update the foundational docs and proof artifacts so operators can see the new
model-directed routing contract, the recursive loop behavior, and the remaining
transitional gaps without reverse-engineering the runtime from code.

## Acceptance Criteria

- [ ] README and companion architecture docs describe model-directed top-level action selection and how it fits into the recursive harness. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Operator guidance documents explain that the model owns bounded action selection while the controller owns validation and budgets. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] Execution proofs show before/after routing behavior for at least one turn that previously depended on heuristic top-level classification. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
