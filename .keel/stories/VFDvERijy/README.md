---
# system-managed
id: VFDvERijy
status: done
created_at: 2026-03-28T20:30:28
updated_at: 2026-03-28T21:20:44
# authored
title: Separate Planner And Synthesizer Model Contracts
type: feat
operator-signal:
scope: VFDv1i61H/VFDv3gE5m
index: 4
started_at: 2026-03-28T21:14:24
submitted_at: 2026-03-28T21:20:40
completed_at: 2026-03-28T21:20:44
---

# Separate Planner And Synthesizer Model Contracts

## Summary

Separate planner and synthesizer model contracts so recursive evidence
construction and final answer generation can be routed independently.

## Acceptance Criteria

- [x] The planner handoff to synthesis is a typed evidence/trace contract rather than free-form planner prose. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] Routing can choose planner and synthesizer providers independently according to runtime constraints. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] Fallback behavior remains local-first when a heavier planner model is unavailable. [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
- [x] Planner traces, action decisions, stop reasons, and synthesizer handoff data remain observable to operators. [SRS-NFR-03/AC-04] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-4.log-->
