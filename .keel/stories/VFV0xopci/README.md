---
# system-managed
id: VFV0xopci
status: in-progress
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T19:02:17
# authored
title: Document Direct Search Boundary Constraints And Capabilities
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 4
started_at: 2026-03-31T19:02:17
---

# Document Direct Search Boundary Constraints And Capabilities

## Summary

Make the direct search boundary explicit in the repo docs so maintainers understand what paddles plans, what sift executes, and which constraints shape the integration.

## Acceptance Criteria

- [ ] Documentation explains that paddles owns recursive planning while sift owns direct retrieval execution. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-1.log-->
- [ ] Documentation describes the supported capabilities and constraints of the direct search boundary, including retrieval progress semantics. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [ ] User-facing search/progress docs no longer center the autonomous planner model for normal paddles retrieval turns. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [ ] Trace and progress terminology in docs align with the runtime labels introduced by the direct retrieval path. [SRS-NFR-03/AC-04] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-4.log-->
