# Trust Planner Rationale Verbatim - Product Requirements

## Problem Statement

Controller currently overwrites decision.rationale at src/application/recursive_control.rs:147-152, breaking the 'let the model reason first' contract and lying in the forensic trace. Move controller-derived signal summaries and governance notes onto sibling fields and ensure the model's own rationale flows verbatim into traces and the manifold view.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Resolve the problem described above for the primary user. | A measurable outcome is defined for this problem | Target agreed during planning |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | The person or team most affected by the problem above. | A clearer path to the outcome this epic should improve. |

## Scope

### In Scope

- [SCOPE-01] Stop assigning to `decision.rationale` in `recursive_control.rs:147-152` and any other code path that mutates a model-produced rationale, reasoning, or final-answer text field.
- [SCOPE-02] Add a sibling field on the planner decision (e.g. `controller_signal_summary`) for the controller-derived rationale that `compile_recursive_paddles_rationale` currently produces.
- [SCOPE-03] Update `TurnEvent::PlannerActionSelected` and downstream forensics / manifold projections to carry both the model rationale and the controller signal summary as distinct fields.

### Out of Scope

- [SCOPE-04] Removing the controller's ability to *reject* a planner decision; only rewriting model output is in scope.
- [SCOPE-05] Changes to deliberation signal extraction or premise / evidence checks themselves.
- [SCOPE-06] Vocabulary renames around the touched code (handled by the Tier-1 vocabulary epic VI2sJZcV9).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Deliver the primary user workflow for this epic end-to-end. | GOAL-01 | must | Establishes the minimum functional capability needed to achieve the epic goal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01 | must | Keeps operations stable and makes regressions detectable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
