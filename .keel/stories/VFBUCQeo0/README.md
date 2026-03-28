---
# system-managed
id: VFBUCQeo0
status: backlog
created_at: 2026-03-28T10:30:32
updated_at: 2026-03-28T10:33:21
# authored
title: Refactor Runtime For Gatherer And Synthesizer Lanes
type: feat
operator-signal:
scope: VFBTXlHli/VFBTYpPo6
index: 2
---

# Refactor Runtime For Gatherer And Synthesizer Lanes

## Summary

Refactor runtime wiring so Paddles can configure and prepare separate gatherer
and synthesizer lanes while keeping the current local answer path as the
default.

## Acceptance Criteria

- [ ] Runtime and configuration wiring support distinct gatherer and synthesizer lanes instead of assuming one active answer model path. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] The default answer/tool path remains local-first and operational without any mandatory new network dependency for common prompt handling. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] When no gatherer lane is configured, the synthesizer lane remains the configured default runtime. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
