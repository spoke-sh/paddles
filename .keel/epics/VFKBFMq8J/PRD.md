# Transit Trace Visualization - Product Requirements

## Problem Statement

Transit trace streams capture rich lineage data (branches, merges, checkpoints) but there is no visual representation. A railroad-style DAG with hexagonal turnstep nodes needs to render trace records in real time, showing branch divergence, planner action sequences, and merge reconciliation.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | A trace graph endpoint returns the DAG structure for a session | GET /sessions/:id/trace/graph returns nodes, edges, and branches as JSON | First voyage |
| GOAL-02 | A browser visualization renders the trace DAG with hexagonal nodes in real time | SVG railroad diagram updates as TurnEvents stream in | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles locally | Visualize the recursive planning process as a branching railroad of decisions |

## Scope

### In Scope

- [SCOPE-01] GET /sessions/:id/trace/graph endpoint returning nodes, edges, and branches
- [SCOPE-02] Browser SVG visualization rendering the DAG as a railroad diagram
- [SCOPE-03] Hexagonal node shapes with color/icon conveying TraceRecordKind
- [SCOPE-04] Real-time updates as new TraceRecords arrive via SSE
- [SCOPE-05] Branch swimlanes showing divergence and merge points

### Out of Scope

- [SCOPE-06] Historical trace browsing across sessions
- [SCOPE-07] Interactive node editing or annotation
- [SCOPE-08] 3D or physics-based graph layouts

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Trace graph endpoint converts TraceReplay into a flat node/edge/branch JSON structure | GOAL-01 | must | API for visualization clients |
| FR-02 | SVG visualization renders hexagonal nodes positioned in railroad-style vertical flow | GOAL-02 | must | Core visual metaphor |
| FR-03 | Node color and label reflect TraceRecordKind (root, action, tool, checkpoint, merge) | GOAL-02 | must | Visual semantics |
| FR-04 | Branch divergence renders as parallel swimlanes splitting from the mainline | GOAL-02 | must | Shows planner branching |
| FR-05 | Merge records render as lanes converging back | GOAL-02 | must | Shows thread reconciliation |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Visualization is part of the same HTML served by paddles, no separate build | GOAL-02 | must | Consistent with chat interface approach |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Graph endpoint | curl /sessions/:id/trace/graph returns valid JSON with nodes and edges | Response structure matches spec |
| Visualization | Manual: run a multi-step turn, observe hexagonal nodes appearing in browser | Screenshot |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| SVG rendering is performant for traces up to hundreds of nodes | Would need canvas or virtualization | Typical turns produce fewer than 20 trace records |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should branch swimlanes use fixed or dynamic X-offsets | operator | Resolved: fixed offsets for simplicity |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Trace graph endpoint returns structured DAG data for a completed turn
- [ ] Browser renders hexagonal turnstep nodes with branch/merge visualization
<!-- END SUCCESS_CRITERIA -->
