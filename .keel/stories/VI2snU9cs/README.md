---
# system-managed
id: VI2snU9cs
status: backlog
created_at: 2026-04-27T18:38:26
updated_at: 2026-04-27T18:46:11
# authored
title: Promote PlanOnly To First-Class Plan Mode
type: feat
operator-signal:
scope: VI2sKP2cz/VI2sgLj5t
index: 1
---

# Promote PlanOnly To First-Class Plan Mode

## Summary

Make plan mode an operator-visible, end-to-end flow: enter via `/plan`, see the proposed action / edit summary, approve or decline. Add the slash command surface (`/help`, `/plan`, `/compact`, `/cost`, `/agents`, `/mcp`) alongside the existing `/login`, `/model`, `/resume`. Slash commands for capability that has not shipped (subagents, MCP) reply with an honest "not yet wired" notice rather than 404.

## Acceptance Criteria

- [ ] `/plan` enters plan mode in the TUI and the operator can review and approve or decline the proposed action before any tool runs. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] `/help`, `/plan`, `/compact`, `/cost`, `/agents`, and `/mcp` are recognized in the TUI; each either performs its action or prints an honest "not yet wired" notice. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Governance denials surface to the operator (via a TUI prompt or inline notice) instead of being silently summarized back to the planner only. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
