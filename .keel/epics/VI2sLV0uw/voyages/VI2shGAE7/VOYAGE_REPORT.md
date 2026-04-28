# VOYAGE REPORT: Rewrite Docs To Renamed Vocabulary And Drop Aspirational Sections

## Voyage Metadata
- **ID:** VI2shGAE7
- **Epic:** VI2sLV0uw
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 1/1 stories complete

## Implementation Narrative
### Rewrite README And Architecture For Renamed Vocabulary
- **ID:** VI2snWAfM
- **Status:** done

#### Summary
Rewrite `README.md` and `ARCHITECTURE.md` to use the renamed vocabulary (`AgentRuntime`, `agent_loop`, `Tool` / `ToolExecutor`, `subagents`, `runtime_profile`, `retriever`, `trace`/`inspector`, `controller_signals`, `reasoning_signals`, `evidence_check`, `compaction_trigger`) and remove or move-to-roadmap any sections describing capability that paddles does not ship today. Subsequent stories under voyage VI2shGAE7 will sweep the rest of the top-level docs.

#### Acceptance Criteria
- [x] `README.md` intro and "Why This Architecture" use `agent runtime` framing instead of "mech suit"; `ARCHITECTURE.md` "Engine, Its Chambers, And The Governor" section is rewritten as "The Agent Runtime And Its Phases" and explicitly notes that `*Chamber` and `MechSuitService` are being retired under mission VI2q5DKHe (rename to `AgentRuntime` already shipped). Full prose sweep of remaining `gatherer`/`forensic` references is tracked under follow-up stories on voyage VI2shGAE7. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] Both `README.md` and `ARCHITECTURE.md` carry an explicit "Roadmap / Not Yet Shipped" section listing real subagents, MCP, plan-mode review/approve UX, automatic tier promotion, concurrent sibling generation, operator-triggered `/compact`, and the deterministic-resolver framing. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [x] No new bespoke vocabulary introduced; the rewrite adopts industry-standard terms (`agent runtime`, `phase`, `subagents`, `MCP`, `plan mode`, `slash command`). [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->


