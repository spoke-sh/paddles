---
# system-managed
id: VFBUDSjqS
status: backlog
created_at: 2026-03-28T10:30:36
updated_at: 2026-03-28T10:33:21
# authored
title: Add Context-1 Adapter Boundary And Harness Gate
type: feat
operator-signal:
scope: VFBTXlHli/VFBTYpPo6
index: 4
---

# Add Context-1 Adapter Boundary And Harness Gate

## Summary

Introduce an experimental Context-1 gatherer boundary that reports capability
state honestly, documents the harness expectation, and avoids treating
Context-1 as a drop-in answer runtime.

## Acceptance Criteria

- [ ] An experimental Context-1 adapter boundary exists behind an explicit capability or opt-in gate and reports `available`, `unsupported`, or `harness-required` honestly. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Verbose or debug output reports routing decisions and concise evidence bundle summaries for gatherer-driven turns. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Unsupported or harness-required Context-1 states fail closed with clear operator-visible messaging. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end -->
- [ ] Docs and configuration explain the expected Context-1 harness boundary plus how to inspect missing-context or misrouting behavior. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end -->
