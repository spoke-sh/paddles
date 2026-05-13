# Shared Planner Schema Contract - SRS

## Summary

Epic: VJXeteRQ5
Goal: Create the shared authored planner action schema renderer and tests proving it matches the Rust action surface.

## Scope

### In Scope

- [SCOPE-01] Shared authored planner action schema data structure.
- [SCOPE-02] Renderer for prompt-facing schema blocks and shared action rules.
- [SCOPE-03] Tests proving schema entries match the Rust action surface.
- [SCOPE-04] Inclusion of semantic workspace actions and `external_capability`.

### Out of Scope

- [SCOPE-05] Migrating provider prompts to the renderer.
- [SCOPE-06] Foundational documentation updates.
- [SCOPE-07] Adding new planner actions.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define an explicit authored contract for planner action names, JSON examples, required fields, shared rules, and prompt variants. | SCOPE-01 | FR-01 | automated test |
| SRS-02 | Render canonical schema blocks for initial action, recursive next action, retry, and redecision prompts. | SCOPE-02 | FR-01 | automated test |
| SRS-03 | Cover all current action variants: `answer`, workspace actions, semantic actions, `external_capability`, `refine`, `branch`, and `stop`. | SCOPE-01, SCOPE-04 | FR-02 | automated test |
| SRS-04 | Keep turn-specific availability in `PlannerExecutionContract`; the schema renderer defines vocabulary and shape only. | SCOPE-02 | FR-03 | code review + automated test |
| SRS-05 | Add enum/schema parity tests that fail when Rust action variants drift from the authored schema. | SCOPE-03 | FR-02 | automated test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Place the shared renderer outside provider adapters so every planner lane can reuse it. | SCOPE-01, SCOPE-02 | NFR-02 | code review |
| SRS-NFR-02 | Make test failures identify missing or extra schema actions clearly. | SCOPE-03 | NFR-03 | automated test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
