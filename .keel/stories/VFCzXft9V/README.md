---
# system-managed
id: VFCzXft9V
status: done
created_at: 2026-03-28T16:41:19
updated_at: 2026-03-28T17:22:05
# authored
title: Add Sift Autonomous Gatherer Adapter
type: feat
operator-signal:
scope: VFCzL9KKd/VFCzWHL1Y
index: 2
started_at: 2026-03-28T17:14:41
submitted_at: 2026-03-28T17:22:03
completed_at: 2026-03-28T17:22:05
---

# Add Sift Autonomous Gatherer Adapter

## Summary

Add a local Sift-backed autonomous gatherer adapter that wraps the supported
upstream autonomous planner runtime and returns a typed evidence-first result to
the synthesizer lane.

## Acceptance Criteria

- [x] A new local autonomous gatherer adapter maps `ContextGatherRequest` into `Sift::search_autonomous` and returns synthesis-ready evidence plus planner metadata. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] The adapter defaults to the heuristic planner strategy and keeps model-driven planner support optional and capability-gated. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] Targeted tests cover planner-response mapping, capability reporting, and failure paths that must degrade safely to the controller fallback path. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
