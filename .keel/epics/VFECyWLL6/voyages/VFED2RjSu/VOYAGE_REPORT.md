# VOYAGE REPORT: Model-Directed Routing Backbone

## Voyage Metadata
- **ID:** VFED2RjSu
- **Epic:** VFECyWLL6
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Define Model-Directed Next-Action Contract
- **ID:** VFEDDrcF7
- **Status:** done

#### Summary
Define the constrained top-level action schema that replaces heuristic routing
for non-trivial turns. This story owns the contract shape, validation rules,
and the planner/synth boundary needed for model-directed first action
selection.

#### Acceptance Criteria
- [x] A top-level action contract exists for first action selection and can express direct answer or synthesize, search, read, inspect, refine, branch, and stop. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] The contract defines the validation envelope the controller must enforce, including safe inspect/tool boundaries and fail-closed behavior, without yet owning the runtime refactor. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] The contract is positioned as a general-purpose harness boundary rather than a Keel-specific routing feature. [SRS-NFR-03/AC-03] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFEDDrcF7/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFEDDrcF7/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFEDDrcF7/EVIDENCE/ac-3.log)
- [llm-judge-a-top-level-action-contract-exists-for-first-action-selection-and-can-express-direct-answer-or-synthesize-search-read-inspect-refine-branch-and-stop.txt](../../../../stories/VFEDDrcF7/EVIDENCE/llm-judge-a-top-level-action-contract-exists-for-first-action-selection-and-can-express-direct-answer-or-synthesize-search-read-inspect-refine-branch-and-stop.txt)

### Feed Interpretation Context Into First Action Selection
- **ID:** VFEDDsIF6
- **Status:** done

#### Summary
Move `AGENTS.md`, linked foundational docs, recent turns, and relevant local
state into the first action-selection prompt so the model chooses its initial
bounded action from interpretation context rather than after a controller
shortcut.

#### Acceptance Criteria
- [x] Non-trivial turns assemble interpretation context before first bounded action selection. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The first action-selection prompt or contract demonstrably includes operator memory and linked foundational guidance. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] Interpretation-context assembly remains observable in the default user surface. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFEDDsIF6/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFEDDsIF6/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFEDDsIF6/EVIDENCE/ac-3.log)

### Replace Heuristic Top-Level Routing With Planner Decisions
- **ID:** VFEDDsfF9
- **Status:** done

#### Summary
Retire the current heuristic top-level routing gate for non-trivial turns and
drive the initial route from the validated model-selected action path instead.
This story owns the runtime refactor across controller, planner loop, and safe
execution boundaries.

#### Acceptance Criteria
- [x] Non-trivial turns no longer depend on a separate heuristic classifier to choose their first resource action. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Safe inspect/tool execution remains controller-validated and bounded after the refactor. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [x] Recursive planner execution and synthesizer handoff operate through the new top-level action contract without regressing grounded answers. [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end, proof: ac-3.log-->
- [x] Model-directed routing remains local-first and fails closed when planner output is invalid or a heavier planner provider is unavailable. [SRS-NFR-01/AC-04] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFEDDsfF9/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFEDDsfF9/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFEDDsfF9/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VFEDDsfF9/EVIDENCE/ac-4.log)

### Document And Prove Model-Directed Routing
- **ID:** VFEDDtAFz
- **Status:** done

#### Summary
Update the foundational docs and proof artifacts so operators can see the new
model-directed routing contract, the recursive loop behavior, and the remaining
transitional gaps without reverse-engineering the runtime from code.

#### Acceptance Criteria
- [x] README and companion architecture docs describe model-directed top-level action selection and how it fits into the recursive harness. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-1.log-->
- [x] Operator guidance documents explain that the model owns bounded action selection while the controller owns validation and budgets. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] Execution proofs show before/after routing behavior for at least one turn that previously depended on heuristic top-level classification. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFEDDtAFz/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFEDDtAFz/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFEDDtAFz/EVIDENCE/ac-3.log)


