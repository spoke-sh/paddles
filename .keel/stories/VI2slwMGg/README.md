---
# system-managed
id: VI2slwMGg
status: done
created_at: 2026-04-27T18:38:20
updated_at: 2026-04-27T19:51:11
# authored
title: Rename MechSuitService to AgentRuntime
type: refactor
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 1
started_at: 2026-04-27T19:47:54
completed_at: 2026-04-27T19:51:11
---

# Rename MechSuitService to AgentRuntime

## Summary

Mechanical, behavior-preserving rename of the `MechSuitService` god-object to `AgentRuntime`. Update every type reference, factory closure type, trait impl, test, and trace identifier across `src/`, `tests/`, `apps/`, and the keel artifacts that mention the type. No functional changes — public CLI flags, on-disk trace schemas, and HTTP routes remain identical.

## Acceptance Criteria

- [x] `struct MechSuitService` is renamed to `struct AgentRuntime` and every `MechSuitService` reference across `src/` and `tests/` is updated (verified by `git grep -F MechSuitService` returning zero hits). [SRS-01/AC-01] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] `cargo check`, `cargo test --lib`, and `cargo clippy --all-targets -- -D warnings` pass with the rename in place and no behavior change. [SRS-01/AC-02] <!-- verify: cargo test --lib, SRS-01:start:end -->
- [x] Public CLI flags, web HTTP routes, and persisted trace record schemas are unchanged — the rename is in-process types only. [SRS-01/AC-03] <!-- verify: cargo test --lib, SRS-01:start:end -->
