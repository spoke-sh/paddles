---
# system-managed
id: VFHIX0uKc
status: backlog
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T10:25:12
# authored
title: Persist Transit Thread Branches And Artifacts
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 2
---

# Persist Transit Thread Branches And Artifacts

## Summary

Project thread creation, reply, backlink/summary, merge, and checkpoint
transitions through the existing recorder boundary so embedded `transit-core`
can durably replay threaded work without leaking raw transit types into the
domain.

## Acceptance Criteria

- [ ] The recorder path persists explicit thread branch creation, replies, backlinks or summaries, merge decisions, and checkpoints through embedded `transit-core`. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Thread-local replay reconstructs enough mainline and child-thread provenance for later planning and synthesis. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [ ] The implementation works through the existing embedded recorder path and does not require a separate trace server. [SRS-NFR-04/AC-03] <!-- verify: manual, SRS-NFR-04:start:end -->
