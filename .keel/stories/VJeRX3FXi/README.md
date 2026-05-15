---
# system-managed
id: VJeRX3FXi
status: backlog
created_at: 2026-05-14T19:17:22
updated_at: 2026-05-14T19:19:31
# authored
title: Rename Collaboration Runtime Contract
type: refactor
operator-signal:
scope: VJeQx1O20/VJeRAPzHh
index: 1
---

# Rename Collaboration Runtime Contract

## Summary

Rename and reshape the agent-loop `collaboration` field into a turn contract or turn policy concept that accurately describes mutation posture, output contract, clarification policy, and mode status.

## Acceptance Criteria

- [ ] Agent-loop/application internals use `turn_contract` or `turn_policy` naming instead of `collaboration` for runtime policy. [SRS-01/AC-01] <!-- verify: rg -n "context\\.collaboration|collaboration_runtime_notes|CollaborationModeResult" src/application src/domain src/infrastructure, SRS-01:start:end -->
- [ ] Existing planning, execution, and review semantics are preserved under the renamed contract. [SRS-02/AC-02] <!-- verify: cargo test turn_contract_preserves_mode_semantics -- --nocapture, SRS-02:start:end -->
- [ ] Any retained collaboration terminology is documented as external or serialized compatibility. [SRS-NFR-02/AC-03] <!-- verify: rg -n "collaboration" src ARCHITECTURE.md CONFIGURATION.md, SRS-NFR-02:start:end -->
