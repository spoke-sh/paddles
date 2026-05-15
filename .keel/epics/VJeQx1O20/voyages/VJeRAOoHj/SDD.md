# Unify First Action Entry Point - Software Design Description

> Normal turn execution enters execute_agent_loop before any model-selected action; the first loop iteration handles direct answers, stops, and workspace actions.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the model-selected first-action lane from turn orchestration. `turn.rs` should assemble prompt, interpretation, recent-turn, operator-memory, and runtime-contract inputs, then invoke `execute_agent_loop` with no preselected action. The agent loop owns sequence zero and can choose a direct answer, stop, search, read, inspect, shell, edit, commit, or external capability action through the same action-selection API used for later steps.

## Context & Boundaries

The voyage is limited to the turn entry point and the first loop iteration. It does not change the workspace executor, execution hand governance, or provider transport beyond interface changes needed to stop calling `select_initial_action` from normal turn execution.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `ActionSelectionEngine` | domain port | Provides model-owned action selection for each loop step. | internal |
| `execute_agent_loop` | application phase | Owns recursive action selection, evidence accumulation, and stop outcomes. | internal |
| final renderer | application port | Renders loop direct-answer/evidence outcomes. | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First action ownership | The agent loop selects sequence zero. | A separate initial selector recreates the planner lane and can get in the way before the loop observes evidence. |
| Direct answer handling | Represent direct answers as loop stop outcomes. | Simple requests should satisfy through the same loop contract as tool-using turns. |
| Fallback behavior | Action-selection unavailability remains a fallback, but it must be explicit and not a hidden second route for normal turns. | The operator needs visible degradation when model action selection is unavailable. |

## Architecture

`turn.rs` should become an input assembly and finalization boundary. It prepares interpretation, runtime notes, operator memory, recent turns, instruction frame, and governance surfaces, then delegates to `agent_loop.rs`. `agent_loop.rs` initializes sequence zero, builds the `PlannerRequest`/successor request with the current loop state, and calls the action selector through one loop action API.

## Components

- `turn.rs`: assemble loop context, emit turn-level trace events, call the loop, and render/finalize the loop outcome.
- `agent_loop.rs`: own all model-selected turn actions starting at sequence zero.
- `ActionSelectionEngine`: expose the loop action API used for both first and subsequent steps.
- Tests: cover direct answer, workspace action, and stop outcomes from the first loop iteration.

## Interfaces

The desired internal interface is one loop action method. If provider wire contracts still require initial-action names, adapters can translate internally, but normal turn execution should not depend on a separate initial-action trait method.

## Data Flow

1. Turn orchestration derives interpretation and runtime context.
2. Turn orchestration calls `execute_agent_loop` without an initial model decision.
3. The loop builds the first action-selection request with empty/new loop state.
4. The action selector chooses the first bounded action.
5. The loop executes or stops, records evidence, and repeats as needed.
6. Turn orchestration renders the loop outcome.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Action selector unavailable | `PlannerCapability::Unsupported` or provider error | Emit explicit fallback and return a governed direct response path only for degraded operation. | Restore action-selection provider or fix configuration. |
| First loop action cannot be rendered | focused test failure | Keep direct-answer/stop data on `AgentLoopOutcome`. | Update final renderer handoff. |
