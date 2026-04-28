# VOYAGE REPORT: Preserve Planner Rationale Through Trace Pipeline

## Voyage Metadata
- **ID:** VI2sdFRI7
- **Epic:** VI2sGaOrg
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Route Controller Signal Summary To Sibling Field
- **ID:** VI2snQ4bl
- **Status:** done

#### Summary
Land the rationale-trust change end-to-end: stop assigning to `decision.rationale` in `src/application/recursive_control.rs`, add a sibling field for the controller-derived signal summary, and update `TurnEvent::PlannerActionSelected` plus forensics / manifold projections so the model's own rationale text flows through unchanged while controller annotations remain visible alongside it.

#### Acceptance Criteria
- [x] `recursive_control.rs` no longer assigns to `decision.rationale`; the planner model's rationale text is preserved verbatim from `RecursivePlannerDecision` through to the trace. [SRS-01/AC-01] <!-- verify: cargo test --lib planner_rationale_flows_verbatim_with_controller_summary_on_sibling_field, SRS-01:start:end -->
- [x] A sibling field (`controller_summary`) carries the controller-derived narrative and is emitted on `TurnEvent::PlannerActionSelected` alongside the model's rationale. [SRS-01/AC-02] <!-- verify: cargo test --lib planner_rationale_flows_verbatim_with_controller_summary_on_sibling_field, SRS-01:start:end -->
- [x] `TraceRecordKind::PlannerAction` carries `controller_summary` as a separate field so forensics and manifold projections render the model's rationale and the controller annotation distinctly. [SRS-01/AC-03] <!-- verify: cargo test --lib planner_rationale_flows_verbatim_with_controller_summary_on_sibling_field, SRS-01:start:end -->


