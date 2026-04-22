# Single Recursive Control Plane - Software Design Description

> Collapse nested adapter tool loops into the application harness and route workspace actions through explicit execution boundaries rather than synthesis ports.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the recursive harness the only owner of repository action
execution. The application layer receives planner-selected workspace actions,
routes them through an explicit executor boundary, records governance and
evidence, and only then hands any gathered evidence to response-authoring
lanes. Model adapters stop owning their own repository tool loops and become
authoring/planning integrations rather than alternate control planes.

## Context & Boundaries

### In Scope

- workspace action executor boundary owned by the application layer
- synthesizer port cleanup so authoring and mutation are separate concerns
- removal of adapter-owned repository tool loops
- single-source budgeting and governance for recursive execution

### Out of Scope

- projection/read-model ownership cleanup beyond what loop extraction needs
- new provider features unrelated to removing nested execution loops

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│ planner decision -> application loop -> workspace executor         │
│                                  -> governance/evidence            │
│                                  -> synthesis handoff              │
│                                                                    │
│ adapters: plan / gather / author only                              │
└────────────────────────────────────────────────────────────────────┘
          ↑                                         ↑
      planner lane                            synthesizer lane
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `RecursivePlanner` | internal | Selects bounded next actions | current |
| `WorkspaceEditor` and execution-hand infrastructure | internal | Executes repository actions under governance | current |
| `SynthesizerEngine` authoring lane | internal | Authors final user-facing responses from evidence | current |
| Execution-governance and trace recording surfaces | internal | Preserve safety and visibility while ownership changes | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Executor ownership | Application layer owns workspace action execution | Keeps control semantics in one place |
| Synthesizer contract | Response authoring only | Avoids mixing mutation with narration |
| Adapter behavior | No repository tool loops in adapters | Prevents duplicate budgets and retries |
| Visibility | Preserve governance/evidence emission in the application loop | Operators still need one observable control plane |

## Architecture

1. The recursive planner selects a bounded action.
2. The application loop validates the action and hands it to a dedicated
   executor boundary.
3. The executor returns summaries, edits, and governance outcomes.
4. The application loop records evidence and events, updates budgets, and
   decides whether to continue.
5. The synthesizer lane receives only the resulting evidence/handoff and
   authors the final response.

## Components

`WorkspaceActionExecutor`
: Application-owned boundary that executes validated repository actions and
returns summaries, applied edits, and governance results.

`RecursiveLoopCoordinator`
: The single loop owner for budgeting, retries, stops, evidence, and executor
invocation.

`SynthesizerAuthoringPort`
: Response-authoring-only port that consumes gathered evidence and handoff
context.

`AdapterPlannerAndAuthoringClients`
: Provider integrations that supply planning or authoring behavior but no
independent repository execution loop.

## Interfaces

Candidate internal interfaces:

- `execute_workspace_action(action, frame) -> WorkspaceActionResult`
- `respond_for_turn(prompt, intent, evidence, handoff, sink) -> AuthoredResponse`
- `select_next_action(request, sink) -> RecursivePlannerDecision`

Candidate removals:

- `SynthesizerEngine::execute_workspace_action(...)`
- adapter-local repository tool-loop entry points used as alternate control
  planes

## Data Flow

1. Planner selects a workspace action.
2. Application loop validates the action against budgets and collaboration
   posture.
3. Executor boundary performs the action and returns governance plus evidence.
4. Application loop records emitted events, updates evidence/budget state, and
   decides whether to continue or stop.
5. Final response authoring happens after the loop, not inside an adapter-owned
   repository tool loop.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| A provider adapter still attempts to own repository tool execution | Contract tests or compile failures against the trimmed synthesizer interface | Block the slice and route execution back through the application executor boundary | Remove or wrap the stray adapter entry point |
| Governance/evidence events disappear during loop extraction | Turn-loop tests or manual trace inspection fail | Treat as regression and keep the old emission path until parity is restored | Reattach emission at the application coordinator boundary |
| Budget/retry logic remains duplicated | Architectural review or tests show two active owners | Reject the intermediate state | Finish removing the duplicate loop before shipping |
