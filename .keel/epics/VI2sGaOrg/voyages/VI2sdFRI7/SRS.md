# Preserve Planner Rationale Through Trace Pipeline - SRS

## Summary

Epic: VI2sGaOrg
Goal: Stop overwriting decision.rationale at recursive_control.rs:147-152; route controller-derived signal summaries to a sibling field; verify the model's own rationale flows verbatim into structured trace, forensics, and manifold projections.

## Scope

### In Scope

- [SCOPE-01] Stop assigning to `decision.rationale` in `recursive_control.rs:147-152` and any other code path that mutates a model-produced rationale, reasoning, or final-answer text field.
- [SCOPE-02] Add a sibling field on the planner decision (e.g. `controller_signal_summary`) for the controller-derived rationale that `compile_recursive_paddles_rationale` currently produces.
- [SCOPE-03] Update `TurnEvent::PlannerActionSelected` and downstream forensics / manifold projections to carry both the model rationale and the controller signal summary as distinct fields.

### Out of Scope

- [SCOPE-04] Removing the controller's ability to *reject* a planner decision; only rewriting model output is in scope.
- [SCOPE-05] Changes to deliberation signal extraction or premise / evidence checks themselves.
- [SCOPE-06] Vocabulary renames around the touched code (handled by the Tier-1 vocabulary epic VI2sJZcV9).

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The planner model's rationale text must flow verbatim from `RecursivePlannerDecision` into the structured trace, forensics, and manifold projections; controller-derived signal summaries must be emitted on a separate sibling field. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-01 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Existing planner / synthesizer behavior, governance enforcement, and trace recorder schemas must remain backward compatible aside from the added sibling field. | SCOPE-01, SCOPE-03 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
