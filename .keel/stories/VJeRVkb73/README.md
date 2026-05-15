---
# system-managed
id: VJeRVkb73
status: backlog
created_at: 2026-05-14T19:17:17
updated_at: 2026-05-14T19:19:26
# authored
title: Collapse Initial Action Interface
type: refactor
operator-signal:
scope: VJeQx1O20/VJeRAOoHj
index: 2
---

# Collapse Initial Action Interface

## Summary

Collapse the separate initial-action API and runtime planning structs that let normal turns make a model-owned decision before the agent loop starts.

## Acceptance Criteria

- [ ] Normal runtime code no longer calls `select_initial_action`. [SRS-04/AC-01] <!-- verify: rg -n "select_initial_action" src/application src/domain src/infrastructure, SRS-04:start:end -->
- [ ] `PromptExecutionPlan` and `PromptExecutionPath` are removed from the normal runtime path. [SRS-04/AC-02] <!-- verify: rg -n "PromptExecutionPlan|PromptExecutionPath" src/application src/domain src/infrastructure, SRS-04:start:end -->
- [ ] Provider compatibility, if still needed, is isolated away from turn orchestration. [SRS-04/AC-03] <!-- verify: cargo test action_selection_initial_compatibility_is_not_runtime_routing -- --nocapture, SRS-04:start:end -->
- [ ] Full library tests pass after the entry-point interface collapse. [SRS-NFR-02/AC-04] <!-- verify: cargo test --lib, SRS-NFR-02:start:end -->
