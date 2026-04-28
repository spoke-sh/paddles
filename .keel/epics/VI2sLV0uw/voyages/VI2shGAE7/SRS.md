# Rewrite Docs To Renamed Vocabulary And Drop Aspirational Sections - SRS

## Summary

Epic: VI2sLV0uw
Goal: Rewrite README, ARCHITECTURE, CONSTITUTION, PROTOCOL, STAGE, POLICY, INSTRUCTIONS, CONFIGURATION, and AGENTS to use the renamed vocabulary; remove or move-to-roadmap sections that describe non-existent capability (automatic tier promotion, concurrent sibling generation, deterministic entity resolution as deterministic, specialist brains beyond session-continuity-v1).

## Scope

### In Scope

- [SCOPE-01] Rewrite `README.md` and `ARCHITECTURE.md` to use the renamed vocabulary (`AgentRuntime`, `agent_loop`, `Tool` / `ToolExecutor`, `subagents`, `runtime_profile`, `retriever`, `trace`/`inspector`, etc.) and to drop or move-to-roadmap any sections describing capability paddles does not ship today (automatic tier promotion, concurrent sibling generation, deterministic entity resolution framed as deterministic, specialist brains beyond `session-continuity-v1`).
- [SCOPE-02] Update `CONSTITUTION.md`, `PROTOCOL.md`, `STAGE.md`, `POLICY.md`, `INSTRUCTIONS.md`, `CONFIGURATION.md`, and `AGENTS.md` to use the renamed vocabulary and remove jargon that has no implementation referent.
- [SCOPE-03] Clearly mark any remaining aspirational sections with a "Roadmap" or "Not yet shipped" callout so docs no longer overpromise.

### Out of Scope

- [SCOPE-04] Adding new operator-facing tutorials, walkthroughs, or demo content beyond what already exists.
- [SCOPE-05] Restructuring the docs site (`apps/docs`) information architecture; only prose updates are in scope.
- [SCOPE-06] Translating docs or producing API reference output beyond what `cargo doc` already generates.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | After this voyage, the listed top-level docs use the renamed vocabulary, contain no references to the retired bespoke terms (`MechSuitService`, `*Chamber`, `ExecutionHand`, etc.), and either describe a shipped capability or sit under an explicit "Not yet shipped / Roadmap" heading. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-01 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Doc rewrites must not introduce new bespoke vocabulary; if a new concept is needed it adopts an industry-standard term from the agent-tooling field. | SCOPE-01, SCOPE-02 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
