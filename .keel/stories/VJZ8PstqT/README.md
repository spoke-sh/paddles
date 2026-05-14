---
# system-managed
id: VJZ8PstqT
status: icebox
created_at: 2026-05-13T21:30:04
updated_at: 2026-05-13T21:36:53
# authored
title: Rename Planner Synthesizer Gatherer Ports To Turn Phases
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 2
---

# Rename Planner Synthesizer Gatherer Ports To Turn Phases

## Summary

Rename planner, synthesizer, and gatherer ports/modules where they encode the
old lane architecture. Preserve behavior under clearer turn phase names such as
action selection, final rendering, retrieval, and evidence.

## Acceptance Criteria

- [ ] Internal planner/synthesizer/gatherer names are replaced where they describe lane architecture rather than unavoidable compatibility. [SRS-02/AC-01] <!-- verify: automated, SRS-02:start:end -->
- [ ] The turn loop still exposes tested behavior for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] Prompt and execution-contract tests continue to expose live capabilities and enforced constraints without synthetic controller-authored plans. [SRS-02/AC-03] <!-- verify: automated, SRS-02:start:end -->
