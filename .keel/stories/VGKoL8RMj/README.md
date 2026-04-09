---
# system-managed
id: VGKoL8RMj
status: done
created_at: 2026-04-09T15:15:52
updated_at: 2026-04-09T15:44:16
# authored
title: Define Transport Capability Vocabulary And Lifecycle Contract
type: feat
operator-signal:
scope: VGKnsYg1z/VGKoF0hsS
index: 1
started_at: 2026-04-09T15:21:23
completed_at: 2026-04-09T15:44:16
---

# Define Transport Capability Vocabulary And Lifecycle Contract

## Summary

Define the first shared transport contract for native connection capabilities and lifecycle semantics. This story should name the phases and vocabulary that HTTP, SSE, WebSocket, and Transit adapters will all use so later transport work does not create duplicate protocol-specific meanings.

## Acceptance Criteria

- [x] The shared native transport vocabulary defines lifecycle phases, negotiated capabilities, and stable session identity semantics for every transport adapter [SRS-01/AC-01] <!-- verify: cargo test native_transport_, SRS-01:start:end -->
- [x] The shared lifecycle contract is explicit enough that later transport stories can consume it without re-defining protocol-specific state names [SRS-01/AC-02] <!-- verify: cargo test native_transport_, SRS-01:start:end -->
