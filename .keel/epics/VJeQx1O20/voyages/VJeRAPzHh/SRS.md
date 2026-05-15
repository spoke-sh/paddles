# Move Turn Contract Into Agent Loop - SRS

## Summary

Epic: VJeQx1O20
Goal: Read-only, execution, review, edit, commit, and grounding policy are loop inputs and execution-contract constraints rather than pre-loop routing decisions.

## Scope

### In Scope

- [SCOPE-01] Rename and reshape `context.collaboration` into a turn contract/policy input.
- [SCOPE-02] Carry mutation posture, output contract, clarification posture, edit obligation, commit obligation, and grounding requirements through the agent loop.
- [SCOPE-03] Enforce read-only/review/execute behavior through the loop request and execution contract.
- [SCOPE-04] Preserve trace disclosure of active turn contract decisions.

### Out of Scope

- [SCOPE-05] Replacing execution hand authorization or command policy.
- [SCOPE-06] Adding new collaboration modes beyond existing planning, execution, and review semantics.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Rename runtime use of `CollaborationModeResult` in the agent loop to a turn contract/policy concept while preserving serialized compatibility where needed. | SCOPE-01 | FR-03 | static search and tests |
| SRS-02 | Read-only and review mutation blocks must be enforced inside the loop/execution contract after model action selection. | SCOPE-02, SCOPE-03 | FR-03 | focused test |
| SRS-03 | Edit and commit obligations must be modeled as instruction-frame or loop-state data consumed by the first loop request. | SCOPE-02 | FR-04 | focused test |
| SRS-04 | Grounding pressure must be loop context, not a forced pre-loop bootstrap action. | SCOPE-02 | FR-04 | focused test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Governance denial and read-only stop traces must remain explicit and user-facing. | SCOPE-03, SCOPE-04 | NFR-01 | focused test |
| SRS-NFR-02 | Internal names must make the architecture legible: turn contract/policy, not collaboration lane. | SCOPE-01 | NFR-03 | static search |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
