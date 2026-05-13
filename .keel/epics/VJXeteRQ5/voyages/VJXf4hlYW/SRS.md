# Planner Lane Schema Adoption - SRS

## Summary

Epic: VJXeteRQ5
Goal: Replace adapter-local action-schema blocks with the shared renderer and prove Sift and HTTP mocked turns receive the same canonical schema.

## Scope

### In Scope

- [SCOPE-01] Replace Sift planner prompt hardcoded action-schema blocks.
- [SCOPE-02] Replace HTTP planner prompt hardcoded action-schema blocks.
- [SCOPE-03] Preserve provider-specific transport instructions.
- [SCOPE-04] Add mocked-turn parity tests for Sift and HTTP prompts.

### Out of Scope

- [SCOPE-05] Changing provider transport protocols.
- [SCOPE-06] Changing local execution governance or capability filtering.
- [SCOPE-07] Foundational documentation updates.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Sift initial, recursive, retry, and redecision prompts must use the shared schema renderer for action JSON examples and shared rules. | SCOPE-01 | FR-04 | mocked turn + prompt test |
| SRS-02 | HTTP planner prompts must use the shared schema renderer for action JSON examples and shared rules while retaining transport-specific instructions. | SCOPE-02, SCOPE-03 | FR-04 | mocked turn + prompt test |
| SRS-03 | Remove or reduce adapter-local action-schema prose to calls into the shared renderer. | SCOPE-01, SCOPE-02 | FR-04 | code review + `rg` proof |
| SRS-04 | Mocked Sift and HTTP turns must prove the same canonical schema block is present for initial prompts. | SCOPE-04 | FR-05 | automated test |
| SRS-05 | Mocked Sift and HTTP turns must prove the same canonical schema block is present for recursive next-action prompts. | SCOPE-04 | FR-05 | automated test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Provider-specific transport wording may differ, but canonical schema text must compare equal across lanes. | SCOPE-03, SCOPE-04 | NFR-02 | automated test |
| SRS-NFR-02 | Test failures must identify which lane or prompt variant drifted. | SCOPE-04 | NFR-03 | automated test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
