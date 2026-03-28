---
# system-managed
id: VF7tCKEgw
status: backlog
created_at: 2026-03-27T19:44:45
updated_at: 2026-03-27T19:48:12
# authored
title: Build Sift Session Controller
type: feat
operator-signal:
scope: VF7t633ux/VF7tAvs7B
index: 1
---

# Build Sift Session Controller

## Summary

Replace the wonopcode-owned prompt loop with a Paddles-managed Sift session
controller that owns conversational state and retained context.

## Acceptance Criteria

- [ ] `MechSuitService` executes prompts through a Sift session controller rather than `wonopcode_core::PromptLoop` and `Instance`. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Prompt turns retain prior agent turns and bounded workspace evidence through Sift context state. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
