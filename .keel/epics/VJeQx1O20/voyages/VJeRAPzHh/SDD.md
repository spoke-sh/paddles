# Move Turn Contract Into Agent Loop - Software Design Description

> Read-only, execution, review, edit, commit, and grounding policy are loop inputs and execution-contract constraints rather than pre-loop routing decisions.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage keeps the valid safety and output constraints that `context.collaboration` currently carries, but turns them into an explicit loop input rather than a collaboration subsystem or pre-loop router. The contract should be visible to the action selector, execution contract, and final renderer without giving `turn.rs` authority to preselect the first action.

## Context & Boundaries

The voyage changes internal naming and data flow around turn policy. It should not broaden permissions. It should not create a new mode system. Existing planning, execution, and review semantics remain, but they should be expressed as a turn contract that the loop and governance enforce.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| turn contract model | domain model | Carries mode, mutation posture, output contract, and clarification policy. | internal |
| execution contract service | application service | Renders capabilities and completion requirements for action selection. | internal |
| execution governance | infrastructure/application | Enforces command and mutation policy. | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Naming | Prefer `turn_contract` or `turn_policy` over `collaboration` in loop code. | The field is not human collaboration state; it is runtime policy. |
| Enforcement location | Enforce after model selection and before execution inside the loop. | The model still owns action choice, while the harness keeps hard safety boundaries. |
| Obligations | Represent edit/commit/grounding as instruction frame or loop state. | Obligations should guide the model inside the loop instead of forcing a hidden first action. |

## Architecture

The domain model should expose a compact contract/result type. `turn.rs` resolves the requested/default contract once and passes it into `AgentLoopContext`. `agent_loop.rs` attaches it to every action-selection request and uses the execution contract service to render blocked capabilities and completion requirements. Hard mutation boundaries remain in the loop before workspace or external execution.

## Components

- Turn contract model: mode, mutation posture, output contract, clarification policy, request status, and detail.
- Execution contract service: turns contract state into capability and completion lines.
- Agent loop boundary check: blocks forbidden mutating actions with clear stop/clarification behavior.
- Runtime presentation: keeps user-facing mode disclosure concise.

## Interfaces

The internal `AgentLoopContext` should carry the renamed turn contract. Adapters may keep serialized collaboration labels only when an external surface depends on them; such use must be documented as compatibility.

## Data Flow

1. A mode request is resolved to a turn contract at turn start.
2. The contract is attached to agent-loop context.
3. Each loop action-selection request receives the contract in runtime notes and execution contract lines.
4. The selected action is checked against the contract before execution.
5. The final renderer receives the same contract for output shape.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Model selects mutation in read-only mode | loop boundary check | Stop with structured clarification or read-only explanation. | Re-run in execution mode if mutation is desired. |
| Contract naming leaks old architecture | static search | Keep only documented compatibility occurrences. | Rename internal symbols and tests. |
