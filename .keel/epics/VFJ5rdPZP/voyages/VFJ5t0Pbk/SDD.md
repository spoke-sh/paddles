# Model-Judged Interpretation And Retrieval - Software Design Description

> Replace residual reasoning heuristics with model-judged interpretation, fallback, and retrieval selection while preserving controller safety constraints.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the remaining reasoning-heavy heuristics from the primary
`paddles` harness path without weakening controller-owned constraints.

The intended flow is:

1. load only `AGENTS.md` roots as operator memory,
2. ask the planner model to derive the relevant guidance subgraph,
3. ask the planner model to derive the interpretation context from that graph,
4. ask the planner model for the next bounded action,
5. if the reply is invalid, prefer another constrained model re-decision rather
   than lexical/controller reasoning,
6. keep controller validation, budgets, allowlists, and deterministic execution
   unchanged,
7. synthesize from the resulting evidence with the controller still surfacing
   the boundary between model judgement and controller safety.

## Context & Boundaries

- In scope:
  - legacy routing/tool-inference heuristic removal
  - model-judged interpretation relevance and decision-framework selection
  - constrained model re-decision for invalid planner replies
  - retrieval/evidence heuristics that currently stand in for reasoning
  - docs and proof artifacts
- Out of scope:
  - removing controller safety rails
  - remote-only planners
  - configurable memory roots
  - a full cross-tool resource graph rewrite

```
┌──────────────────────────────────────────────────────────────────────┐
│                              This Voyage                            │
│                                                                      │
│  AGENTS.md roots                                                     │
│      ↓                                                               │
│  model-derived guidance graph                                        │
│      ↓                                                               │
│  model-derived interpretation context                                │
│      ↓                                                               │
│  model-selected initial / next action                                │
│      ↓ invalid?                                                      │
│  constrained re-decision prompt                                      │
│      ↓                                                               │
│  controller validation / budgets / execution                         │
│      ↓                                                               │
│  recursive evidence + synthesis handoff                              │
└──────────────────────────────────────────────────────────────────────┘
          ↑                                           ↑
   controller-owned safety rails              operator-visible traces
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| planner lane model | local runtime | Derives guidance graph, interpretation context, and bounded next actions | current local planner model |
| `AGENTS.md` roots | project memory | Provide the only hardcoded interpretation roots | repo-local files |
| controller runtime | internal runtime | Preserves validation, budgets, allowlists, deterministic execution, and fail-closed behavior | current Rust runtime |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Keep `AGENTS.md` as the only hardcoded memory root | Additional guidance must be model-derived from references | Avoids encoding a project-specific foundational file set into the controller |
| Replace lexical reasoning with constrained model passes | Use structure and schema, not prose heuristics | Keeps reasoning with the model while preserving determinism at the controller boundary |
| Preserve controller safety rails | Budgets, allowlists, path validation, and fail-closed behavior remain in Rust | These are constraints, not reasoning heuristics |
| Treat retrieval heuristics case-by-case | Remove reasoning-heavy ranking/query heuristics but keep controller constraints | Not every heuristic is a bug; some are safety or resource limits |

## Architecture

The voyage touches five cooperating layers:

1. `OperatorMemoryRoots`
   Loads only `AGENTS.md` roots from system/user/workspace scopes.

2. `GuidanceGraphDeriver`
   Uses a constrained planner prompt to choose the next referenced guidance
   documents to load from those roots.

3. `InterpretationAssembler`
   Builds turn-time interpretation context from the loaded graph and removes
   lexical relevance ranking where model judgement should decide.

4. `ConstrainedReDecision`
   Replaces heuristic initial/planner fallback with additional bounded model
   re-decision passes before fail-closed controller fallback.

5. `RetrievalJudgementBoundary`
   Moves retrieval-selection and evidence-prioritization decisions that reflect
   reasoning toward model judgement while leaving controller safety logic in
   place.

## Components

- `OperatorMemoryRoots`
  Purpose: load only `AGENTS.md` roots and stop statically crawling named
  foundational docs.

- `GuidanceGraphDeriver`
  Purpose: derive a prompt-relevant referenced subgraph from loaded operator
  memory using the planner model.

- `InterpretationAssembler`
  Purpose: build excerpts, tool hints, and decision procedures from the
  model-selected guidance graph instead of lexical scoring.

- `ConstrainedReDecision`
  Purpose: retry invalid initial/planner decisions through constrained prompts
  before any residual fail-closed fallback.

- `RetrievalJudgementBoundary`
  Purpose: remove hardcoded reasoning-heavy retrieval selection/ranking where
  the model should choose the next evidence move.

## Interfaces

- `RecursivePlanner::derive_interpretation_context`
  Produces interpretation context from `AGENTS.md` roots and a model-derived
  guidance graph.

- initial/planner action prompts
  Continue to use bounded JSON schemas, but fallback should prefer another
  constrained model pass instead of lexical/controller reasoning.

- gatherer/retrieval config
  Continues to expose controller-owned budgets and modes, while moving
  reasoning-heavy selection/prioritization toward model judgement.

## Data Flow

1. Load `AGENTS.md` roots.
2. Ask the planner model which referenced documents should be loaded next.
3. Load the selected guidance subgraph.
4. Build interpretation context from that graph.
5. Ask the planner model for the bounded next action.
6. If invalid, run a constrained re-decision pass instead of lexical fallback.
7. Validate and execute the chosen action through controller-owned safety
   constraints.
8. Feed resulting evidence back into the recursive loop and final synthesis.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Model-derived guidance graph is invalid or empty | schema parse / missing files | fall back to AGENTS-rooted interpretation only | remain fail-closed and visible |
| Re-decision still returns invalid action JSON | schema parse | use minimal controller fallback rather than reasoning-heavy heuristic ranking | preserve bounded execution |
| Retrieval judgement degrades answer quality | tests/proofs show weaker evidence choice | keep or restore the smallest controller safeguard necessary | iterate in follow-up slice |
| Removing a heuristic accidentally weakens safety | controller tests fail or invalid command slips through | reject the change and keep the controller constraint in Rust | preserve local-first safety posture |
