---
# system-managed
id: VGcvOsjRh
status: backlog
created_at: 2026-04-12T17:36:49
updated_at: 2026-04-12T17:39:30
# authored
title: Project Mode State Findings And Clarification Across Surfaces
type: feat
operator-signal:
scope: VGb1c1pAR/VGcvNTG74
index: 3
---

# Project Mode State Findings And Clarification Across Surfaces

## Summary

Project mode transitions, review findings, and structured clarification
exchanges across trace, transcript, UI, API, and docs so operators can see why
the harness paused, reviewed, or changed stance.

## Acceptance Criteria

- [ ] Mode entry, exit, and structured clarification exchanges remain visible in runtime traces and operator-facing projections. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Invalid or unavailable mode requests degrade honestly with typed results instead of silently falling back to default execution behavior. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Review findings and structured clarification requests remain auditable through replay and transcript projections. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end -->
