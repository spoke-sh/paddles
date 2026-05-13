# Agent Vocabulary Cleanup - SRS

## Summary

Epic: VJXwbmekZ
Goal: Update prompts, tests, and foundational docs so planning is described as
model reasoning through bounded recursive agent actions, not as a separate
architecture phase.

## Scope

### In Scope

- [SCOPE-01] Adapter prompt vocabulary for Sift/local and HTTP/remote lanes.
- [SCOPE-02] Tests that prevent reintroducing pre-loop initial-routing language.
- [SCOPE-03] Foundational docs: README, POLICY, ARCHITECTURE, CONFIGURATION, and owning docs as needed.

### Out of Scope

- [SCOPE-04] Runtime behavior changes; handled by VJXwlE718.
- [SCOPE-05] Generated docs site redesign.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Planner-lane prompts must describe the model as selecting bounded recursive agent actions inside the loop, including the first action. | SCOPE-01 | FR-06 | cargo test |
| SRS-02 | Prompt and schema tests must fail if Sift or HTTP reintroduce adapter-local first-action/recursive-action schema drift. | SCOPE-01, SCOPE-02 | FR-05 | cargo test |
| SRS-03 | Foundational docs must state that model reasoning is the planning and that planning is not a separate architecture phase outside the agent loop. | SCOPE-03 | FR-06 | rg |
| SRS-04 | Docs must describe terminal `answer`/`stop`, workspace actions, semantic actions, and `external_capability` as one recursive action vocabulary gated by the capability manifest. | SCOPE-03 | FR-06 | rg |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Documentation and prompt wording must not imply a final-answer-only purpose or remote-only schema disclosure. | SCOPE-01, SCOPE-03 | NFR-02 | rg |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
