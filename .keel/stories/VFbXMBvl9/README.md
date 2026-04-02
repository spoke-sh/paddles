---
# system-managed
id: VFbXMBvl9
status: done
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T22:00:55
# authored
title: Record Context Lineage And Force Snapshots In Transit
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 2
started_at: 2026-04-01T21:50:03
completed_at: 2026-04-01T22:00:55
---

# Record Context Lineage And Force Snapshots In Transit

## Summary

Capture the lineage and force metadata that explains how context was assembled and constrained. Transit should record lineage edges plus force snapshots and source-contribution estimates so the web inspector can explain not only what happened, but why.

## Acceptance Criteria

- [x] Transit records lineage edges between conversation, turn, model call, planner step, artifacts, and resulting outputs [SRS-02/AC-01] <!-- verify: cargo test -q structured_turn_trace_records_lineage_edges_for_model_calls_and_outputs, SRS-02:start:end, proof: ac-1.log-->
- [x] Transit records force snapshots for pressure, truncation/compaction, execution/edit pressure, fallback/coercion, and budget effects at the relevant steps [SRS-03/AC-02] <!-- verify: cargo test -q structured_turn_trace_records_force_snapshots_with_contribution_estimates, SRS-03:start:end, proof: ac-2.log-->
- [x] Transit records contribution estimates by source alongside the applied forces using a documented heuristic/controller-derived model [SRS-03/AC-03] <!-- verify: rg -n 'pressure_force_contributions|compaction_force_contributions|fallback_force_details|budget_force_details' /home/alex/workspace/spoke-sh/paddles/src/application/mod.rs, SRS-03:start:end, proof: ac-3.log-->
- [x] Forensic replay can order lineage and force records coherently for a selected turn [SRS-NFR-01/AC-04] <!-- verify: cargo test -q structured_turn_trace, SRS-NFR-01:start:end, proof: ac-4.log-->
