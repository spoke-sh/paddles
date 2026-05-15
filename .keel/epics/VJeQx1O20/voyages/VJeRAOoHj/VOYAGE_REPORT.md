# VOYAGE REPORT: Unify First Action Entry Point

## Voyage Metadata
- **ID:** VJeRAOoHj
- **Epic:** VJeQx1O20
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Route First Action Through Agent Loop
- **ID:** VJeRUd3hl
- **Status:** done

#### Summary
Route normal turn execution into the recursive agent loop before any model-selected action is accepted. The first loop iteration should own direct answers, stops, and workspace actions instead of receiving a preselected initial decision from `turn.rs`.

#### Acceptance Criteria
- [x] Turn orchestration enters `execute_agent_loop` before model-selected action execution for normal turns. [SRS-01/AC-01] <!-- verify: cargo test route_first_action_through_agent_loop -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The first loop iteration can return direct-answer, stop, and workspace-action outcomes. [SRS-02/AC-02] <!-- verify: cargo test first_agent_loop_iteration_handles_initial_outcomes -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] Direct-answer and final-rendering behavior remains available through `AgentLoopOutcome`. [SRS-03/AC-03] <!-- verify: cargo test agent_loop_outcome_preserves_direct_answer_rendering -- --nocapture, SRS-03:start:end, proof: ac-3.log-->
- [x] First-action trace evidence is emitted by the loop rather than by a pre-loop router. [SRS-NFR-01/AC-03] <!-- verify: cargo test first_action_trace_is_loop_owned -- --nocapture, SRS-NFR-01:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRUd3hl/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRUd3hl/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRUd3hl/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VJeRUd3hl/EVIDENCE/ac-4.log)

### Collapse Initial Action Interface
- **ID:** VJeRVkb73
- **Status:** done

#### Summary
Collapse the separate initial-action API and runtime planning structs that let normal turns make a model-owned decision before the agent loop starts.

#### Acceptance Criteria
- [x] Normal runtime code no longer calls `select_initial_action`. [SRS-04/AC-01] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "select_initial_action" src/application src/domain src/infrastructure; then exit 1; else test $? -eq 1; fi', SRS-04:start:end, proof: ac-1.log-->
- [x] `PromptExecutionPlan` and `PromptExecutionPath` are removed from the normal runtime path. [SRS-04/AC-02] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "PromptExecutionPlan|PromptExecutionPath" src/application src/domain src/infrastructure; then exit 1; else test $? -eq 1; fi', SRS-04:start:end, proof: ac-2.log-->
- [x] Provider compatibility, if still needed, is isolated away from turn orchestration. [SRS-04/AC-03] <!-- verify: cargo test action_selection_initial_compatibility_is_not_runtime_routing -- --nocapture, SRS-04:start:end, proof: ac-3.log-->
- [x] Full library tests pass after the entry-point interface collapse. [SRS-NFR-02/AC-04] <!-- verify: cargo test --lib, SRS-NFR-02:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRVkb73/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRVkb73/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRVkb73/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VJeRVkb73/EVIDENCE/ac-4.log)


