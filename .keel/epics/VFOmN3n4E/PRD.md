# Recursive Self-Assessing Compaction - Product Requirements

## Problem Statement

Current context compaction in paddles is purely mechanical: operator memory truncated at 12,000 chars, thread summaries at 80 chars, evidence snippets at 600 chars, artifact envelopes at configurable inline limits. These fixed caps have no awareness of relevance, staleness, or cross-component dependencies. A 12k AGENTS.md file is truncated at a byte boundary regardless of which sections are most relevant to the current turn.

The system needs to evaluate its own context state and decide what to compact, promote, archive, or surface. This self-assessment should use the same bounded planner/evidence mechanisms the system uses for workspace tasks — treating context evaluation as a planner task with budget constraints. Compacted summaries become first-class artifact envelopes that can themselves be further compacted, making compaction recursive rather than terminal.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define a `CompactionRequest` type that describes what context to evaluate and the budget for evaluation | Type compiles with fields for scope, budget, and relevance criteria | Type exists |
| GOAL-02 | Implement bounded self-assessment: the system evaluates a context scope and produces a relevance-ranked compaction plan | Given a set of context artifacts, produce a ranked list of keep/compact/discard decisions | Plan produced within budget |
| GOAL-03 | Compacted output is a first-class artifact envelope with a locator pointing to the pre-compaction content | Compacted summary carries a `ContextLocator` to its source material | Locator resolution works |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Turn orchestrator | MechSuitService assembling interpretation context | Fit more relevant context into fixed budgets by compacting stale or low-relevance material |
| Operator memory loader | Agent memory adapter loading AGENTS.md cascade | Intelligent truncation that preserves high-relevance sections over low-relevance ones |

## Scope

### In Scope

- [SCOPE-01] `CompactionRequest` and `CompactionPlan` domain types
- [SCOPE-02] Bounded self-assessment: evaluate context artifacts for relevance using the planner's evidence mechanisms
- [SCOPE-03] Compacted summaries as artifact envelopes with locators to pre-compaction content

### Out of Scope

- [SCOPE-04] Automatic compaction triggers (this epic provides the mechanism, not the policy)
- [SCOPE-05] Multi-tier compaction cascading (promote from inline to transit)
- [SCOPE-06] Changing the existing fixed character limits (this adds smart compaction alongside them)

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Define `CompactionRequest` with target scope (which artifacts), relevance query (current turn context), and budget (max assessment steps) | GOAL-01 | must | Structures the compaction task as a bounded operation |
| FR-02 | Define `CompactionPlan` with ranked artifact decisions: keep (with priority), compact (with summary), or discard (with reason) | GOAL-01 | must | Makes compaction decisions explicit and auditable |
| FR-03 | Implement `assess_context_relevance()` that takes a `CompactionRequest` and produces a `CompactionPlan` using the planner's evidence-gathering mechanisms | GOAL-02 | must | Reuses existing bounded planning for self-assessment |
| FR-04 | Compacted summaries are wrapped in `ArtifactEnvelope` with a `ContextLocator::Transit` pointing to the full pre-compaction content | GOAL-03 | should | Enables recursive compaction — the summary itself can be compacted later |
| FR-05 | Self-assessment respects the same `PlannerBudget` constraints as workspace tasks (max steps, bounded evidence) | GOAL-02 | must | Prevents unbounded meta-recursion per charter constraint |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Self-assessment budget is strictly bounded — cannot exceed the requesting planner's remaining budget | GOAL-02 | must | Prevents infinite meta-recursion |
| NFR-02 | Compaction is composable — a compacted summary can be input to another compaction round | GOAL-03 | should | Enables recursive compaction |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Domain types | Unit test: construct CompactionRequest and CompactionPlan, verify serialization | Test output |
| Self-assessment | Integration test: assess a set of artifacts, verify ranked plan respects budget | Test output |
| Recursive compaction | Manual: compact an artifact, then compact the compacted summary | Session observation |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing planner evidence mechanisms can evaluate context relevance, not just workspace content | May need a specialized assessment prompt template | Test with context artifacts as input |
| Compaction plan decisions are deterministic enough to be useful | May produce inconsistent results across runs | Compare plans across multiple assessments |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How to prevent compaction from consuming too much of the planner's budget? | Design | Open — likely a separate sub-budget carved from the parent |
| Should compaction be synchronous within a turn or deferred to between turns? | Design | Open — synchronous for first iteration |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `CompactionRequest` and `CompactionPlan` types exist with proper domain modeling
- [ ] Self-assessment produces relevance-ranked decisions within bounded budget
- [ ] Compacted summaries carry locators to their source material
- [ ] Self-assessment cannot exceed its allocated budget
<!-- END SUCCESS_CRITERIA -->
