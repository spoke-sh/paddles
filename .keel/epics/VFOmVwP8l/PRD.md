# Context Pressure And Relevance Signals - Product Requirements

## Problem Statement

The paddles capabilities framework models planner budget exhaustion (step limits, read limits, inspect limits, branch factor) and gatherer capability states (Available, Unsupported, HarnessRequired), but context quality is invisible. When operator memory is truncated at the 12k char boundary, when evidence snippets are clipped at 600 chars, when thread summaries lose detail at 80 chars — the system has no signal that context quality has degraded. Budget exhaustion triggers a `stop_reason` string in `PlannerSummary`, but context pressure has no equivalent mechanism.

Context pressure, staleness, and relevance decay need to be modeled as first-class capability signals within the existing constraints framework. The system should be able to report "context budget at 85% — operator memory truncated, 3 artifacts truncated" the same way it reports "inspect-budget-exhausted" today. This enables the system to self-diagnose and adapt when context quality degrades.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define `ContextPressure` as a measurable signal within the capabilities framework | Type exists with pressure level, contributing factors, and thresholds | Emitted as TurnEvent |
| GOAL-02 | Track context truncation events and aggregate them into a pressure score | Each truncation (memory, artifact, thread summary) contributes to a cumulative pressure metric | Pressure score computed per turn |
| GOAL-03 | Emit context pressure as a TurnEvent so the TUI and planner can observe it | Context pressure visible at verbose=1 in the turn stream | Event rendered in TUI |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer monitoring a complex turn | Understand when context quality is degraded and why |
| Planner loop | Recursive planner making evidence decisions | Adjust search strategy when context pressure indicates truncation or staleness |

## Scope

### In Scope

- [SCOPE-01] `ContextPressure` domain type with pressure level, truncation count, and contributing factors
- [SCOPE-02] Truncation tracking: count and categorize truncation events during context assembly
- [SCOPE-03] `TurnEvent::ContextPressure` for visibility in the turn event stream
- [SCOPE-04] TUI rendering of context pressure at verbose=1+

### Out of Scope

- [SCOPE-05] Automatic planner strategy adaptation based on pressure (future work)
- [SCOPE-06] Staleness detection based on temporal decay (requires timestamp tracking not yet in place)
- [SCOPE-07] Cross-turn pressure trend analysis

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Define `ContextPressure` with `level: PressureLevel` (Low/Medium/High/Critical), `truncation_count: usize`, and `factors: Vec<PressureFactor>` | GOAL-01 | must | Models context quality as a structured signal |
| FR-02 | Define `PressureFactor` enum: MemoryTruncated, ArtifactTruncated, ThreadSummaryTrimmed, EvidenceBudgetExhausted | GOAL-01 | must | Categorizes sources of context pressure |
| FR-03 | Track truncation events during `build_interpretation_context()` and `build_planner_prior_context()` | GOAL-02 | must | Aggregates truncation into pressure score |
| FR-04 | Compute `PressureLevel` from factor count and severity: 0 factors=Low, 1-2=Medium, 3-5=High, 6+=Critical | GOAL-02 | should | Simple threshold-based classification |
| FR-05 | Emit `TurnEvent::ContextPressure` after interpretation context assembly | GOAL-03 | must | Makes pressure visible in the turn stream |
| FR-06 | Render `ContextPressure` in TUI at verbose=1 as "Context pressure: Medium (2 truncations: memory, artifact)" | GOAL-03 | should | Human-readable pressure summary |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Pressure tracking adds no measurable overhead to the interpretation context assembly path | GOAL-02 | must | Context assembly is on the critical path |
| NFR-02 | Pressure signals are informational — they do not alter turn flow in this epic | GOAL-03 | must | Observation before intervention |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Domain types | Unit test: construct ContextPressure with various factor combinations, verify level computation | Test output |
| Truncation tracking | Integration test: assemble context with truncated memory and artifacts, verify pressure emitted | Test output |
| TUI rendering | Manual: run turn with verbose=1, observe context pressure event | Session observation |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Truncation points in the codebase are identifiable and can be instrumented | May need refactoring truncation functions to accept a tracker | Audit all truncation call sites |
| Simple threshold-based pressure levels are useful without temporal decay | May need more sophisticated scoring later | User feedback on pressure accuracy |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should pressure be reported per-component or as a single aggregate? | Design | Open — aggregate first, per-component later |
| How to weight different truncation types (memory truncation vs artifact truncation)? | Design | Open — equal weight initially |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `ContextPressure` type models pressure level with contributing factors
- [ ] Truncation events are tracked during context assembly
- [ ] `TurnEvent::ContextPressure` is emitted and visible in the TUI at verbose=1
- [ ] Pressure tracking adds no measurable overhead
<!-- END SUCCESS_CRITERIA -->
