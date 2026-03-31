---
id: VFUNJz9zT
---

# Adaptive Interpretation Context Refinement — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Directly addresses planner budget waste and stale guidance — the #1 user-visible quality gap |
| Confidence | 4 | InterpretationContext is Clone, integration seams are clean, per-request architecture already supports replacement |
| Effort | 3 | Core refinement loop is straightforward; thread-aware refinement and background async add complexity |
| Risk | 2 | Refinement latency could slow the loop; oscillation is possible but mitigable with cooldown |

## Findings

1. **InterpretationContext is immutable per request but replaceable between requests** [SRC-01]. The loop clones `context.interpretation` into each `PlannerRequest` at line 1346. Replacing the interpretation on the `PlannerLoopContext` between iterations is a zero-disruption change.

2. **Evidence and step history are fully visible at each refinement point** [SRC-01]. `loop_state.steps` and `loop_state.evidence_items` are cloned into each request, providing the model complete context for refinement decisions.

3. **Thread decisions already carry interpretation but don't feed changes back** [SRC-03]. `ThreadDecisionRequest` at line 1233 receives the same frozen interpretation. After a thread branch/merge, the interpretation should reflect the narrowed or reconciled scope.

4. **ContextPressure is emitted but never acted on** [SRC-02]. The existing pressure infrastructure provides a natural trigger — when pressure reaches High or Critical, the system should renegotiate constraints rather than just reporting.

5. **The planner budget doubled from 6 to 12 steps partially compensates for stale context** [SRC-04]. Refinement would let the planner work effectively within a smaller budget by keeping guidance current.

## Opportunity Cost

Pursuing this means delaying other planner improvements (multi-file edit orchestration, concurrent branch execution). However, refinement directly improves planner efficiency which compounds across all other planner capabilities.

## Dependencies

- The `RecursivePlanner` trait already has `derive_interpretation_context` [SRC-01] — a lightweight variant for delta refinement would reuse this infrastructure
- Transit thread replay [SRC-03] provides the thread transition signal but doesn't currently feed back into the interpretation pipeline

## Alternatives Considered

1. **Larger budgets only** — compensates for stale context but wastes time and API calls on redundant investigation [SRC-04]. Does not scale.
2. **Pre-computed interpretation variants** — derive multiple interpretation contexts upfront for different scenarios (code edit, search, casual) [SRC-01]. Cheaper per-turn but inflexible and doesn't adapt to discovered evidence.
3. **Full re-derivation at trigger points** — simpler than delta refinement but slower (full model call each time) [SRC-01]. Viable as a first implementation step before optimizing to deltas.

## Recommendation

[x] Proceed — convert to mission with phased delivery [SRC-01] [SRC-02] [SRC-03] [SRC-04] [SRC-05]

**Phase 1:** Evidence-threshold refinement. After every 3 planner steps, check if the accumulated evidence warrants an interpretation update. Use full re-derivation initially (reuse `derive_interpretation_context`). No new model call needed — reuse the existing planner engine.

**Phase 2:** Pressure-triggered constraint adjustment. When ContextPressure reaches High, trigger a refinement that can adjust budget allocation and retrieval strategy.

**Phase 3:** Thread-aware refinement. After thread branch/merge decisions, re-derive interpretation scoped to the new thread context.

**Phase 4:** Background async refinement. Run refinement concurrently with the planner's current step so it doesn't add latency to the critical path.
