# Recursive Loop Migration - Software Design Description

> Run the first model action as step zero of the same recursive agent loop while preserving direct answers, edit obligations, and fail-closed behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

Move from `initial routing -> recursive planner loop -> synthesis` to one
recursive agent loop. The first model-selected action becomes loop step zero.
`answer` and `stop` are terminal loop actions, not a bypass around the loop.

## Context & Boundaries

The runtime must preserve existing operator behavior while changing the internal
control shape. This voyage owns application-loop changes and focused runtime
tests; prompt vocabulary cleanup is handled separately.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Unified action contract voyage | Domain contract | Provides the target decision shape | VJXwlCA0P |
| `AgentRuntime` turn loop | Application service | Runtime path to migrate | current repo |
| Execution hand / workspace action executor | Application boundary | Executes loop actions | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First action handling | Treat as step zero in loop state | Makes the whole agent loop recursive. |
| Terminal answers | Model-selected `answer` and `stop` terminate the loop | Direct answers remain fast without becoming pre-loop routing. |
| Edit metadata | Carry on the decision envelope and loop context | Preserves known-edit and commit pressure. |

## Architecture

The target flow is:

```text
assemble context
loop:
  model selects AgentAction
  controller validates action, budget, and capability manifest
  if terminal: finish answer/block
  else: execute bounded action
  append observation/evidence
```

There should be no separate code path whose job is to decide whether the loop
exists. The first decision can terminate immediately, but it should still be a
loop decision.

## Components

| Component | Purpose |
|-----------|---------|
| Turn orchestration | Builds loop request and starts step-zero decision |
| Agent loop executor | Validates, executes, records, and terminates actions |
| Edit obligation context | Carries candidate files and applied-edit/commit obligations |
| Steering review layer | Continues to bias non-convergent edit/commit/review paths |

## Interfaces

Provider adapters should expose one action-selection method for agent loop
decisions. Temporary wrappers may exist only while migration stories remove
callers.

## Data Flow

1. Context assembly produces operator memory, capability manifest, and loop
   context.
2. Step zero calls the same action-selection contract as later steps.
3. Terminal actions finish through the normal response path.
4. Non-terminal actions execute and append observations to loop state.
5. Steering reviews continue to re-enter the same loop.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Invalid step-zero model reply | Parser/validation failure | Retry through the same action-selection recovery | Fail closed to terminal stop |
| Direct answer without required grounding | Repository-grounding guard | Force bounded workspace evidence action | Continue loop |
| Edit turn attempts advice-only completion | Instruction frame review | Reject terminal answer/stop | Re-enter loop with edit pressure |
