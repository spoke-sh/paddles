# Model-Directed Routing Backbone - Software Design Description

> Replace heuristic top-level turn routing with model-selected bounded actions driven by AGENTS-informed interpretation context, while keeping controller safety, observability, and grounded synthesis.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage replaces the current heuristic top-level gate with a model-directed
next-action contract.

The intended flow is:

1. assemble interpretation context from operator memory, linked docs, recent
   turns, and relevant local state,
2. ask a planner-capable model to choose the next bounded action from a
   constrained schema,
3. validate and execute that action under controller budgets and allowlists,
4. repeat through the recursive loop when resource actions are chosen,
5. hand the resulting trace and evidence to the synthesizer when the model
   chooses a terminal answer or synthesis path.

## Context & Boundaries

- In scope:
  - model-directed first action selection
  - interpretation-before-routing
  - controller validation and safe execution
  - recursive loop/synthesis integration
  - foundational documentation updates
- Out of scope:
  - Keel-specific runtime intents
  - unbounded free-form autonomy
  - mandatory remote planners
  - unrelated UI or boot-path work

```
┌──────────────────────────────────────────────────────────────┐
│                         This Voyage                          │
│                                                              │
│  User Turn                                                   │
│     ↓                                                        │
│  Interpretation Context                                      │
│  (AGENTS + linked docs + turns + local state)                │
│     ↓                                                        │
│  Action Selection Model                                      │
│     ↓ choose one bounded action                              │
│  answer | search | read | inspect | refine | branch | stop   │
│     ↓                                                        │
│  Controller Validation + Execution                           │
│     ↓                                                        │
│  Recursive Loop State / Evidence / Trace                     │
│     ↓                                                        │
│  Synthesizer Handoff                                         │
└──────────────────────────────────────────────────────────────┘
          ↑                                   ↑
      Workspace tools                   Planner/synth models
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | library | Local search, autonomous search, and workspace retrieval used by planner actions | current pinned Cargo dependency |
| local Qwen runtime | internal runtime | Planner-capable and synthesizer-capable local model lanes | current candle-backed runtime |
| operator memory loader | internal runtime | Loads `AGENTS.md` and linked foundational docs for interpretation context | existing repo implementation |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Top-level routing is model-directed | Replace string heuristics with constrained next-action selection | Keeps routing grounded in interpretation context |
| Controller remains bounded and authoritative | Validate schema, budgets, allowlists, and failures in controller code | Preserves safety without stealing routing ownership back |
| Interpretation context comes first | Build memory/doc context before first action choice | Makes `AGENTS.md` operational rather than decorative |
| Keel remains generic context | Do not introduce board-specific runtime intents | Prevents overfitting the harness to one tool ecosystem |

## Architecture

The voyage touches four cooperating layers:

1. `InterpretationContextAssembler`
   Builds the context envelope used before first action selection.

2. `TopLevelActionSelector`
   A planner-capable model that chooses the next bounded action from a
   constrained schema.

3. `ControllerValidatorExecutor`
   Validates the chosen action, executes safe resource/tool work, and maintains
   loop budgets and event visibility.

4. `RecursivePlannerAndSynthHandoff`
   Reuses the validated action path inside the recursive loop and hands the
   final evidence trace to synthesis.

## Components

- `InterpretationContextAssembler`
  Purpose: merge operator memory, linked docs, recent turns, and local state
  before the first action is chosen.

- `TopLevelActionSelector`
  Purpose: let the model choose the next bounded action instead of a heuristic
  controller branch.

- `ControllerValidatorExecutor`
  Purpose: enforce schema validity, budgets, read-only inspect constraints, and
  fail-closed behavior.

- `RecursivePlannerAndSynthHandoff`
  Purpose: ensure the selected action path feeds the recursive loop and the
  final evidence handoff cleanly.

## Interfaces

- `InterpretationContext`
  Planner-visible prompt context used before first action selection.

- `TopLevelActionDecision`
  Constrained JSON action emitted by the model for the next bounded step.

- `ValidatedAction`
  Controller-approved action ready for execution or synthesis handoff.

- `SynthesisInput`
  Evidence and trace bundle passed to the synthesizer after terminal routing.

## Data Flow

1. Receive the user turn.
2. Assemble interpretation context from memory, docs, turns, and local state.
3. Ask the model to choose one bounded next action.
4. Validate the action.
5. Execute safe resource actions or hand terminal decisions to synthesis.
6. Feed resulting outputs back into recursive loop state when additional work is
   needed.
7. Render action trace and final answer.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Model emits invalid top-level action JSON | Schema/parser validation fails | Record fallback event and retry or fail closed through bounded fallback behavior | Preserve explicit insufficiency rather than guessing |
| Model chooses unsafe inspect command | Allowlist or single-step validation rejects it | Refuse execution and request another bounded action or stop | Keep controller safety authoritative |
| Interpretation context is missing or weak | Memory/doc assembly yields low-signal context | Continue with minimal context but emit visible event/fallback | Encourage narrower follow-up or richer memory/docs |
| Recursive loop cannot produce grounded evidence | Evidence budget exhausted or stop without useful evidence | Hand partial evidence to synthesizer with insufficiency signal | Return extractive evidence or explicit uncertainty |
