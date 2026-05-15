---
# system-managed
id: VJeRVkb73
status: done
created_at: 2026-05-14T19:17:17
updated_at: 2026-05-14T19:51:41
# authored
title: Collapse Initial Action Interface
type: refactor
operator-signal:
scope: VJeQx1O20/VJeRAOoHj
index: 2
started_at: 2026-05-14T19:43:01
completed_at: 2026-05-14T19:51:41
---

# Collapse Initial Action Interface

## Summary

Collapse the separate initial-action API and runtime planning structs that let normal turns make a model-owned decision before the agent loop starts.

## Acceptance Criteria

- [x] Normal runtime code no longer calls `select_initial_action`. [SRS-04/AC-01] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "select_initial_action" src/application src/domain src/infrastructure; then exit 1; else test $? -eq 1; fi', SRS-04:start:end, proof: ac-1.log-->
- [x] `PromptExecutionPlan` and `PromptExecutionPath` are removed from the normal runtime path. [SRS-04/AC-02] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "PromptExecutionPlan|PromptExecutionPath" src/application src/domain src/infrastructure; then exit 1; else test $? -eq 1; fi', SRS-04:start:end, proof: ac-2.log-->
- [x] Provider compatibility, if still needed, is isolated away from turn orchestration. [SRS-04/AC-03] <!-- verify: cargo test action_selection_initial_compatibility_is_not_runtime_routing -- --nocapture, SRS-04:start:end, proof: ac-3.log-->
- [x] Full library tests pass after the entry-point interface collapse. [SRS-NFR-02/AC-04] <!-- verify: cargo test --lib, SRS-NFR-02:start:end, proof: ac-4.log-->
