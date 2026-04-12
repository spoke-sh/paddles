---
# system-managed
id: VGbPaBICN
status: backlog
created_at: 2026-04-12T11:24:10
updated_at: 2026-04-12T11:25:34
# authored
title: Define Turn And Thread Control Contracts
type: feat
operator-signal:
scope: VGb1c1AAK/VGbPWnUh2
index: 1
---

# Define Turn And Thread Control Contracts

## Summary

Define the typed turn-control, thread-control, and runtime-item contracts so
Paddles can express same-turn steering, thread lifecycle transitions, and live
plan or diff state as durable runtime semantics instead of surface-specific
prompt conventions.

## Acceptance Criteria

- [ ] The runtime defines typed contracts for turn and thread control operations, control results, and shared runtime items for plan, diff, command, file, and control-state updates. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The contract vocabulary defines turn and thread control semantics independently of any one operator surface or prompt phrasing. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] The contract model builds directly on the existing recorder, replay, and thread-lineage substrate instead of introducing a parallel state store. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end -->
