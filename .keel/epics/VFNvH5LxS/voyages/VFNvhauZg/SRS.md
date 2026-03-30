# Refinement Loop Integration - SRS

## Summary

Epic: VFNvH5LxS
Goal: Wire the refinement loop into the application layer, emit progress events, and add coverage confidence

## Scope

### In Scope

- [SCOPE-01] Bounded gap-filling re-expansion cycle
- [SCOPE-02] Application layer wiring of validate → re-expand → re-assemble
- [SCOPE-03] Coverage confidence field and refinement TurnEvents

### Out of Scope

- [SCOPE-04] The core types and validation function (voyage 1)
- [SCOPE-05] More than 1 re-expansion cycle

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Re-expand guidance graph targeting gap areas with suggestions as hints, bounded to 1 cycle | SCOPE-01 | FR-05 | manual |
| SRS-02 | Re-assemble interpretation context from expanded graph after gap-filling | SCOPE-01 | FR-06 | manual |
| SRS-03 | Fall back to original context if re-expansion fails | SCOPE-01 | FR-05 | manual |
| SRS-04 | Application layer calls validate → re-expand → re-assemble after derive_interpretation_context | SCOPE-02 | FR-06 | manual |
| SRS-05 | Total refinement capped at 2 model calls (validate + re-assemble) | SCOPE-02 | NFR-02 | manual |
| SRS-06 | TurnEvents emitted for each refinement stage | SCOPE-02 | FR-07 | manual |
| SRS-07 | CoverageConfidence enum (High/Medium/Low) on InterpretationContext | SCOPE-03 | FR-08 | manual |
| SRS-08 | TurnEvent::InterpretationValidated emitted after validation | SCOPE-03 | FR-07 | manual |
| SRS-09 | TurnEvent::InterpretationRefined emitted after refinement cycle | SCOPE-03 | FR-07 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Refinement loop bounded to max 1 additional cycle | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | Fallback to single-pass on any failure | SCOPE-01 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
