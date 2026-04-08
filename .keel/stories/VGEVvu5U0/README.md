---
# system-managed
id: VGEVvu5U0
status: backlog
created_at: 2026-04-08T13:25:07
updated_at: 2026-04-08T13:28:40
# authored
title: Split Web Runtime Tests By Domain Surface
type: feat
operator-signal:
scope: VGEVm5Ibi/VGEVsXLkG
index: 2
---

# Split Web Runtime Tests By Domain Surface

## Summary

Split the runtime web test surface by domain so shell/chat, inspector, manifold, and transit behaviors can be maintained without one kitchen-sink test file.

## Acceptance Criteria

- [ ] Runtime tests are reorganized into domain-focused files with shared setup utilities rather than one monolithic runtime-app test surface. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Domain-level tests continue to cover the major route and shell contracts after the split. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
