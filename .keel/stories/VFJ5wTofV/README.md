---
# system-managed
id: VFJ5wTofV
status: done
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T18:27:56
# authored
title: Shift Retrieval Selection And Evidence Prioritization Toward Model Judgement
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 3
started_at: 2026-03-29T18:24:47
completed_at: 2026-03-29T18:27:56
---

# Shift Retrieval Selection And Evidence Prioritization Toward Model Judgement

## Summary

Move retrieval-selection and evidence-prioritization choices that represent
reasoning into constrained model judgement so recursive turns stop depending on
static lexical defaults and hardcoded source-priority rankings where the model
should decide.

## Acceptance Criteria

- [x] Retrieval query/mode selection on recursive turns no longer depends on hardcoded reasoning heuristics where the model can decide the better move. [SRS-04/AC-01] <!-- verify: cargo test -q recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary, SRS-04:start:end, proof: ac-1.log-->
- [x] Evidence prioritization used for recursive reasoning/synthesis is revised so reasoning-heavy ranking is not encoded as static controller policy. [SRS-04/AC-02] <!-- verify: cargo test -q grounded_answer_fallback_preserves_evidence_order_without_source_priority, SRS-04:start:end, proof: ac-2.log-->
- [x] Controller-owned safety constraints for execution and resource bounds remain unchanged. [SRS-NFR-01/AC-03] <!-- verify: cargo test -q exhausting_the_tool_budget_returns_an_error && cargo test -q read_file_rejects_symlink_escape, SRS-NFR-01:start:end, proof: ac-3.log-->
