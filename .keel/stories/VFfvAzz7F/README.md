---
# system-managed
id: VFfvAzz7F
status: backlog
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T15:27:53
# authored
title: Stage Route Cutover Between Embedded Shell And React App
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 2
---

# Stage Route Cutover Between Embedded Shell And React App

## Summary

Define and implement the controlled cutover seam between the existing embedded shell and the React runtime app so migration can happen without breaking current operator workflows.

## Acceptance Criteria

- [ ] The first React slice preserves the existing Rust backend API surface and keeps the embedded shell available until parity work is complete. [SRS-05/AC-01] <!-- verify: automated, SRS-05:start:end -->
- [ ] Repo documentation states clearly that the embedded shell remains the runtime source of truth until React route cutover is complete. [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end -->
