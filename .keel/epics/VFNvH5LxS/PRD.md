# Recursive Interpretation Refinement - Product Requirements

## Problem Statement

The interpretation context is assembled in a single model pass with no self-validation — the model may miss precedence rules, overlook conflicts between guidance documents, or produce a shallow understanding that causes downstream planner decisions to drift from operator intent.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Interpretation context identifies what categories of guidance it found (rules, conventions, procedures, constraints) | Structured classification in the context output | Every derived context includes typed categories |
| GOAL-02 | Interpretation context surfaces precedence and conflict resolution | Explicit precedence chain and detected conflicts in the output | Conflicts logged; precedence chain matches AGENTS.md hierarchy |
| GOAL-03 | Interpretation quality improves through bounded recursive refinement | Refinement loop catches gaps that single-pass misses | Measurable via before/after comparison on guidance-heavy workspaces |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer relying on paddles to follow workspace conventions | Confidence that paddles understood the rules before acting |
| Guidance author | Developer maintaining AGENTS.md hierarchy | Feedback on whether guidance is being interpreted as intended |

## Scope

### In Scope

- [SCOPE-01] Typed guidance categories in the interpretation context schema
- [SCOPE-02] Precedence chain extraction from document hierarchy
- [SCOPE-03] Conflict detection between guidance documents
- [SCOPE-04] Bounded refinement loop (validate → identify gaps → re-expand → re-assemble)
- [SCOPE-05] Coverage assessment: "what did I find vs what might I be missing?"

### Out of Scope

- [SCOPE-06] External resource fetching (HTTP, APIs) during interpretation
- [SCOPE-07] Persistent knowledge graph across sessions
- [SCOPE-08] User-editable interpretation overrides

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Interpretation prompt requests typed guidance categories (rules, conventions, constraints, procedures, preferences) | GOAL-01 | must | Structures the model's understanding |
| FR-02 | Interpretation prompt requests explicit precedence chain from document hierarchy | GOAL-02 | must | Makes conflict resolution visible |
| FR-03 | Interpretation prompt asks model to identify conflicts between guidance sources and state resolution | GOAL-02 | must | Surfaces contradictions before they cause drift |
| FR-04 | Add a validation pass after initial assembly that checks for gaps: "are there guidance areas the user prompt touches that I found no rules for?" | GOAL-03 | must | Core self-validation loop |
| FR-05 | If validation identifies gaps, re-expand the guidance graph targeting missing areas (bounded to 1 additional expansion cycle) | GOAL-03 | must | Recursive quality improvement |
| FR-06 | Re-assemble interpretation context after gap-filling expansion | GOAL-03 | must | Completes the refinement loop |
| FR-07 | Emit TurnEvent for each refinement iteration so the user sees the loop working | GOAL-03 | should | Transparency |
| FR-08 | Add a coverage confidence field to InterpretationContext ("high"/"medium"/"low" based on gaps found) | GOAL-03 | should | Quick signal for downstream decision-making |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Refinement loop bounded to max 1 additional cycle (2 total assembly passes) | - | must | Prevents runaway cost |
| NFR-02 | Total additional model calls for refinement capped at 2 (validate + re-assemble) | - | must | Predictable cost ceiling |
| NFR-03 | Fallback to single-pass result if refinement fails or times out | - | must | Never worse than today |
| NFR-04 | No new crate dependencies | - | must | Keep dependency surface minimal |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Category typing | Unit test: known AGENTS.md → expected categories in output | Test artifact |
| Precedence extraction | Unit test: multi-level hierarchy → correct precedence chain | Test artifact |
| Conflict detection | Unit test: contradictory guidance → conflict surfaced | Test artifact |
| Refinement loop | Integration test: guidance with deliberate gap → gap detected and filled | Test artifact |
| Fallback safety | Unit test: refinement failure → single-pass result returned | Test artifact |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| One refinement cycle is enough to catch most gaps | May need 2+ cycles for complex guidance hierarchies | Monitor gap detection rate in first sessions |
| Models can reliably detect conflicts between guidance documents | May produce false positives or miss subtle contradictions | Manual review of conflict reports |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The validation prompt design is critical — poor prompting will miss gaps | Operator | Open — needs prompt engineering iteration |
| Cost of 2 additional model calls per turn may be noticeable on slow providers | Operator | Mitigated by NFR-01/NFR-02 bounds |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Interpretation context includes typed guidance categories
- [ ] Precedence chain is explicit and matches document hierarchy
- [ ] Conflicts between guidance documents are detected and reported
- [ ] Refinement loop fills at least one gap that single-pass missed (manual verification)
- [ ] Total interpretation cost bounded to 2 additional model calls max
<!-- END SUCCESS_CRITERIA -->
