---
# system-managed
id: VFV0xnwcf
status: in-progress
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T18:50:02
# authored
title: Replace Sift Autonomous Gatherer With Direct Retrieval Adapter
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 1
started_at: 2026-03-31T18:50:02
---

# Replace Sift Autonomous Gatherer With Direct Retrieval Adapter

## Summary

Replace the current `sift-autonomous` gatherer execution path with a direct sift-backed retrieval adapter so paddles keeps ownership of recursive planning and refinement decisions.

## Acceptance Criteria

- [ ] Planner-driven gatherer turns no longer call the nested `sift-autonomous` planner path and instead execute a direct sift retrieval boundary. [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end, proof: ac-1.log-->
- [ ] The new direct adapter accepts the current paddles query, retrieval mode, strategy, budget, and prior context inputs. [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end, proof: ac-2.log-->
- [ ] Returned evidence and summaries remain usable by the existing paddles planner loop after the adapter swap. [SRS-01/AC-03] <!-- verify: test, SRS-01:start:end, proof: ac-3.log-->
- [ ] The new boundary preserves local-first execution without introducing a new network dependency. [SRS-NFR-02/AC-04] <!-- verify: review, SRS-NFR-02:start:end, proof: ac-4.log-->
