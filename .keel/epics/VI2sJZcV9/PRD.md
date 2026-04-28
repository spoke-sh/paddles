# Tier-1 Idiomatic Vocabulary Renames - Product Requirements

## Problem Statement

Bespoke vocabulary (MechSuitService, *Chamber, ExecutionHand, WorkspaceAction, PlannerAction, instruction_frame, specialist_brains, harness_profile, gatherer, forensics, compaction_cue, premise_challenge, deliberation_signals, steering_signals) burns calories on every read and obscures concepts the rest of the field already names. Rename to industry-standard agent vocabulary (AgentRuntime, agent_loop / context_assembly / synthesis / turn modules, ToolRunner, Tool / ToolExecutor, AgentStep, system_instructions, subagents, runtime_profile, retriever, trace / inspector, compaction_trigger, evidence_check, reasoning_signals, controller_signals) — one mechanically-reviewable rename per PR, no behavior change.

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

- [SCOPE-01] Rename the `MechSuitService` struct (and `mech_suit*` references) to `AgentRuntime` across `src/`, `tests/`, and trace artifact identifiers.
- [SCOPE-02] Rename the `*Chamber` types (`RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and any siblings) to plain function modules: `agent_loop`, `context_assembly`, `synthesis`, `turn`. The wrapper structs are deleted; their methods move to module-level functions.
- [SCOPE-03] Rename the `recursive_control` module to `agent_loop` and update all `use` paths.

### Out of Scope

- [SCOPE-04] Renames of `ExecutionHand`, `WorkspaceAction`, `PlannerAction`, `instruction_frame`, `specialist_brains`, `harness_profile`, `gatherer`, `forensics`, `compaction_cue`, `premise_challenge`, `deliberation_signals`, `steering_signals` (handled by sibling voyages under epic VI2sJZcV9 in subsequent slices).
- [SCOPE-05] Splitting `src/application/mod.rs` (17,556 lines) into separately-tested services. The chamber rename only flattens names; the file split is a separate mission.
- [SCOPE-06] Behavior changes — this voyage is mechanically rename-only.

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
