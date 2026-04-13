---
# system-managed
id: VGcvOsBRp
status: backlog
created_at: 2026-04-12T17:36:49
updated_at: 2026-04-12T17:39:30
# authored
title: Route Planning And Review Behavior Through The Recursive Harness
type: feat
operator-signal:
scope: VGb1c1pAR/VGcvNTG74
index: 2
---

# Route Planning And Review Behavior Through The Recursive Harness

## Summary

Route planning, execution, and review behaviors through the existing recursive
harness so planning can clarify without mutating, review can inspect local
changes findings-first, and execution remains the default bounded mutation
path.

## Acceptance Criteria

- [ ] Planning mode supports non-mutating exploration and bounded structured clarification when the runtime genuinely needs user input. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Review mode inspects local changes and emits findings-first output with grounded file or line references plus residual risks or gaps when needed. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Execution mode remains the default mutation path while honoring mode-specific permissions, escalation rules, and fail-closed restrictions. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] Planning and review preserve the same recursive-harness identity and evidence standards as execution mode. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end -->
- [ ] Mode-specific mutation posture is encoded structurally so planning and review restrictions can fail closed. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end -->
