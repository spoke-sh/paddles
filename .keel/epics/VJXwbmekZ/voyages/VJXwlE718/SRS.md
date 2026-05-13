# Recursive Loop Migration - SRS

## Summary

Epic: VJXwbmekZ
Goal: Run the first model action as step zero of the same recursive agent loop
while preserving direct answers, edit obligations, and fail-closed behavior.

## Scope

### In Scope

- [SCOPE-01] Runtime migration from pre-loop initial routing to step-zero agent action execution.
- [SCOPE-02] Direct answer and stop as terminal loop actions.
- [SCOPE-03] Known-edit, commit, review, and repository-grounding bootstrap behavior.
- [SCOPE-04] Fail-closed invalid-reply and unavailable-planner behavior.

### Out of Scope

- [SCOPE-05] New action types.
- [SCOPE-06] Provider capability expansion.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The first model decision must be represented and recorded as step zero of the recursive agent loop. | SCOPE-01 | FR-03 | cargo test |
| SRS-02 | `answer` and `stop` must terminate the same loop and produce the same user-visible behavior as existing direct-answer paths. | SCOPE-02 | FR-03 | cargo test |
| SRS-03 | Workspace, refine, and branch first actions must enter the same execution path used by later recursive decisions. | SCOPE-01 | FR-03 | cargo test |
| SRS-04 | Edit obligations, candidate-file hints, known-edit bootstrap, commit bootstrap, review bootstrap, and repository-grounding bootstrap must survive the migration. | SCOPE-03 | FR-04 | cargo test |
| SRS-05 | Invalid or unavailable model decisions must still fail closed with typed stop/block behavior. | SCOPE-04 | FR-04 | cargo test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The migration must not add network dependencies or provider-specific branches outside existing adapter boundaries. | SCOPE-01 | NFR-01 | cargo test |
| SRS-NFR-02 | Loop observability must still show terminal actions, workspace actions, evidence, and steering reviews. | SCOPE-01, SCOPE-03 | NFR-02 | cargo test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
