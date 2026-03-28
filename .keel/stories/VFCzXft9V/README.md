---
# system-managed
id: VFCzXft9V
status: backlog
created_at: 2026-03-28T16:41:19
updated_at: 2026-03-28T16:43:21
# authored
title: Add Sift Autonomous Gatherer Adapter
type: feat
operator-signal:
scope: VFCzL9KKd/VFCzWHL1Y
index: 2
---

# Add Sift Autonomous Gatherer Adapter

## Summary

Add a local Sift-backed autonomous gatherer adapter that wraps the supported
upstream autonomous planner runtime and returns a typed evidence-first result to
the synthesizer lane.

## Acceptance Criteria

- [ ] A new local autonomous gatherer adapter maps `ContextGatherRequest` into `Sift::search_autonomous` and returns synthesis-ready evidence plus planner metadata. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [ ] The adapter defaults to the heuristic planner strategy and keeps model-driven planner support optional and capability-gated. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [ ] Targeted tests cover planner-response mapping, capability reporting, and failure paths that must degrade safely to the controller fallback path. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
