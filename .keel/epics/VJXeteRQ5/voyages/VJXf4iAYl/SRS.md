# Planner Schema Documentation - SRS

## Summary

Epic: VJXeteRQ5
Goal: Update foundational docs so the turn loop and planner action contract describe one shared schema renderer plus turn-specific capability manifests.

## Scope

### In Scope

- [SCOPE-01] README updates for the turn loop and bounded action contract.
- [SCOPE-02] POLICY updates for shared schema ownership and adapter drift.
- [SCOPE-03] ARCHITECTURE updates for implementation boundary and prompt lanes.
- [SCOPE-04] Any other owning foundational docs touched by the behavior change.

### Out of Scope

- [SCOPE-05] Marketing copy or generated docs site redesign.
- [SCOPE-06] New runtime behavior beyond documenting the shared schema contract.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | README must describe one shared planner action schema renderer and clarify that capability manifests are turn-specific. | SCOPE-01 | FR-06 | doc diff review |
| SRS-02 | POLICY must state that adapter-local planner action schema lists are not allowed. | SCOPE-02 | FR-06 | doc diff review |
| SRS-03 | ARCHITECTURE must identify the shared renderer boundary and provider adapter responsibilities. | SCOPE-03 | FR-06 | doc diff review |
| SRS-04 | Documentation must mention semantic actions and `external_capability` as part of the canonical schema when capability-disclosed. | SCOPE-01, SCOPE-03 | FR-06 | doc diff review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Docs must not imply remote-only schema disclosure or final-answer-only purpose. | SCOPE-01, SCOPE-03 | NFR-02 | doc diff review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
