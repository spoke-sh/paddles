---
# system-managed
id: VFJ5wTEei
status: done
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T18:27:53
# authored
title: Replace Lexical Interpretation Scoring With Model-Judged Guidance Selection
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 2
started_at: 2026-03-29T18:24:40
completed_at: 2026-03-29T18:27:53
---

# Replace Lexical Interpretation Scoring With Model-Judged Guidance Selection

## Summary

Replace lexical interpretation relevance scoring and ranked hint/procedure
selection with constrained model judgement so `AGENTS.md` roots and their
referenced guidance graph determine what memory matters for the current turn.

## Acceptance Criteria

- [x] Interpretation-time guidance selection no longer depends on lexical term scoring for relevance on the primary path. [SRS-02/AC-01] <!-- verify: cargo test -q interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots, SRS-02:start:end, proof: ac-1.log-->
- [x] Tool hints and decision procedures are selected from the model-derived guidance graph rather than controller keyword ranking. [SRS-02/AC-02] <!-- verify: cargo test -q initial_action_prompts_include_interpretation_context && cargo test -q interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots, SRS-02:start:end, proof: ac-2.log-->
- [x] `AGENTS.md` remains the only hardcoded interpretation root. [SRS-NFR-02/AC-03] <!-- verify: cargo test -q agent_memory_roots_only_load_agents_documents, SRS-NFR-02:start:end, proof: ac-3.log-->
