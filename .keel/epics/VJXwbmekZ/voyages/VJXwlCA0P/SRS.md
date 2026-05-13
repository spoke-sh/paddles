# Unified Agent Action Contract - SRS

## Summary

Epic: VJXwbmekZ
Goal: Define one recursive agent action decision contract that covers first and
subsequent model choices without a separate `InitialAction` routing type.

## Scope

### In Scope

- [SCOPE-01] Domain value types for a unified recursive agent action decision.
- [SCOPE-02] Compatibility tests proving existing first-action and recursive-action labels map into one vocabulary.
- [SCOPE-03] Shared schema renderer changes that render first and later variants from the same action entry set.
- [SCOPE-04] Parser/schema parity for semantic actions and `external_capability`.

### Out of Scope

- [SCOPE-05] Runtime control-flow migration; handled by voyage VJXwlE718.
- [SCOPE-06] Foundational documentation cleanup; handled by voyage VJXwlG70U.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The domain must expose one `AgentAction` / `AgentDecision` style contract that can represent first and later model decisions. | SCOPE-01 | FR-01 | cargo test |
| SRS-02 | The unified action contract must include terminal `answer`, workspace actions, `refine`, `branch`, and `stop` without treating `answer` as a separate pre-loop route. | SCOPE-01, SCOPE-02 | FR-01 | cargo test |
| SRS-03 | The shared schema renderer must render first and later decision variants from one canonical action-entry source, with `answer` availability controlled by variant. | SCOPE-03 | FR-02 | cargo test |
| SRS-04 | Schema parity tests must cover semantic workspace actions and `external_capability` against the Rust action contract. | SCOPE-04 | FR-02 | cargo test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Transitional compatibility must be explicit and tested; old names must not become a second hidden contract. | SCOPE-01, SCOPE-02 | NFR-03 | cargo test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
