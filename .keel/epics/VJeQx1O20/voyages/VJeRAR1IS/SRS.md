# Retire Pre-Loop Bootstraps And Vocabulary - SRS

## Summary

Epic: VJeQx1O20
Goal: Controller bootstraps, traces, docs, and runtime vocabulary reflect a single agent-loop action-selection architecture with compatibility shims isolated.

## Scope

### In Scope

- [SCOPE-01] Remove or collapse commit, known-edit, repository-grounding, and review bootstraps that force initial actions before the loop.
- [SCOPE-02] Update trace/runtime presentation to describe one agent-loop action-selection path.
- [SCOPE-03] Update architecture/configuration docs to describe the unified loop.
- [SCOPE-04] Add regression coverage for the original repeated-read/operator-contract failure shape.

### Out of Scope

- [SCOPE-05] Broad refactors unrelated to turn entry, bootstraps, traces, or docs.
- [SCOPE-06] Changing Keel board semantics or AGENTS.md operator guidance.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Pre-loop bootstraps must be deleted or represented as loop signals/instruction-frame data selected by the loop. | SCOPE-01 | FR-04 | static search and focused test |
| SRS-02 | Runtime labels must not imply a separate planner/initial-action lane for normal turns. | SCOPE-02 | FR-02 | static search |
| SRS-03 | Architecture docs must identify the agent loop as the single turn action-selection owner. | SCOPE-03 | FR-02 | doc review |
| SRS-04 | A regression test must cover an operator-contract question where the loop should answer from evidence instead of repeatedly reading the same source due to pre-loop control. | SCOPE-04 | FR-01 | focused test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Static cleanup must distinguish internal architecture cleanup from provider compatibility shims. | SCOPE-02, SCOPE-03 | NFR-03 | static search |
| SRS-NFR-02 | Full library tests and formatting must pass after cleanup. | SCOPE-01, SCOPE-04 | NFR-01 | `cargo fmt --all --check`, `cargo test --lib` |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
