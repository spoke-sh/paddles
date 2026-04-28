---
# system-managed
id: VI2snWAfM
status: backlog
created_at: 2026-04-27T18:38:26
updated_at: 2026-04-27T18:46:12
# authored
title: Rewrite README And Architecture For Renamed Vocabulary
type: docs
operator-signal:
scope: VI2sLV0uw/VI2shGAE7
index: 1
---

# Rewrite README And Architecture For Renamed Vocabulary

## Summary

Rewrite `README.md` and `ARCHITECTURE.md` to use the renamed vocabulary (`AgentRuntime`, `agent_loop`, `Tool` / `ToolExecutor`, `subagents`, `runtime_profile`, `retriever`, `trace`/`inspector`, `controller_signals`, `reasoning_signals`, `evidence_check`, `compaction_trigger`) and remove or move-to-roadmap any sections describing capability that paddles does not ship today. Subsequent stories under voyage VI2shGAE7 will sweep the rest of the top-level docs.

## Acceptance Criteria

- [ ] `README.md` and `ARCHITECTURE.md` use the renamed vocabulary throughout and contain no references to the retired bespoke terms (`MechSuitService`, `*Chamber`, `ExecutionHand`, `WorkspaceAction`, `specialist_brains`, etc.). [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Sections describing unshipped capability (automatic tier promotion, concurrent sibling generation, deterministic entity resolution as deterministic, specialist brains beyond `session-continuity-v1`) are either removed or sit under an explicit "Not yet shipped / Roadmap" heading. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] No new bespoke vocabulary is introduced; any new concept adopts an industry-standard agent-tooling term. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
