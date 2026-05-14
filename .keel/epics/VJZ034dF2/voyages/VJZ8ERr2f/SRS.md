# Collapse Runtime Lane Terminology - SRS

## Summary

Epic: VJZ034dF2
Goal: Collapse planner, synthesizer, and gatherer lane terminology across user-facing surfaces and internal Rust code so the codebase centers on turn runtime phases and model clients.

## Scope

### In Scope

- [SCOPE-06] Rename internal Rust runtime lane types, planner/synthesizer/gatherer ports, and lane-shaped modules to turn runtime concepts.
- [SCOPE-07] Retire user-facing "runtime lane", "planner lane", "synthesizer lane", and "gatherer lane" language from CLI help, TUI/route copy, docs, and prompt prose.
- [SCOPE-04] Preserve tested turn phases for action selection, retrieval, execution, evidence accumulation, reflection/refinement, and final rendering.
- [SCOPE-05] Keep legacy compatibility aliases only when they are intentionally documented as migration shims.

### Out of Scope

- [SCOPE-06] Replacing the canonical turn loop with a different orchestration engine.
- [SCOPE-07] Removing Sift retrieval/indexing.
- [SCOPE-08] Hiding live capability surfaces or enforced execution constraints from the model.
- [SCOPE-09] Cosmetic-only renames that leave the lane architecture intact.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Internal Rust types such as runtime lane preparation, prepared model lanes, and prepared gatherer lanes are renamed to turn runtime, model-client, retrieval, and phase concepts. | SCOPE-06 | FR-07 | compile and targeted refactor tests |
| SRS-02 | Internal planner/synthesizer/gatherer ports and modules are renamed or retired where they encode lane architecture rather than stable turn-loop behavior. | SCOPE-06, SCOPE-04 | FR-07 | compile and behavioral tests |
| SRS-03 | User-facing surfaces no longer present planner, synthesizer, or gatherer as runtime lanes. | SCOPE-07 | FR-07 | string scans and UI/CLI tests |
| SRS-04 | Compatibility aliases, if retained, are explicit migration shims that point to action-selection, final-rendering, or retrieval phase terminology. | SCOPE-05 | FR-07 | config and doc tests |
| SRS-05 | Turn-loop behavior remains covered for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. | SCOPE-04 | FR-07 | existing and new behavioral tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The refactor is behavior-preserving except for intentional compatibility warnings/errors and documented surface text changes. | SCOPE-06, SCOPE-07 | NFR-03 | full test suite and diff review |
| SRS-NFR-02 | Prompt and execution-contract wording continues to expose live capability surfaces and enforced constraints rather than synthetic controller-authored plans. | SCOPE-07, SCOPE-04 | NFR-04 | prompt/contract tests |
| SRS-NFR-03 | Each implementation story starts with a failing test or doc check before runtime behavior or owning docs are changed. | SCOPE-06, SCOPE-07, SCOPE-04, SCOPE-05 | NFR-01 | story evidence review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
