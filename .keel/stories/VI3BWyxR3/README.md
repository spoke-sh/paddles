---
# system-managed
id: VI3BWyxR3
status: done
created_at: 2026-04-27T19:52:51
updated_at: 2026-04-27T22:43:40
# authored
title: Rename Chamber Wrappers To Plain Modules
type: refactor
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 2
started_at: 2026-04-27T22:28:06
completed_at: 2026-04-27T22:43:40
---

# Rename Chamber Wrappers To Plain Modules

## Summary

Delete the stateless `*Chamber` wrappers in `src/application/` (`RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and any siblings) and migrate their methods to plain function modules. The new module names match SCOPE-02: `agent_loop`, `context_assembly`, `synthesis`, `turn`. Behavior unchanged; CLI flags, web routes, and trace schemas untouched.

## Acceptance Criteria

- [x] Every `*Chamber` wrapper struct in `src/application/` is deleted (`InterpretationChamber`, `SynthesisChamber`, `ConversationReadModelChamber`, `RecursiveControlChamber`, `TurnOrchestrationChamber`); its methods are now free functions in modules named for the phase (`context_assembly`, `synthesis`, `conversation_read_model`, `agent_loop`, `turn`). `git grep -E '\\bChamber\\b' src/application/` returns only string-literal hits in test fixtures. [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] All call sites that did `self.service.foo_chamber().bar()` (or `self.foo_chamber().bar()` on `AgentRuntime`) are updated to call the module-level functions directly with `service` as the first argument; the corresponding `AgentRuntime::*_chamber()` accessors are removed. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] `cargo check`, `cargo test --lib` (782 passing), and `cargo clippy --all-targets -- -D warnings` pass with the migration in place and no behavior change. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->
