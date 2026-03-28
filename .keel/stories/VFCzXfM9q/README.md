---
# system-managed
id: VFCzXfM9q
status: backlog
created_at: 2026-03-28T16:41:19
updated_at: 2026-03-28T16:43:21
# authored
title: Extend Gatherer Contract For Planner Trace
type: feat
operator-signal:
scope: VFCzL9KKd/VFCzWHL1Y
index: 1
---

# Extend Gatherer Contract For Planner Trace

## Summary

Extend the typed context-gathering contract so autonomous planners can return
trace metadata, stop reasons, retained artifacts, and warnings alongside the
existing synthesis-ready evidence bundle.

## Acceptance Criteria

- [ ] The gatherer request/result surface can represent planner strategy, planner trace or summary, planner stop reason, retained artifacts, and warnings without weakening the evidence-first contract. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [ ] Existing non-autonomous gatherers remain expressible through the same port without pretending to return planner metadata they do not have. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [ ] Foundational architecture or policy docs explain that autonomous gatherers return evidence plus planner metadata for downstream synthesis rather than final answers. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->
