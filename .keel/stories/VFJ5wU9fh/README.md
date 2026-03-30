---
# system-managed
id: VFJ5wU9fh
status: backlog
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T17:47:05
# authored
title: Document And Prove The Controller-Versus-Model Boundary
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 4
---

# Document And Prove The Controller-Versus-Model Boundary

## Summary

Update the foundational docs and ship proof artifacts that make the resulting
controller-versus-model boundary explicit so the heuristic-removal work remains
auditable and does not regress into hidden controller reasoning later.

## Acceptance Criteria

- [ ] Foundational docs describe which decisions are model-judged and which remain controller-owned constraints. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Proof artifacts demonstrate at least one end-to-end turn where model-judged interpretation and fallback replace prior heuristics. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] The docs stay generic across repositories rather than hardcoding a project-specific replacement intent taxonomy. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
