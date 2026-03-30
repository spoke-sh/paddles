# VOYAGE REPORT: Model-Judged Interpretation And Retrieval

## Voyage Metadata
- **ID:** VFJ5t0Pbk
- **Epic:** VFJ5rdPZP
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Remove Legacy Direct Routing Heuristics From Remaining Turn Paths
- **ID:** VFJ5wSYep
- **Status:** done

#### Summary
Remove the remaining legacy direct-path reasoning helpers such as string-based
casual/tool/follow-up inference from the primary harness path so bounded
model-selected actions remain the source of reasoning even when the turn does
not immediately recurse through the planner loop.

#### Acceptance Criteria
- [x] Remaining legacy direct-path routing/tool-inference heuristics are removed or demoted so they no longer decide the primary turn path. [SRS-01/AC-01] <!-- verify: cargo test -q deterministic_action_turns_require_model_selected_tool_calls && cargo test -q respond_starts_a_fresh_conversation_each_turn, SRS-01:start:end, proof: ac-1.log-->
- [x] Any retained direct-path fallback stays clearly fail-closed and controller-owned rather than silently reasoning for the model. [SRS-NFR-01/AC-02] <!-- verify: cargo test -q invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid && cargo test -q invalid_planner_replies_fail_closed_after_redecision_is_still_invalid, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Transcript/tests prove the model-selected path still handles conversational versus workspace turns without the old string classifier. [SRS-01/AC-03] <!-- verify: cargo test -q process_prompt_assembles_interpretation_before_model_selected_initial_action, SRS-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFJ5wSYep/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFJ5wSYep/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFJ5wSYep/EVIDENCE/ac-3.log)

### Replace Lexical Interpretation Scoring With Model-Judged Guidance Selection
- **ID:** VFJ5wTEei
- **Status:** done

#### Summary
Replace lexical interpretation relevance scoring and ranked hint/procedure
selection with constrained model judgement so `AGENTS.md` roots and their
referenced guidance graph determine what memory matters for the current turn.

#### Acceptance Criteria
- [x] Interpretation-time guidance selection no longer depends on lexical term scoring for relevance on the primary path. [SRS-02/AC-01] <!-- verify: cargo test -q interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots, SRS-02:start:end, proof: ac-1.log-->
- [x] Tool hints and decision procedures are selected from the model-derived guidance graph rather than controller keyword ranking. [SRS-02/AC-02] <!-- verify: cargo test -q initial_action_prompts_include_interpretation_context && cargo test -q interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots, SRS-02:start:end, proof: ac-2.log-->
- [x] `AGENTS.md` remains the only hardcoded interpretation root. [SRS-NFR-02/AC-03] <!-- verify: cargo test -q agent_memory_roots_only_load_agents_documents, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFJ5wTEei/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFJ5wTEei/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFJ5wTEei/EVIDENCE/ac-3.log)

### Shift Retrieval Selection And Evidence Prioritization Toward Model Judgement
- **ID:** VFJ5wTofV
- **Status:** done

#### Summary
Move retrieval-selection and evidence-prioritization choices that represent
reasoning into constrained model judgement so recursive turns stop depending on
static lexical defaults and hardcoded source-priority rankings where the model
should decide.

#### Acceptance Criteria
- [x] Retrieval query/mode selection on recursive turns no longer depends on hardcoded reasoning heuristics where the model can decide the better move. [SRS-04/AC-01] <!-- verify: cargo test -q recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary, SRS-04:start:end, proof: ac-1.log-->
- [x] Evidence prioritization used for recursive reasoning/synthesis is revised so reasoning-heavy ranking is not encoded as static controller policy. [SRS-04/AC-02] <!-- verify: cargo test -q grounded_answer_fallback_preserves_evidence_order_without_source_priority, SRS-04:start:end, proof: ac-2.log-->
- [x] Controller-owned safety constraints for execution and resource bounds remain unchanged. [SRS-NFR-01/AC-03] <!-- verify: cargo test -q exhausting_the_tool_budget_returns_an_error && cargo test -q read_file_rejects_symlink_escape, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFJ5wTofV/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFJ5wTofV/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFJ5wTofV/EVIDENCE/ac-3.log)

### Document And Prove The Controller-Versus-Model Boundary
- **ID:** VFJ5wU9fh
- **Status:** done

#### Summary
Update the foundational docs and ship proof artifacts that make the resulting
controller-versus-model boundary explicit so the heuristic-removal work remains
auditable and does not regress into hidden controller reasoning later.

#### Acceptance Criteria
- [x] Foundational docs describe which decisions are model-judged and which remain controller-owned constraints. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Proof artifacts demonstrate at least one end-to-end turn where model-judged interpretation and fallback replace prior heuristics. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] The docs stay generic across repositories rather than hardcoding a project-specific replacement intent taxonomy. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFJ5wU9fh/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFJ5wU9fh/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFJ5wU9fh/EVIDENCE/ac-3.log)

### Replace Heuristic Planner Fallback With Constrained Model Re-Decision
- **ID:** VFJ5wUdgP
- **Status:** done

#### Summary
Replace heuristic initial/planner fallback selection with additional constrained
model re-decision passes so invalid action replies do not immediately trigger
controller reasoning substitutes.

#### Acceptance Criteria
- [x] Invalid initial-action replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-01] <!-- verify: cargo test -q invalid_initial_action_replies_use_constrained_redecision_before_succeeding, SRS-03:start:end, proof: ac-1.log-->
- [x] Invalid recursive planner replies prefer constrained model re-decision before controller fallback. [SRS-03/AC-02] <!-- verify: cargo test -q invalid_planner_replies_use_constrained_redecision_before_succeeding, SRS-03:start:end, proof: ac-2.log-->
- [x] Any residual controller fallback is minimal, explicit, and fail-closed rather than a ranked reasoning engine. [SRS-NFR-01/AC-03] <!-- verify: cargo test -q invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid && cargo test -q invalid_planner_replies_fail_closed_after_redecision_is_still_invalid, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFJ5wUdgP/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFJ5wUdgP/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFJ5wUdgP/EVIDENCE/ac-3.log)


