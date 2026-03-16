# Foundational Chord Integration - Product Requirements

## Problem Statement

The paddles assistant lacks its primary 'chord' capabilities for agentic coding tasks. Currently, the `paddles` binary is a thin wrapper that doesn't utilize the `wonopcode` crate's capabilities, which are essential for its role as a "mech suit" for AI assistants.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Integrate `wonopcode` (chord) into the paddles CLI. | `paddles --prompt` executes a chord-powered task. | 100% |
| GOAL-02 | Establish foundational agentic coding workflow. | Chord can successfully modify a file in a controlled test. | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Using paddles to automate coding tasks. | High-fidelity execution of coding prompts via chord. |

## Scope

### In Scope

- [SCOPE-01] Wiring `wonopcode` into `src/main.rs`.
- [SCOPE-02] Basic prompt handling via chord.
- [SCOPE-03] Verification test for a simple chord-driven file modification.

### Out of Scope

- [SCOPE-04] Advanced multi-step reasoning or complex tool use.
- [SCOPE-05] Integration with `sift` for advanced retrieval (future epic).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `paddles --prompt <text>` should delegate to chord for execution. | GOAL-01 | must | Core functionality of the mech suit. |
| FR-02 | Chord must be able to read and write files within the project. | GOAL-02 | must | Necessary for coding tasks. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Chord execution must be observable via tracing. | GOAL-01 | must | Essential for debugging agentic behavior. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Problem outcome | Tests, CLI proofs, or manual review chosen during planning | Story-level verification artifacts linked during execution |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The problem statement reflects a real user or operator need. | The epic may optimize the wrong outcome. | Revisit with planners during decomposition. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which metric best proves the problem above is resolved? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The team can state a measurable user outcome that resolves the problem above.
<!-- END SUCCESS_CRITERIA -->
