# Split Application Into Hexagonal Boundaries - SRS

## Summary

Epic: VHkfpJJc4
Goal: Reduce the application monolith by extracting domain-driven ports, application services, and infrastructure adapters without changing recursive behavior.

## Scope

### In Scope

- [SCOPE-08] Split the application monolith using domain-driven design and hexagonal architecture so domain policy, application orchestration, and infrastructure adapters are independently testable.

### Out of Scope

- [SCOPE-10] Behavioral rewrites unrelated to boundary extraction.
- [SCOPE-12] Moving user-facing UI code before runtime boundaries are stable.
- [SCOPE-10] Changing recursive planner/synthesizer semantics.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Document the target DDD and hexagonal boundary map for the recursive harness. | SCOPE-08 | FR-08 | manual: architecture doc review |
| SRS-02 | Extract planner-loop orchestration from the application monolith behind explicit application services. | SCOPE-08 | FR-08 | test: behavior-preserving service tests |
| SRS-03 | Extract execution contract and capability disclosure assembly into focused services. | SCOPE-08 | FR-08 | test: contract snapshot tests |
| SRS-04 | Add boundary checks that protect domain, application, and infrastructure dependencies. | SCOPE-08 | FR-08 | test: architecture boundary checks |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Refactor slices preserve behavior and keep tests green at each commit boundary. | SCOPE-08 | NFR-05 | test: targeted and full relevant suite |
| SRS-NFR-02 | New modules follow domain/application/infrastructure naming and ownership conventions. | SCOPE-08 | NFR-05 | test: boundary checks |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
