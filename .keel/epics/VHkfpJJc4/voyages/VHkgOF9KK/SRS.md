# Create Recursive Harness Eval Suite - SRS

## Summary

Epic: VHkfpJJc4
Goal: Create a local eval harness and initial corpus proving recursive evidence gathering, tool failure recovery, edit obligations, delegation, context pressure, and architecture boundaries.

## Scope

### In Scope

- [SCOPE-07] Create a recursive harness evaluation suite that verifies capability disclosure, evidence gathering, tool failure recovery, edit obligations, delegation, context pressure, and architecture boundaries.

### Out of Scope

- [SCOPE-12] Vendor leaderboard benchmarking.
- [SCOPE-11] Online-only eval services.
- [SCOPE-12] Broad performance benchmarking beyond harness correctness.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Add an eval runner that executes local harness scenarios and reports structured outcomes. | SCOPE-07 | FR-07 | test: eval runner smoke test |
| SRS-02 | Add initial eval scenarios for recursive evidence, tool recovery, edit obligations, delegation, context pressure, and replay. | SCOPE-07 | FR-07 | eval: initial corpus passes |
| SRS-03 | Add architecture boundary checks that protect domain, application, and infrastructure layering. | SCOPE-07 | FR-07 | test: boundary check fixtures |
| SRS-04 | Document eval usage and expected evidence artifacts. | SCOPE-07 | FR-07 | manual: docs review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Evals run locally without network access by default. | SCOPE-07 | NFR-01 | test: offline eval mode |
| SRS-NFR-02 | Eval failures point to the violated harness contract. | SCOPE-07 | NFR-04 | test: failure reporting snapshots |
| SRS-NFR-03 | Eval harness delivery follows TDD with a failing runner test before implementation. | SCOPE-07 | NFR-02 | test: red/green eval runner proof |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
