---
id: VE4H1EBG0
title: Wire Real Core Engine
type: feat
status: done
created_at: 2026-03-16T14:22:15
started_at: 2026-03-16T14:25:00
updated_at: 2026-03-16T14:27:29
operator-signal: 
scope: VE47wLZRk/VE4Gv6Gv3
index: 1
submitted_at: 2026-03-16T14:27:22
completed_at: 2026-03-16T14:27:29
---

# Wire Real Core Engine

## Summary

This story involves the actual technical wiring of the `wonopcode-core` engine into the `paddles` CLI, replacing the placeholder simulation.

## Acceptance Criteria

- [x] Project builds with real `wonopcode-core` and `openssl`. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end -->
- [x] CLI successfully instantiates `Instance` and `Session`. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [x] CLI executes a real `PromptLoop`. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
