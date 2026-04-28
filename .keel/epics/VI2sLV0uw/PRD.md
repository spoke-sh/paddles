# Doc Truthfulness Pass - Product Requirements

## Problem Statement

README, ARCHITECTURE, CONSTITUTION, PROTOCOL, STAGE, POLICY, INSTRUCTIONS, CONFIGURATION, and AGENTS describe capability that paddles does not ship — automatic tier promotion, concurrent sibling generation, deterministic entity resolution as deterministic, specialist brains beyond session-continuity-v1 — and use the bespoke vocabulary that this mission is renaming. Rewrite docs to use the renamed terms and either ship the aspirational sections or remove them.

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

- [SCOPE-01] Rewrite `README.md` and `ARCHITECTURE.md` to use the renamed vocabulary (`AgentRuntime`, `agent_loop`, `Tool` / `ToolExecutor`, `subagents`, `runtime_profile`, `retriever`, `trace`/`inspector`, etc.) and to drop or move-to-roadmap any sections describing capability paddles does not ship today (automatic tier promotion, concurrent sibling generation, deterministic entity resolution framed as deterministic, specialist brains beyond `session-continuity-v1`).
- [SCOPE-02] Update `CONSTITUTION.md`, `PROTOCOL.md`, `STAGE.md`, `POLICY.md`, `INSTRUCTIONS.md`, `CONFIGURATION.md`, and `AGENTS.md` to use the renamed vocabulary and remove jargon that has no implementation referent.
- [SCOPE-03] Clearly mark any remaining aspirational sections with a "Roadmap" or "Not yet shipped" callout so docs no longer overpromise.

### Out of Scope

- [SCOPE-04] Adding new operator-facing tutorials, walkthroughs, or demo content beyond what already exists.
- [SCOPE-05] Restructuring the docs site (`apps/docs`) information architecture; only prose updates are in scope.
- [SCOPE-06] Translating docs or producing API reference output beyond what `cargo doc` already generates.

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
