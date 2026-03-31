# Foundational Chord Integration - Product Requirements

## Problem Statement

The paddles assistant lacks its primary 'chord' capabilities for agentic coding tasks. Currently, the `paddles` binary is a thin wrapper that doesn't utilize the `legacy-engine` crate's capabilities, which are essential for its role as a "mech suit" for AI assistants.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Integrate `legacy-engine` (chord) into the paddles CLI. | `paddles --prompt` executes a chord-powered task. | 100% |
| GOAL-02 | Establish foundational agentic coding workflow. | Chord can successfully modify a file in a controlled test. | 100% |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Developer | Using paddles to automate coding tasks. | High-fidelity execution of coding prompts via chord. |

## Scope

### In Scope

- [SCOPE-01] Wiring `legacy-engine` into `src/main.rs`.
- [SCOPE-02] Basic prompt handling via chord.
- [SCOPE-03] Verification test for a simple chord-driven file modification.
- [SCOPE-06] Integration of `legacy-core` `Instance` and `PromptLoop`.
- [SCOPE-07] Successful compilation with real dependencies.
- [SCOPE-08] Execution of a real agentic prompt via the CLI.

### Out of Scope

- [SCOPE-04] Advanced multi-step reasoning or complex tool use.
- [SCOPE-05] Integration with `sift` for advanced retrieval (future epic).
- [SCOPE-09] Advanced TUI features from `legacy-engine`.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `paddles --prompt <text>` should delegate to chord for execution. | GOAL-01 | must | Core functionality of the mech suit. |
| FR-02 | Chord must be able to read and write files within the project. | GOAL-02 | must | Necessary for coding tasks. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Chord execution must be observable via tracing. | GOAL-01 | must | Essential for debugging agentic behavior. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Automated build verification via `cargo build`.
- Manual verification of prompt execution using the `paddles` binary.
- Tracing logs review for agentic step verification.

## Assumptions

| Assumption | Rationale |
|------------|-----------|
| A-01 | The `legacy-core` API is stable enough for initial wiring. | Necessary for building against the crate. |

## Open Questions & Risks

| ID | Question/Risk | Mitigation |
|----|---------------|------------|
| R-01 | OpenSSL dependency issues in Nix environment. | Updated `flake.nix` to include `openssl` and `pkg-config`. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles --prompt "test"` initiates a `legacy-engine` session.
- [ ] The system compiles and runs without mock simulations.
<!-- END SUCCESS_CRITERIA -->
