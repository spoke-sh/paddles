---
# system-managed
id: VFV0xopci
status: done
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T19:07:24
# authored
title: Document Direct Search Boundary Constraints And Capabilities
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 4
started_at: 2026-03-31T19:02:17
completed_at: 2026-03-31T19:07:24
---

# Document Direct Search Boundary Constraints And Capabilities

## Summary

Make the direct search boundary explicit in the repo docs so maintainers understand what paddles plans, what sift executes, and which constraints shape the integration.

## Acceptance Criteria

- [x] Documentation explains that paddles owns recursive planning while sift owns direct retrieval execution. [SRS-06/AC-01] <!-- verify: rg -n "owns recursive planning|owns retrieval execution|paddles plans|sift retrieves" /home/alex/workspace/spoke-sh/paddles/SEARCH.md /home/alex/workspace/spoke-sh/paddles/README.md, SRS-06:start:end, proof: ac-1.log-->
- [x] Documentation describes the supported capabilities and constraints of the direct search boundary, including retrieval progress semantics. [SRS-06/AC-02] <!-- verify: rg -n "Capabilities|Constraints|initialization|indexing|embedding|retrieval|ranking" /home/alex/workspace/spoke-sh/paddles/SEARCH.md, SRS-06:start:end, proof: ac-2.log-->
- [x] User-facing search/progress docs no longer center the autonomous planner model for normal paddles retrieval turns. [SRS-06/AC-03] <!-- verify: rg -n "sift-direct|compatibility alias" /home/alex/workspace/spoke-sh/paddles/README.md /home/alex/workspace/spoke-sh/paddles/CONFIGURATION.md /home/alex/workspace/spoke-sh/paddles/SEARCH.md /home/alex/workspace/spoke-sh/paddles/ARCHITECTURE.md, SRS-06:start:end, proof: ac-3.log-->
- [x] Trace and progress terminology in docs align with the runtime labels introduced by the direct retrieval path. [SRS-NFR-03/AC-04] <!-- verify: rg -n "initialization|indexing|retrieval|ranking|progress" /home/alex/workspace/spoke-sh/paddles/SEARCH.md /home/alex/workspace/spoke-sh/paddles/README.md /home/alex/workspace/spoke-sh/paddles/ARCHITECTURE.md /home/alex/workspace/spoke-sh/paddles/CONFIGURATION.md, SRS-NFR-03:start:end, proof: ac-4.log-->
