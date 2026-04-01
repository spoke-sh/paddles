---
# system-managed
id: VFV0xnwcf
status: done
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T19:06:17
# authored
title: Replace Sift Autonomous Gatherer With Direct Retrieval Adapter
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 1
started_at: 2026-03-31T18:50:02
completed_at: 2026-03-31T19:06:17
---

# Replace Sift Autonomous Gatherer With Direct Retrieval Adapter

## Summary

Replace the current `sift-autonomous` gatherer execution path with a direct sift-backed retrieval adapter so paddles keeps ownership of recursive planning and refinement decisions.

## Acceptance Criteria

- [x] Planner-driven gatherer turns no longer call the nested `sift-autonomous` planner path and instead execute a direct sift retrieval boundary. [SRS-01/AC-01] <!-- verify: cargo test -q direct_gatherer_returns_direct_retrieval_metadata_and_evidence, SRS-01:start:end, proof: ac-1.log-->
- [x] The new direct adapter accepts the current paddles query, retrieval mode, strategy, budget, and prior context inputs. [SRS-02/AC-02] <!-- verify: cargo test -q direct_gatherer_respects_budget_and_requested_mode_metadata, SRS-02:start:end, proof: ac-2.log-->
- [x] Returned evidence and summaries remain usable by the existing paddles planner loop after the adapter swap. [SRS-01/AC-03] <!-- verify: cargo test -q recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary, SRS-01:start:end, proof: ac-3.log-->
- [x] The new boundary preserves local-first execution without introducing a new network dependency. [SRS-NFR-02/AC-04] <!-- verify: cargo test -q direct_gatherer_returns_direct_retrieval_metadata_and_evidence, SRS-NFR-02:start:end, proof: ac-4.log-->
