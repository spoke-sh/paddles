---
# system-managed
id: VI3BWyxR3
status: icebox
created_at: 2026-04-27T19:52:51
updated_at: 2026-04-27T19:52:51
# authored
title: Rename Chamber Wrappers To Plain Modules
type: refactor
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 2
---

# Rename Chamber Wrappers To Plain Modules

## Summary

Delete the stateless `*Chamber` wrappers in `src/application/` (`RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and any siblings) and migrate their methods to plain function modules. The new module names match SCOPE-02: `agent_loop`, `context_assembly`, `synthesis`, `turn`. Behavior unchanged; CLI flags, web routes, and trace schemas untouched.

## Acceptance Criteria

- [ ] Every `*Chamber` wrapper struct in `src/application/` is deleted; its methods become free functions in a module named for the phase. [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [ ] All call sites that did `self.foo_chamber().bar()` are updated to call the module-level functions directly. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [ ] `cargo check`, `cargo test --lib`, and `cargo clippy --all-targets -- -D warnings` pass with the rename in place and no behavior change. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->
