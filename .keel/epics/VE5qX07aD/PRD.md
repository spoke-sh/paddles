# Lattice Refactor - Product Requirements

## Problem Statement

The current `paddles` codebase is a flat `main.rs`. While functional for initial capacity proofs, it lacks the structure required for long-term expansion (multiple providers, complex tool use, advanced retrieval). We need to move to a Domain-Driven Design (DDD) and Hexagonal Architecture to separate core logic from infrastructure concerns.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Separate Domain logic from Infrastructure | Domain crate/module has zero dependencies on CLI/Third-party libs (except candle-core) | 100% |
| GOAL-02 | Establish Ports and Adapters | External dependencies (like Candle) are hidden behind traits | 100% |
| GOAL-03 | Maintain Zero Functional Regression | `just paddles` and `--prompt` behave identically to pre-refactor state | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Extending Paddles with new features | A clean, modular codebase where logic is easy to find and test |

## Scope

### In Scope

- [SCOPE-01] Defining the `Domain` layer (Entities: `Session`, `BootContext`; Value Objects: `Weights`, `Bias`, `Dogma`).
- [SCOPE-02] Defining the `Ports` (Interfaces for `LanguageModel`).
- [SCOPE-03] Implementing the `Adapters` (Concrete `CandleAdapter`).
- [SCOPE-04] Defining the `Application` layer (Use cases: `BootSystem`, `ProcessPrompt`).
- [SCOPE-05] Defining the `Infrastructure` layer (CLI parsing, Config).

### Out of Scope

- [SCOPE-06] Adding new features or providers.
- [SCOPE-07] Multi-crate workspace split (sticking to modules within one crate for now).

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | All boot logic must reside in the Domain layer. | GOAL-01 | must | Core business logic belongs in the domain. |
| FR-02 | The interaction with Candle must be through a Port trait. | GOAL-02 | must | Decouples domain from specific inference engines. |
| FR-03 | CLI functionality must remain identical after refactor. | GOAL-03 | must | Ensures zero regression. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Domain layer must be testable in isolation. | GOAL-01 | must | Key benefit of DDD. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Automated tests for the Domain layer.
- CLI smoke tests to ensure parity.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| Module-based separation is sufficient for now. | Workspace split is overkill for the current size. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | Complexity overhead | Keep the initial structure as simple as possible. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Code is organized into `domain`, `application`, `infrastructure` modules.
- [ ] CLI remains fully functional with all existing flags.
<!-- END SUCCESS_CRITERIA -->
