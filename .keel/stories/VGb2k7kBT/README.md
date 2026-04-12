---
# system-managed
id: VGb2k7kBT
status: done
created_at: 2026-04-12T09:53:26
updated_at: 2026-04-12T10:23:10
# authored
title: Route Shell And Workspace Hands Through The Permission Gate
type: feat
operator-signal:
scope: VGb1c0pAN/VGb2gViJ2
index: 2
started_at: 2026-04-12T10:11:59
submitted_at: 2026-04-12T10:23:10
completed_at: 2026-04-12T10:23:10
---

# Route Shell And Workspace Hands Through The Permission Gate

## Summary

Wrap the existing shell and workspace-edit hands in the shared permission gate
so side effects occur only when the active execution-governance profile allows
them, and blocked requests return structured deny or escalation results instead
of falling through to raw execution.

## Acceptance Criteria

- [x] Shell and workspace-edit requests declare the permissions they need and run through one shared gate before side effects occur. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Requests that exceed the active posture return structured deny or escalation outcomes instead of retrying with broader authority. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Escalation outcomes can scope bounded reuse metadata without permanently widening later execution authority. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] Policy-evaluation failures fail closed and surface explicit diagnostics rather than implicitly widening execution authority. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-4.log-->
