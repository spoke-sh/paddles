---
# system-managed
id: VFHIX0uKc
status: done
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T11:58:34
# authored
title: Persist Transit Thread Branches And Artifacts
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 2
started_at: 2026-03-29T11:56:51
submitted_at: 2026-03-29T11:58:33
completed_at: 2026-03-29T11:58:34
---

# Persist Transit Thread Branches And Artifacts

## Summary

Create the paddles-owned conversation/thread layer and project its thread
creation, reply, backlink/summary, merge, and checkpoint transitions through
the existing recorder boundary so embedded `transit-core` can durably replay
threaded work without turning `transit-core` into a conversation API.

## Acceptance Criteria

- [x] A paddles-owned conversation/thread layer exists above the recorder boundary and owns the thread DTOs needed by runtime and UX code. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] The layer consumes the new upstream `transit-core` metadata, branch replay, and artifact helper APIs where they simplify low-level plumbing, without turning `transit-core` into the conversation API boundary. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Thread-local replay reconstructs enough mainline and child-thread provenance for later planning and synthesis. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] The implementation works through the existing embedded recorder path, does not require a separate trace server, and remains extractable from paddles later. [SRS-NFR-04/AC-03] <!-- verify: manual, SRS-NFR-04:start:end, proof: ac-4.log-->
