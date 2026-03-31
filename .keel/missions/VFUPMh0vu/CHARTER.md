# Adaptive Interpretation Context Refinement - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Implement evidence-threshold refinement: re-derive interpretation context every N planner steps when accumulated evidence warrants it | board: VFUNJz9zT |
| MG-02 | Implement pressure-triggered constraint adjustment: when ContextPressure reaches High, trigger refinement that adjusts budget and retrieval strategy | board: VFUNJz9zT |
| MG-03 | Implement thread-aware refinement: re-derive interpretation context after thread branch/merge decisions scoped to the new thread | board: VFUNJz9zT |
| MG-04 | Implement background async refinement: run context re-evaluation concurrently with the planner's current step | board: VFUNJz9zT |

## Constraints

- Refinement must not add more than 2 seconds of latency to any planner step (amortized)
- Direct-response turns (no planner loop) must not regress in latency
- InterpretationContext replacement must happen between planner requests, never mid-request
- Refinement must use the existing `derive_interpretation_context` infrastructure initially, optimizing to delta refinement only after the full re-derivation path is proven
- No new model dependencies — refinement reuses the planner engine's interpretation derivation

## Halting Rules

- DO NOT halt while any MG goal has unfinished epics or voyages on the board
- HALT when all four phases (MG-01 through MG-04) have verified board evidence
- YIELD to human if refinement latency exceeds the 2-second constraint and requires architectural redesign
