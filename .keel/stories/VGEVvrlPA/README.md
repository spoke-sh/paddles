---
# system-managed
id: VGEVvrlPA
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Separate Runtime Store Transport And Event Reduction
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsWLk2
index: 3
---

# Separate Runtime Store Transport And Event Reduction

## Summary

Separate bootstrap, projection streaming, and event-log reduction into dedicated store/client modules while keeping the current shell-facing runtime store contract intact.

## Acceptance Criteria

- [ ] Runtime bootstrap, SSE projection updates, and send-turn transport move behind dedicated store/client modules without changing the `useRuntimeStore` consumer contract. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Event accumulation semantics and prompt-history bootstrap remain covered after the transport split. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
