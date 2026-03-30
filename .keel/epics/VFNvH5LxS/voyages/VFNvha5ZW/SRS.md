# Refinement Loop Core - SRS

## Summary

Epic: VFNvH5LxS
Goal: Add typed guidance categories, precedence extraction, conflict detection, and coverage gap validation to the interpretation pipeline

## Scope

### In Scope

- [SCOPE-01] Typed guidance categories in interpretation schema and prompt
- [SCOPE-02] Precedence chain extraction from document hierarchy
- [SCOPE-03] Conflict detection between guidance sources
- [SCOPE-04] Standalone validation pass for coverage gap detection

### Out of Scope

- [SCOPE-05] Wiring the refinement loop into the application layer (voyage 2)
- [SCOPE-06] Coverage confidence signaling (voyage 2)

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | GuidanceCategory enum (Rules, Conventions, Constraints, Procedures, Preferences) in planning.rs | SCOPE-01 | FR-01 | manual |
| SRS-02 | InterpretationContext.categories field populated from model response | SCOPE-01 | FR-01 | manual |
| SRS-03 | Interpretation prompt requests typed guidance categories | SCOPE-01 | FR-01 | manual |
| SRS-04 | InterpretationContext.precedence_chain field with source, rank, scope_label | SCOPE-02 | FR-02 | manual |
| SRS-05 | Interpretation prompt requests precedence chain given document loading order | SCOPE-02 | FR-02 | manual |
| SRS-06 | InterpretationContext.conflicts field with sources, description, resolution | SCOPE-03 | FR-03 | manual |
| SRS-07 | Interpretation prompt requests conflict identification and resolution | SCOPE-03 | FR-03 | manual |
| SRS-08 | Standalone validation function accepts InterpretationContext + prompt, returns gap list | SCOPE-04 | FR-04 | manual |
| SRS-09 | Validation function makes one model call to identify uncovered areas | SCOPE-04 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Unrecognized category/precedence values fall back gracefully | SCOPE-01 | NFR-04 | manual |
| SRS-NFR-02 | No new crate dependencies | SCOPE-01 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
