---
# system-managed
id: VI2slwMGg
status: backlog
created_at: 2026-04-27T18:38:20
updated_at: 2026-04-27T18:46:11
# authored
title: Rename MechSuitService to AgentRuntime
type: refactor
operator-signal:
scope: VI2sJZcV9/VI2sfbhqT
index: 1
---

# Rename MechSuitService to AgentRuntime

## Summary

Mechanical, behavior-preserving rename of the `MechSuitService` god-object to `AgentRuntime`. Update every type reference, factory closure type, trait impl, test, and trace identifier across `src/`, `tests/`, `apps/`, and the keel artifacts that mention the type. No functional changes — public CLI flags, on-disk trace schemas, and HTTP routes remain identical.

## Acceptance Criteria

- [ ] `struct MechSuitService` is renamed to `struct AgentRuntime` and every `MechSuitService` reference across `src/` and `tests/` is updated. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] `cargo check`, `cargo test`, and `keel doctor` pass with the rename in place and no behavior change. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Public CLI flags, web HTTP routes, and persisted trace record schemas are unchanged by the rename. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
