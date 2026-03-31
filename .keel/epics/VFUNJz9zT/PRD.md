# Adaptive Interpretation Context Refinement - Product Requirements

> Interpretation context assembled once at turn start becomes stale as the planner accumulates evidence. A system that periodically re-evaluates what guidance, constraints, and capabilities are relevant — based on conversation state, evidence gathered, and thread lineage — would produce better-grounded planner decisions and fewer wasted steps.

## Problem Statement

Today, `derive_interpretation_context` runs once per turn. It loads AGENTS.md, derives a guidance subgraph, extracts tool hints and decision procedures, and freezes. The planner then works within that static context for all its steps.

This creates three problems:

1. **Stale relevance.** The guidance derived from the user prompt at turn start may not match what the planner discovers mid-investigation. A search for CI configuration doesn't need the same guidance as a code edit.

2. **No constraint negotiation.** The system never asks "given what I've learned, should my constraints or capabilities change?" The planner budget, tool access, and retrieval strategy are fixed regardless of what evidence reveals.

3. **No cross-turn learning.** In an interactive session, each turn re-derives context from scratch. Thread history, prior evidence, and conversation patterns don't inform what guidance is promoted or demoted.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Evidence-threshold refinement: re-derive interpretation every N steps when evidence warrants it | Fewer redundant planner steps on edit-class prompts | Measured via step count comparison |
| GOAL-02 | Pressure-triggered constraint adjustment: High pressure triggers budget/strategy renegotiation | ContextPressure events produce actionable constraint changes | Manual verification |
| GOAL-03 | Thread-aware refinement: re-derive interpretation scoped to new thread after branch/merge | Thread transitions produce narrowed/reconciled guidance | Manual verification |
| GOAL-04 | Background async refinement: run re-evaluation concurrently with planner steps | No added latency on the planner critical path | Timing comparison |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- [SCOPE-01] Define refinement triggers, policy metadata, and trigger evaluation signals in the planner loop.
- [SCOPE-02] Implement mid-loop interpretation context refinement when trigger conditions indicate context drift.
- [SCOPE-03] Emit structured `RefinementApplied` turn events for trace replay and diagnostics.
- [SCOPE-04] Enforce cooldown and oscillation-prevention guardrails around refinement execution.
- [SCOPE-05] Update validation artifacts (implementation notes, tests, and proofs) for first-wave refinement behavior.

### Out of Scope

- [SCOPE-06] Full async background refinement architecture or full planner throughput redesign.
- [SCOPE-07] Full planner re-architecture, new evidence model, or unrelated CLI workflow changes.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Add explicit refinement triggers and policy metadata for planner-loop checkpoint-based interpretation updates. | GOAL-01 | must | Establishes the control points used to start refinement. |
| FR-02 | Implement mid-loop interpretation refinement and context re-derivation behavior when trigger conditions are met. | GOAL-02 | must | Prevents stale interpretation from driving redundant or misdirected planning steps. |
| FR-03 | Emit `RefinementApplied` turn events with trigger and context delta details into the trace stream. | GOAL-03 | must | Makes refinement behavior observable for replay, diagnostics, and operator trace review. |
| FR-04 | Add cooldown and oscillation prevention checks so refinement cannot happen repeatedly in short bursts during active turns. | GOAL-04 | should | Prevents context churn and unstable loop behavior during pressure spikes. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Guarantee safe failure semantics: if trigger data is incomplete or policy checks fail, the planner continues without refinement and emits warning telemetry. | GOAL-01, GOAL-04 | must | Keeps turn execution stable and deterministic under edge conditions. |
| NFR-02 | Keep refinement decision and event emission on the critical path only when bounded and traceable. | GOAL-01, GOAL-02, GOAL-04 | should | Preserves responsiveness while improving adaptation quality. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What is the right granularity for the negotiation response schema? Full re-derivation vs. delta patches? | Planner | Open |
| Should refinement be synchronous (blocking the next planner step) or asynchronous (background)? | Planner | Open |
| How do we prevent refinement oscillation — context changing back and forth between steps? | Planner | Open |
| Should the first interpretation still be model-derived, or should it start with a cheaper heuristic and refine up? | Planner | Open |
| What transit events should serve as thread transition triggers? Only explicit decisions, or also implicit checkpoint boundaries? | Planner | Open |
| How does this interact with compaction? Should refinement and compaction share the same trigger points? | Planner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Planner makes fewer redundant investigation steps when context is refined mid-turn
- [ ] Thread transitions produce context that reflects the narrowed or broadened scope
- [ ] Context pressure signals trigger meaningful constraint adjustments (not just informational)
- [ ] Refinement overhead stays under 2 seconds per trigger (amortized across the turn)
- [ ] No regression in turn latency for simple direct-response turns
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

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

## Research Provenance

*Source records from bearing evidence:*

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | direct:codebase | src/application/mod.rs:1291-1318 | 2026-03-31 | 2026-03-31 | high | high | derive_interpretation_context runs once per turn and freezes |
| SRC-02 | manual | direct:codebase | src/domain/model/context_quality.rs | 2026-03-30 | 2026-03-31 | high | high | ContextPressure signals exist but don't trigger re-evaluation |
| SRC-03 | manual | direct:codebase | src/domain/model/threading.rs | 2026-03-29 | 2026-03-31 | high | high | Thread decisions create/merge branches but don't feed back into interpretation |
| SRC-04 | manual | direct:user-session | paddles interactive session | 2026-03-31 | 2026-03-31 | high | high | Planner exhausted 6-step budget on redundant investigation because frozen context didn't adapt |
| SRC-05 | manual | direct:codebase | .keel/missions/VFNzln1hr/TOPOLOGY.md | 2026-03-30 | 2026-03-31 | high | high | Context tier model with transit-native addressing enables lazy resolution of stale context |

---

*This PRD was seeded from bearing `VFUNJz9zT`. See `bearings/VFUNJz9zT/` for original research.*
