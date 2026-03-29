# Recursive Planner Harness Backbone - Software Design Description

> Replace static turn heuristics with a model-owned bounded search/refine loop that interprets operator memory first, gathers recursive evidence, and hands structured trace plus evidence to a downstream synthesizer.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage reframes `paddles` as a recursive in-context planning harness.

Instead of making a coarse controller decision and asking one small model to do
the rest, the runtime should:

1. assemble interpretation context from operator memory and recent turns,
2. let a planner model choose the next bounded resource action,
3. execute that action and feed the result back into context,
4. repeat until enough evidence exists or a budget/stop condition is met,
5. hand the accumulated trace plus evidence to a separate synthesizer model.

## Context & Boundaries

- In scope:
  - operator-memory-aware interpretation
  - bounded planner action contracts
  - recursive resource execution
  - planner/synth model separation
  - foundational docs and diagrams
- Out of scope:
  - hardcoded Keel-specific intents
  - mandatory remote planners
  - replacing the existing TUI shell
  - unconstrained free-form autonomous execution

```
┌──────────────────────────────────────────────────────────────┐
│                         This Voyage                          │
│                                                              │
│  User Turn                                                   │
│     ↓                                                        │
│  Interpretation Context                                      │
│  (AGENTS + docs + recent turns + local state)                │
│     ↓                                                        │
│  Planner Model ── next bounded action ──┐                    │
│     ↑                                   │                    │
│     └──── trace + tool/search output ◄──┘                    │
│     ↓                                                        │
│  Evidence Bundle + Planner Trace                             │
│     ↓                                                        │
│  Synthesizer Model                                           │
│     ↓                                                        │
│  TUI / Plain Output                                          │
└──────────────────────────────────────────────────────────────┘
          ↑                                   ↑
     Workspace tools                    Planner/Synth models
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | library | Local search, autonomous search, controller-style multi-step retrieval primitives | current pinned Cargo dependency |
| local Qwen runtime | internal runtime | Planner and synthesizer inference lanes when local models are selected | existing candle-backed runtime |
| operator memory files | project/runtime context | `AGENTS.md` and linked foundational docs used for interpretation | current repo + system/user scopes |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Interpretation precedes routing | Build interpretation context before planner action selection | Fixes the current mismatch where memory influences prompts after the route is chosen |
| Planner and synthesizer are distinct roles | The planner gathers/refines; the synthesizer answers | Enables workload-specific model routing |
| The recursive loop is bounded | Use action, depth, and evidence budgets | Preserves safety and local-first control |
| Keel is evidence, not a turn type | Board artifacts enter through the same resource mechanisms as other workspace data | Keeps the harness general-purpose |

## Architecture

The backbone architecture has five cooperating layers:

1. `InterpretationContextAssembler`
   Collects `AGENTS.md`, linked foundational docs, recent turns, retained
   evidence, and prior tool outputs into planner-visible context.

2. `PlannerLane`
   Owns next-action selection for non-trivial turns. A planner-capable model
   chooses whether to search, read, inspect, refine, branch, or stop.

3. `RecursiveExecutionLoop`
   Executes validated planner actions, appends outputs back into context, and
   repeats until a stop condition or budget boundary is hit.

4. `SynthesisLane`
   Consumes the resulting evidence bundle plus planner trace and produces the
   user-facing answer.

5. `Renderer`
   Shows planner actions, tool outputs, stop reasons, and final synthesis
   through the existing TUI/plain output surfaces.

## Components

- `InterpretationContextAssembler`
  Purpose: produce the planner-visible context envelope.
  Behavior: merges operator memory, foundational links, recent turns, and prior
  loop state.

- `PlannerAction`
  Purpose: typed or validated expression of the next recursive step.
  Behavior: allows search/read/inspect/refine/branch/stop while remaining
  bounded and safe.

- `RecursivePlannerCoordinator`
  Purpose: own the loop budget, state evolution, and stop handling.
  Behavior: executes planner-selected actions until evidence is synthesis-ready
  or the loop ends.

- `PlannerTrace`
  Purpose: record actions, rationale, stop reason, and resulting evidence.
  Behavior: persists the reasoning-visible trail without pretending it is the
  final answer.

- `SynthesizerHandoff`
  Purpose: formalize what the answer model receives.
  Behavior: packages evidence bundle, planner trace, and answer contract.

## Interfaces

- `InterpretationContext`
  Carries operator memory, doc excerpts or references, recent turns, retained
  artifacts, tool outputs, and active budgets.

- `PlannerDecision`
  Bounded action proposal emitted by the planner model.

- `PlannerLoopState`
  The mutable state for recursive execution: trace, evidence, budgets, and
  retained context.

- `SynthesisInput`
  A distinct handoff object for final answer generation.

## Data Flow

1. Read the user turn.
2. Assemble interpretation context from `AGENTS.md`, linked docs, recent turns,
   and prior loop state.
3. Ask the planner lane for the next bounded action.
4. Validate and execute the action.
5. Append results into loop state and emit operator-visible events.
6. Repeat steps 3-5 until the planner stops or a budget is exhausted.
7. Convert loop state into synthesis input.
8. Ask the synthesizer lane for the final grounded answer.
9. Render the trace and final answer.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Planner emits invalid action | Schema/validator rejects decision | Record planner failure and request another bounded decision or stop | Fall back to explicit insufficient-evidence path if repeated |
| Recursive loop exceeds budget | Depth/action/evidence counters reach limits | Stop loop with explicit budget-exhausted reason | Hand partial evidence to synthesizer or admit insufficiency |
| Planner provider unavailable | Capability/config check fails | Route to local fallback planner or constrained current path | Preserve local-first operation |
| Synthesizer cannot ground answer | Reply is blank or unsupported by evidence | Fall back to extractive evidence summary | Ask for narrower follow-up next turn |
