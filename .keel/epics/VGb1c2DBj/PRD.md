# Role-Based Multi-Agent Delegation - Product Requirements

## Problem Statement

Paddles has thread branching and specialist hints, but it lacks a general subagent lifecycle with explicit roles, delegation contracts, and wait/close semantics that let the recursive harness coordinate parallel workers safely.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define a first-class subagent lifecycle for spawning, messaging, waiting, resuming, and closing delegated work. | Parent and worker turns communicate through explicit lifecycle operations and durable artifacts. | Delegation is no longer implicit thread branching plus hope. |
| GOAL-02 | Make delegation role-based and ownership-aware so parallel work can proceed safely. | Worker roles, write ownership, and integration responsibilities are explicit in runtime state and UI output. | Multi-agent work reduces collisions instead of creating them. |
| GOAL-03 | Keep delegated work auditable inside the same recursive harness. | Worker outputs, tool calls, and integration results are traceable and replayable from the parent turn. | Subagents remain part of the same system, not hidden sidecars. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive Operator | A user asking Paddles to split a larger task into bounded concurrent slices. | Faster progress without losing visibility into who did what. |
| Runtime Maintainer | A contributor formalizing the existing subagent and thread lineage foundations. | A delegation contract that does not rely on prompt-only convention. |
| Small-Model Harness Designer | A maintainer trying to lift capability through orchestration rather than a larger single model. | Safe parallel delegation that stays within one recursive-harness architecture. |

## Scope

### In Scope

- [SCOPE-01] Define lifecycle operations for spawning, messaging, waiting on, resuming, and closing subagents.
- [SCOPE-02] Add explicit role and ownership contracts for delegated work.
- [SCOPE-03] Capture worker artifacts, tool calls, and final summaries as parent-inspectable runtime records.
- [SCOPE-04] Surface delegated work and integration state in operator-facing transcript/projection views.
- [SCOPE-05] Update docs and tests around multi-agent delegation semantics.

### Out of Scope

- [SCOPE-06] Unbounded autonomous swarms, self-replicating delegation, or cloud cluster orchestration.
- [SCOPE-07] Parallel write access without explicit ownership or merge responsibility.
- [SCOPE-08] Replacing the parent recursive loop with a separate orchestration product.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must expose explicit subagent lifecycle operations including spawn, follow-up input, wait, resume, and close. | GOAL-01 | must | Lifecycle control is the backbone of any real delegation model. |
| FR-02 | Delegated work must carry explicit role metadata and ownership guidance so parent and worker responsibilities are clear. | GOAL-02 | must | Role-based delegation is safer and more legible than generic branch spawning. |
| FR-03 | Worker outputs, tool calls, and final summaries must be recorded as traceable artifacts that the parent can inspect and integrate. | GOAL-01, GOAL-03 | must | Delegation only helps if results are inspectable and grounded. |
| FR-04 | The parent runtime must be able to continue non-overlapping work while delegated workers run, then integrate or refine returned results without losing lineage. | GOAL-01, GOAL-03 | should | Parallelism is the point of delegation. |
| FR-05 | Conflicting ownership or invalid lifecycle requests must degrade honestly with explicit status instead of silently merging unsafe changes. | GOAL-02, GOAL-03 | must | Multi-agent trust depends on visible conflict handling. |
| FR-06 | Operator-facing surfaces must show active workers, roles, ownership, and completion state clearly enough to follow parallel progress. | GOAL-02, GOAL-03 | should | Delegation should not disappear into hidden background tasks. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Delegation must remain bounded by the same execution-governance and evidence policies as the parent harness. | GOAL-02, GOAL-03 | must | Subagents cannot become a policy bypass. |
| NFR-02 | Multi-agent coordination must stay replayable and comprehensible across transcript and projection surfaces. | GOAL-03 | must | Operators need to audit delegated work after the fact. |
| NFR-03 | Ownership semantics must minimize merge conflicts and hidden shared-state mutation. | GOAL-02 | must | Safe parallelism depends on explicit boundaries. |
| NFR-04 | The delegation model must preserve one recursive-harness identity rather than spawning an unrelated orchestration subsystem. | GOAL-01, GOAL-03 | must | The mission is to deepen Paddles, not fork it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Lifecycle semantics | Unit and runtime tests over spawn/message/wait/resume/close flows | Story-level verification artifacts and command logs |
| Ownership and conflict handling | Integration tests for overlapping versus disjoint write scopes | Story-level verification artifacts and transcript proofs |
| Operator visibility | TUI/web/API projections showing active workers and integration results | Story-level UI proofs and doc updates |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current thread-lineage and specialist-brain foundations can evolve into a broader delegation runtime. | The harness may need a deeper orchestration substrate before multi-agent work is safe. | Validate lifecycle proposals against the existing lineage model during decomposition. |
| A small set of roles and ownership conventions will cover most practical delegation needs initially. | The first taxonomy may be too narrow or too rigid. | Start with a bounded role set and test it against real planning slices. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which initial role taxonomy matters most: explorer/worker/reviewer, or a more Paddles-native vocabulary? | Epic owner | Open |
| How should parent turns decide when to wait for workers versus continuing local non-overlapping work? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles exposes explicit subagent lifecycle operations instead of treating delegation as ad-hoc thread branching.
- [ ] Worker roles and ownership boundaries are visible and enforceable enough to support parallel work safely.
- [ ] Delegated outputs are captured as traceable artifacts that parent turns can inspect and integrate.
- [ ] Operator-facing surfaces make multi-agent progress understandable without hiding conflicts or authority boundaries.
<!-- END SUCCESS_CRITERIA -->
