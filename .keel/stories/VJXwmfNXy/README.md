---
# system-managed
id: VJXwmfNXy
status: backlog
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T16:40:08
# authored
title: Introduce Unified Agent Action Domain Contract
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlCA0P
index: 1
---

# Introduce Unified Agent Action Domain Contract

## Summary

Introduce the domain-level recursive agent action contract that can represent
both the first model decision and later loop decisions. This story should make
the target contract executable before runtime call sites migrate.

## Acceptance Criteria

- [ ] A unified agent action domain contract represents terminal `answer`, workspace action, `refine`, `branch`, and `stop` decisions for first and later loop steps. [SRS-01/AC-01] <!-- verify: cargo test agent_action_domain_contract --lib, SRS-01:start:end -->
- [ ] Contract tests prove the unified action labels cover the existing first-action and recursive-action labels without introducing a second hidden vocabulary. [SRS-02/AC-02] <!-- verify: cargo test agent_action_contract_preserves_existing_action_labels --lib, SRS-02:start:end -->
- [ ] Transitional compatibility, if needed, is explicit and bounded by tests rather than becoming a second public contract. [SRS-NFR-01/AC-03] <!-- verify: cargo test agent_action_compatibility_is_explicit --lib, SRS-NFR-01:start:end -->
