---
# system-managed
id: VFDvES5lF
status: backlog
created_at: 2026-03-28T20:30:28
updated_at: 2026-03-28T20:36:48
# authored
title: Rewrite Foundational Docs Around Recursive Harness Backbone
type: feat
operator-signal:
scope: VFDv1i61H/VFDv3gE5m
index: 5
---

# Rewrite Foundational Docs Around Recursive Harness Backbone

## Summary

Rewrite the foundational docs so the recursive harness is the documented
backbone architecture of the paddles mech suit, with honest notes about the
current interim runtime.

## Acceptance Criteria

- [ ] `README.md` explains the recursive planner-loop backbone and includes architecture diagrams for interpretation context, recursive execution, and model routing. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Supporting foundational docs stay aligned with the README on operator memory, planner/synth separation, and non-special-casing of Keel. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] The docs clearly distinguish intended backbone architecture from the current implementation snapshot. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
- [ ] The documented architecture keeps Keel in the evidence layer rather than elevating it to a first-class runtime intent. [SRS-07/AC-04] <!-- verify: manual, SRS-07:start:end -->
- [ ] The documented recursive harness contract stays general-purpose across repositories and evidence domains rather than Keel-specific. [SRS-NFR-04/AC-05] <!-- verify: manual, SRS-NFR-04:start:end -->
