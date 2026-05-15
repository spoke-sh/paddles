# Agent Loop Owns Turn Action Selection - Product Requirements

## Problem Statement

Paddles still has a model-selected initial action and controller bootstrap path before the recursive agent loop runs. That split lets turn-mode policy, routing, grounding, edit, and commit pressure steer the request outside the loop, which can make simple questions stall or repeat pre-loop evidence actions instead of letting the agent loop reason over live observations.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Route normal turns through one model-owned agent loop from the first action onward. | Static and test evidence show the pre-loop initial action path is gone. | `select_initial_action`, `PromptExecutionPlan`, and pre-loop bootstrap routing are removed or compatibility-only and no longer own turn action selection. |
| GOAL-02 | Preserve safety and operator intent while removing the duplicate lane. | Read-only, review, edit, and commit turns still obey execution governance and output contracts. | Focused tests cover turn policy inside the loop and full library tests pass. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | A human using Paddles to inspect and change the repository through the recursive harness. | Simple requests should enter the agent loop directly and terminate once satisfied. |
| Runtime Maintainer | A developer evolving the turn loop, action schema, and execution governance. | One coherent place to reason about action selection, evidence, mutation boundaries, and stopping. |

## Scope

### In Scope

- [SCOPE-01] Replace the pre-loop `select_initial_action` route with the first iteration of `execute_agent_loop`.
- [SCOPE-02] Move turn mode, mutation posture, output contract, edit pressure, commit pressure, and grounding pressure into the agent-loop request/execution contract.
- [SCOPE-03] Delete or collapse `PromptExecutionPlan`, `PromptExecutionPath`, and controller bootstraps that force first actions before the loop.
- [SCOPE-04] Update runtime presentation, traces, docs, and tests so they describe one agent-loop action-selection path.

### Out of Scope

- [SCOPE-05] Replacing execution governance or permission enforcement.
- [SCOPE-06] Changing provider wire schemas unless required by the internal migration.
- [SCOPE-07] Reworking final response rendering beyond inputs needed by the unified loop.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Every normal turn must enter `execute_agent_loop` before the first model-selected action is executed or accepted. | GOAL-01 | must | The loop must own first-action reasoning rather than receiving a controller-selected seed. |
| FR-02 | The action-selection engine interface must expose one loop action API for turn execution; any initial-action compatibility must not route runtime turns outside the loop. | GOAL-01 | must | A separate first-action API recreates the old planner lane. |
| FR-03 | Turn policy must be represented as a loop input and execution-contract constraint, not as a pre-loop router. | GOAL-01, GOAL-02 | must | Mutation and output posture remain important, but they should not decide the turn before loop reasoning. |
| FR-04 | Edit, commit, grounding, and review pressure must be modeled as loop state or instruction frame data consumed by the first loop iteration. | GOAL-01, GOAL-02 | must | Existing safeguards should become transparent loop context instead of hidden controller overrides. |
| FR-05 | Direct-answer and stop decisions must be selected through the agent loop and rendered without invoking a separate synthesizer-only route for normal turns. | GOAL-01 | should | Simple questions should be satisfied by the same action-selection protocol used for tool use. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain execution governance fail-closed behavior for mutating workspace and external actions. | GOAL-02 | must | The migration removes duplicate routing, not safety checks. |
| NFR-02 | Preserve traceability for why an action was selected, blocked, retried, or stopped. | GOAL-01, GOAL-02 | must | Operators need evidence when the loop chooses a bounded action or stops. |
| NFR-03 | Keep provider compatibility explicit and isolated when old schema names must remain on the wire. | GOAL-01 | should | Internal architecture should be coherent without causing unnecessary provider churn. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| First-action ownership | Focused tests and static search | Tests prove normal turns start action selection inside `execute_agent_loop`; `rg` evidence shows removed pre-loop router symbols. |
| Policy preservation | Focused read-only/review/edit/commit tests | Planning/review still block mutation, execution still allows governed mutation, commit/edit obligations remain enforced. |
| Regression coverage | Full Rust suite | `cargo test --lib` and any affected integration checks pass. |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing final renderer can consume loop outcomes once direct-answer routing is normalized. | Additional rendering changes may be needed. | Spike the first story against direct answer and evidence outcomes. |
| Existing HTTP provider schema names may need compatibility shims during the migration. | Removing them abruptly could break configured providers. | Keep compatibility isolated and document any retained wire names. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which legacy symbols must remain for provider compatibility after runtime code is renamed? | Implementer | Open until static cleanup story. |
| Direct-answer behavior could change if final rendering expects the old `PromptExecutionPlan` shape. | Implementer | Mitigate with focused direct-answer tests before implementation. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Normal turns no longer call a model-selected `select_initial_action` before `execute_agent_loop`.
- [ ] Turn mode and mutation/output policy are passed into and enforced inside the agent loop/execution contract.
- [ ] Pre-loop controller bootstraps are removed, collapsed into loop state, or proven compatibility-only.
- [ ] Focused and full library tests pass.
<!-- END SUCCESS_CRITERIA -->
