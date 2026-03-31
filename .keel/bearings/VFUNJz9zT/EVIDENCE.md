---
id: VFUNJz9zT
---

# Adaptive Interpretation Context Refinement — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | direct:codebase | src/application/mod.rs:1291-1318 | 2026-03-31 | 2026-03-31 | high | high | derive_interpretation_context runs once per turn and freezes |
| SRC-02 | manual | direct:codebase | src/domain/model/context_quality.rs | 2026-03-30 | 2026-03-31 | high | high | ContextPressure signals exist but don't trigger re-evaluation |
| SRC-03 | manual | direct:codebase | src/domain/model/threading.rs | 2026-03-29 | 2026-03-31 | high | high | Thread decisions create/merge branches but don't feed back into interpretation |
| SRC-04 | manual | direct:user-session | paddles interactive session | 2026-03-31 | 2026-03-31 | high | high | Planner exhausted 6-step budget on redundant investigation because frozen context didn't adapt |
| SRC-05 | manual | direct:codebase | .keel/missions/VFNzln1hr/TOPOLOGY.md | 2026-03-30 | 2026-03-31 | high | high | Context tier model with transit-native addressing enables lazy resolution of stale context |

## Feasibility

Evidence strongly supports feasibility. The interpretation context is already a well-typed struct (`InterpretationContext`) with documents, tool hints, and a decision framework. The planner loop already has natural trigger points (after each step, after gatherer returns, at thread decisions). Transit trace lineage provides thread transition signals. The main risk is latency — each refinement requires a model call.

## Key Findings

1. Interpretation context is assembled once and frozen for the entire turn [SRC-01]. The planner operates on stale guidance when its investigation reveals the request is different from what the initial prompt suggested.
2. Context pressure signals are emitted but purely informational [SRC-02]. They could serve as refinement triggers — High/Critical pressure could trigger constraint renegotiation.
3. Thread decisions already produce structured state transitions [SRC-03] that could naturally trigger context re-derivation scoped to the new thread.
4. The planner budget increase from 6 to 12 steps partially masks the problem [SRC-04] — more budget compensates for stale context, but refinement would be more efficient.
5. The four-tier context model with lazy resolution [SRC-05] means refinement can reference deeper tiers without eagerly loading everything upfront.

## Unknowns

- What is the latency cost of a lightweight refinement call vs. a full re-derivation?
- Can refinement be made async (background) without creating consistency hazards between the planner's current step and the refined context?
- How do we prevent refinement oscillation where context flip-flops between competing guidance?
- Should the first turn in a session use a cheaper heuristic interpretation and refine up, or start with full model-derived context?
