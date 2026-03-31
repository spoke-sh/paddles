# Planner Loop Reasoning Visibility - Product Requirements

## Problem Statement

During multi-step recursive planner loops that take 30-120+ seconds, the TUI provides almost no insight into what the system is reasoning about. At verbose=0 users see only a spinner and elapsed time. At verbose=1 they get dense technical summaries after each step completes, but nothing that explains the reasoning thread — what evidence the system is pursuing, why it branched, or how progress is tracking toward the goal. Users need verbosity-tiered visibility into planner reasoning so they can tell whether the system is making progress or stuck.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | At verbose=0, users see a concise live status line showing what the planner is doing right now | TUI shows current step, action type, and target during planner execution | Updated each planner step |
| GOAL-02 | At verbose=1, users see the reasoning thread — why each action was chosen and what was found | Each planner step shows rationale, query/target, and evidence outcome | Shown for every step |
| GOAL-03 | At verbose=2, users see full structural detail including branch topology, budget consumption, and context assembly | Complete planner state visible including graph branches, frontier, and retained artifacts | Every step + loop summary |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer waiting for a complex planned turn | Understand whether the system is making progress and what it's pursuing |
| Power user / debugger | Developer troubleshooting unexpected results | See the full decision chain to identify where reasoning went wrong |

## Scope

### In Scope

- [SCOPE-01] New TurnEvent for planner step progress (verbose=0 tier, in-place updates)
- [SCOPE-02] Enriched PlannerActionSelected rendering with human-readable reasoning at verbose=1
- [SCOPE-03] Branch and refine decision reasoning events at verbose=1
- [SCOPE-04] Budget consumption indicators (steps used/limit, evidence count) at verbose=1
- [SCOPE-05] Full planner graph state rendering at verbose=2

### Out of Scope

- [SCOPE-06] Cancellation of in-progress planner loops
- [SCOPE-07] Changing the planner's actual decision-making logic
- [SCOPE-08] Progress percentage estimation (requires upstream sift changes)

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Emit a TurnEvent::PlannerStepProgress at the start of each planner step with step number, limit, action type, and target query | GOAL-01 | must | Gives verbose=0 users a live status of what's happening |
| FR-02 | PlannerStepProgress events update in-place in the TUI like GathererSearchProgress | GOAL-01 | must | Prevents transcript clutter from step-by-step accumulation |
| FR-03 | At verbose=1, PlannerActionSelected renders with human-readable rationale and the specific query or command being executed | GOAL-02 | must | Users can follow the reasoning thread |
| FR-04 | At verbose=1, emit a brief evidence outcome after each gather/refine step showing what was found and retained | GOAL-02 | should | Closes the loop on each step's contribution |
| FR-05 | At verbose=1, branch and refine decisions explain why the planner chose to branch or refine rather than stop | GOAL-02 | should | Makes multi-step reasoning chains comprehensible |
| FR-06 | At verbose=2, PlannerSummary includes full graph topology (branch IDs, frontier entries, node count, edge count) | GOAL-03 | should | Power users can inspect the complete decision structure |
| FR-07 | At verbose=1+, show budget consumption as "step N/M" and "evidence: K items" in progress indicators | GOAL-02 | should | Users can gauge how much budget remains |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Progress events emitted synchronously within the planner loop without adding latency to the step cycle | GOAL-01 | must | Progress reporting must not slow down the planner |
| NFR-02 | Verbose=0 output remains compact — at most one in-place progress line during the entire loop | GOAL-01 | must | Default experience stays clean |
| NFR-03 | Verbose=1 output is scannable — each step renders in 2-3 lines, not walls of text | GOAL-02 | should | Readable at a glance |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Verbose=0 progress | Manual: run multi-step query, observe in-place step updates | Session observation |
| Verbose=1 reasoning | Manual: run with -v, verify each step shows rationale and outcome | Session output |
| Verbose=2 structure | Manual: run with -vv, verify graph topology in PlannerSummary | Session output |
| No performance regression | Manual: compare turn latency before/after | Timing comparison |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing TurnEventSink is available inside the planner loop | May need to thread the sink through additional call sites | Verify trace access in execute_recursive_planner_loop |
| In-place row updates work correctly for planner progress like they do for search progress | May need a different rendering approach | Test with multi-step queries |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should planner step progress replace search progress or coexist (search within a step)? | Implementation | Open — likely coexist since search progress is within a step |
| How to handle graph-mode branches at verbose=1 without overwhelming? | Design | Open — probably show active branch only |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] At verbose=0, a multi-step planner turn shows live "Step N/M: action" updates instead of just a spinner
- [ ] At verbose=1, each planner step shows what was searched, why, and what was found
- [ ] At verbose=2, full graph topology and budget state are visible
- [ ] No performance regression on the planner loop critical path
<!-- END SUCCESS_CRITERIA -->
