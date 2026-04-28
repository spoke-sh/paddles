# Plan Mode And Slash Command Parity - Product Requirements

## Problem Statement

TUI ships only /login, /model, /resume — far short of the slash-command surface operators expect from Claude Code, Codex, Gemini CLI, and OpenCode, and Collaboration::PlanOnly never reaches users as a first-class plan mode. Promote plan mode end-to-end and add /help, /plan, /compact, /cost, /agents, /mcp at minimum.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Resolve the problem described above for the primary user. | A measurable outcome is defined for this problem | Target agreed during planning |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | The person or team most affected by the problem above. | A clearer path to the outcome this epic should improve. |

## Scope

### In Scope

- [SCOPE-01] Promote `Collaboration::PlanOnly` (or equivalent) to a first-class `plan` mode in the TUI and CLI: the operator can enter plan mode, see the proposed action, and approve/decline before any tool runs.
- [SCOPE-02] Add `/plan` slash command to enter plan mode and `/help` to enumerate available commands.
- [SCOPE-03] Add `/compact`, `/cost`, `/agents`, `/mcp` slash commands as discoverable surfaces, even if some are stubs that report "not yet wired" for capability that hasn't shipped.
- [SCOPE-04] Surface governance denials as visible TUI prompts (or inline notices) instead of silently summarizing them back to the planner.

### Out of Scope

- [SCOPE-05] Implementing real MCP server discovery / invocation (the `/mcp` command may report unavailability).
- [SCOPE-06] Implementing real subagent execution (the `/agents` command may report unavailability).
- [SCOPE-07] Cost model accounting beyond what the existing trace recorder already exposes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Deliver the primary user workflow for this epic end-to-end. | GOAL-01 | must | Establishes the minimum functional capability needed to achieve the epic goal. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01 | must | Keeps operations stable and makes regressions detectable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
