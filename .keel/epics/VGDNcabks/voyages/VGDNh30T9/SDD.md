# Integrate Resolver Into Edit Convergence - Software Design Description

> Thread deterministic entity resolution through the planner loop so edit-oriented turns validate targets, converge sooner, and explain misses instead of hallucinating paths or stalling in repeated search/read steps.

**SRS:** [SRS.md](SRS.md)

## Overview

Thread voyage-one resolver outcomes into the planner/controller seams that currently rely on likely-target heuristics and repeated reads. The planner stays model-directed, but the controller gains a deterministic file-target validation step before it spends more edit budget.

## Context & Boundaries

This voyage integrates the resolver into planning and operator visibility. It does not broaden the resolver’s query semantics beyond voyage one and does not add IDE or LSP dependencies.

```
┌──────────────────────────────────────────────┐
│ Planner Loop / Steering Gates               │
│        ↓                                    │
│ Deterministic Resolver Outcome              │
│   resolved | ambiguous | missing            │
│        ↓                                    │
│ Read / Diff / Edit Or Explicit Miss Path    │
└──────────────────────────────────────────────┘
        ↑                     ↑
  Trace / Runtime Events   Workspace Editor Boundary
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Voyage-one resolver | internal | Supplies deterministic candidate resolution | new |
| Planner loop state / steering reviews | internal | Existing convergence gates that need resolver input | existing |
| Runtime event projections | internal | Operator-visible explanation of resolver outcomes | existing |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Integration point | Consult resolver at known-edit bootstrap and execution-pressure review points | These are the places where hallucinated file targeting currently burns budget |
| Miss handling | Emit explicit miss/ambiguity artifacts and keep edit gating fail-closed | Prevents placeholder patches against unverifiable files |
| Visualization scope | Surface resolver outcomes in existing trace/runtime channels before inventing a separate panel | Keeps behavior legible without a second visualization system |

## Architecture

The controller should stop treating candidate-file selection as a soft hint once deterministic resolution is available. Resolved candidates become the preferred next read/edit targets; ambiguity and miss outcomes become first-class planner notes and runtime artifacts.

## Components

- Known-edit bootstrap integration: resolve user/planner target hints before the first file read.
- Action-bias / convergence review integration: replace repeated-search pressure with resolver-backed target validation.
- Runtime artifact projection: expose resolver outcomes to TUI/web/trace consumers.
- Fail-closed editor guardrails: preserve authored-file safety when resolution is absent or ambiguous.

## Interfaces

Planner/controller inputs should carry the candidate hints that need resolution. Resolver outputs should be injectable as:
- planner notes
- target-file substitutions
- fallback reasons
- operator-visible event summaries

## Data Flow

1. Turn enters edit-oriented planner path.
2. Controller detects likely target family or repeated drift.
3. Resolver is asked to validate or refine the target hint.
4. Resolved target flows into read/diff/edit selection.
5. Ambiguous/missing outcomes surface in notes and runtime events and stop unsafe mutation.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Resolver returns no safe candidate | miss outcome | emit deterministic miss and keep edit path fail-closed | planner may ask one narrower follow-up or stop with explanation |
| Resolver returns multiple plausible authored files | ambiguity outcome | surface candidates, avoid mutation | planner narrows using bounded read/search |
| Controller bypasses resolver during edit pressure | regression test catches missing integration | reject drift in CI | restore resolver invocation at the gate |
