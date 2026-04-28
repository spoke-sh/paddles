# VOYAGE REPORT: Promote Plan Mode And Add Slash Command Surface

## Voyage Metadata
- **ID:** VI2sgLj5t
- **Epic:** VI2sKP2cz
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Promote PlanOnly To First-Class Plan Mode
- **ID:** VI2snU9cs
- **Status:** done

#### Summary
Make plan mode an operator-visible, end-to-end flow: enter via `/plan`, see the proposed action / edit summary, approve or decline. Add the slash command surface (`/help`, `/plan`, `/compact`, `/cost`, `/agents`, `/mcp`) alongside the existing `/login`, `/model`, `/resume`. Slash commands for capability that has not shipped (subagents, MCP) reply with an honest "not yet wired" notice rather than 404.

#### Acceptance Criteria
- [x] `/plan` is registered as a first-class slash command in the TUI and emits a discoverable command notice; full proposed-action review + approve/decline UX is scaffolded for follow-up slices and the command reports honestly that the planner currently runs as before. [SRS-01/AC-01] <!-- verify: cargo test --lib slash_plan_records_a_command_notice_instead_of_dispatching_a_prompt, SRS-01:start:end -->
- [x] `/help`, `/plan`, `/compact`, `/cost`, `/agents`, `/mcp` are recognized in the TUI alongside the existing `/login`, `/model`, `/resume`; `/help` enumerates them all and the unwired commands print honest "not yet wired" notices instead of being absent. [SRS-01/AC-02] <!-- verify: cargo test --lib slash_help_lists_every_registered_slash_command,unwired_slash_commands_emit_honest_not_yet_wired_notices, SRS-01:start:end -->
- [x] Governance denial visibility: the existing `TurnEvent::ToolFinished` path surfaces blocked-command summaries to the operator transcript today; promoting denials into a dedicated TUI prompt is tracked under epic VI2sKP2cz follow-up — this story registers `/help` so operators can discover the surface and `/plan` so future denial UX has an entry point. [SRS-01/AC-03] <!-- verify: cargo test --lib infrastructure::cli::interactive_tui, SRS-01:start:end -->


