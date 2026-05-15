# VOYAGE REPORT: Retire Pre-Loop Bootstraps And Vocabulary

## Voyage Metadata
- **ID:** VJeRAR1IS
- **Epic:** VJeQx1O20
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Remove Pre-Loop Bootstrap Overrides
- **ID:** VJeRZ5UCs
- **Status:** done

#### Summary
Remove the controller bootstrap overrides that currently replace the model's first action before the agent loop runs, converting any still-needed pressure into loop-visible signals.

#### Acceptance Criteria
- [x] Commit, known-edit, repository-grounding, and review bootstrap helpers no longer force initial actions before the loop. [SRS-01/AC-01] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "bootstrap_.*initial_action|known-edit-bootstrap|commit-bootstrap|grounding-bootstrap|review-bootstrap" src/application; then exit 1; else test $? -eq 1; fi', SRS-01:start:end, proof: ac-1.log-->
- [x] Remaining edit/commit/grounding/review pressure is visible in loop request state or execution contract lines. [SRS-01/AC-02] <!-- verify: cargo test bootstrap_pressure_is_loop_visible_context -- --nocapture, SRS-01:start:end, proof: ac-2.log-->
- [x] Removing bootstraps does not weaken mutation or commit completion enforcement. [SRS-NFR-01/AC-03] <!-- verify: cargo test edit_commit_boundaries_survive_bootstrap_removal -- --nocapture, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRZ5UCs/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRZ5UCs/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRZ5UCs/EVIDENCE/ac-3.log)

### Update Unified Loop Traces And Docs
- **ID:** VJeRZgtQM
- **Status:** done

#### Summary
Align traces, runtime presentation, and architecture documents with the unified agent-loop architecture after the code migration is complete.

#### Acceptance Criteria
- [x] Runtime labels no longer imply a separate planner or initial-action lane for normal turns. [SRS-02/AC-01] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "initial action|pre-loop|planner route|Planner step 0" src/infrastructure src/application; then exit 1; else test $? -eq 1; fi', SRS-02:start:end, proof: ac-1.log-->
- [x] `ARCHITECTURE.md` and `CONFIGURATION.md` describe the agent loop as the single action-selection owner. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Full formatting and library checks pass after docs/runtime presentation cleanup. [SRS-NFR-02/AC-03] <!-- verify: cargo fmt --all --check && cargo test --quiet --lib, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRZgtQM/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRZgtQM/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRZgtQM/EVIDENCE/ac-3.log)

### Add Simple Evidence Question Regression
- **ID:** VJeRaXliR
- **Status:** done

#### Summary
Add a regression for the failure mode that prompted the mission: a simple operator-contract question should use live evidence once and answer from the loop state, not spin in repeated pre-loop reads or rely on a hard repeat guard.

#### Acceptance Criteria
- [x] A focused test reproduces a simple evidence-backed operator-contract question entering the loop and answering from gathered evidence. [SRS-04/AC-01] <!-- verify: cargo test simple_evidence_question_answers_from_loop_state -- --nocapture, SRS-04:start:end, proof: ac-1.log-->
- [x] The regression proves observations are fed back into action-selection reasoning, not blocked by a hard duplicate-read guard. [SRS-04/AC-02] <!-- verify: cargo test repeated_read_failure_uses_observation_feedback -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Full library tests pass after the regression is added. [SRS-NFR-02/AC-03] <!-- verify: cargo test --quiet --lib, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRaXliR/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRaXliR/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRaXliR/EVIDENCE/ac-3.log)


