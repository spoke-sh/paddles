---
# system-managed
id: VI1ztz0MP
status: done
created_at: 2026-04-27T15:00:23
updated_at: 2026-04-27T15:10:17
# authored
title: Extract Planner Executor And Capability Modules
type: chore
operator-signal:
scope: VI1zeXMOr/VI1zga7t4
index: 1
started_at: 2026-04-27T15:00:42
completed_at: 2026-04-27T15:10:17
---

# Extract Planner Executor And Capability Modules

## Summary

Extract the recursive planner executor loop, planner action execution helpers, and external-capability execution helpers out of `src/application/mod.rs` into dedicated application modules, preserving existing runtime behavior and governance/evidence emission.

## Acceptance Criteria

- [x] The recursive planner executor loop lives in `src/application/recursive_control.rs`, not as a direct `MechSuitService` method in `src/application/mod.rs`. [SRS-01/AC-01] <!-- verify: cargo test -q execution_ -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Planner action execution helpers for query/evidence-source mapping and governed terminal command execution live outside `src/application/mod.rs`. [SRS-02/AC-02] <!-- verify: cargo test -q planner_action_execution -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] External-capability invocation formatting, governed invocation, result summarization, and evidence projection live outside `src/application/mod.rs`. [SRS-03/AC-03] <!-- verify: cargo test -q external_capability -- --nocapture, SRS-03:start:end, proof: ac-3.log-->
- [x] Existing runtime service APIs and planner action behavior remain unchanged after extraction. [SRS-04/AC-04] <!-- verify: cargo test -q process_prompt_ -- --nocapture, SRS-04:start:end, proof: ac-4.log-->
- [x] Execution governance decisions and evidence records continue to be emitted for terminal, workspace, and external-capability actions. [SRS-NFR-01/AC-05] <!-- verify: cargo test -q governance -- --nocapture, SRS-NFR-01:start:end, proof: ac-5.log-->
- [x] The final proof includes `cargo test` and `keel doctor`. [SRS-NFR-02/AC-06] <!-- verify: cargo test -q && keel doctor, SRS-NFR-02:start:end, proof: ac-6.log-->
