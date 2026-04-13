# Establish A Typed Collaboration Mode And Review Substrate - SRS

## Summary

Epic: VGb1c1pAR
Goal: Establish typed collaboration mode state, a findings-first review lane, and bounded structured clarification on top of the recursive harness.

## Scope

### In Scope

- [SCOPE-01] Define typed collaboration modes for planning, execution, and review as runtime state rather than prompt-only conventions.
- [SCOPE-02] Gate prompting, mutation permissions, and output expectations through the selected collaboration mode.
- [SCOPE-03] Add bounded structured user-input requests and responses for planning or approval workflows that genuinely require clarification.
- [SCOPE-04] Introduce a first-class review workflow that inspects local changes and emits findings-first output with grounded file references.
- [SCOPE-05] Project mode transitions, structured clarification exchanges, and review findings through trace, transcript, UI, API, and docs.

### Out of Scope

- [SCOPE-06] Persona-only wording variants that do not materially change workflow behavior.
- [SCOPE-07] Hosted pull-request, ticketing, or external review-system integrations beyond local diff/worktree inspection.
- [SCOPE-08] Multi-agent delegation semantics or external-capability breadth beyond what later missions add.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define typed collaboration mode state and mode-request semantics that can influence prompting, permission posture, and output expectations without depending on any one surface's wording. | SCOPE-01, SCOPE-02 | FR-01 | manual |
| SRS-02 | Planning mode must support non-mutating exploration plus bounded structured clarification when the runtime genuinely needs user input. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-02 | manual |
| SRS-03 | Review mode must inspect local changes and emit findings-first output with grounded file or line references, followed by residual risks or gaps when needed. | SCOPE-01, SCOPE-02, SCOPE-04 | FR-03 | manual |
| SRS-04 | Execution mode must remain the default mutation path while honoring mode-specific permissions, escalation rules, and fail-closed restrictions. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-04 | manual |
| SRS-05 | Mode entry, exit, and any structured clarification exchanges must be visible in runtime traces and operator-facing surfaces. | SCOPE-03, SCOPE-05 | FR-05 | manual |
| SRS-06 | Invalid or unavailable mode requests must degrade honestly rather than silently reverting to default execution behavior. | SCOPE-01, SCOPE-02, SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Mode semantics must remain concise enough that operators can predict behavior without reading large prompts. | SCOPE-01, SCOPE-02, SCOPE-05 | NFR-01 | manual |
| SRS-NFR-02 | Review findings and structured clarification requests must remain auditable through replay and transcript projections. | SCOPE-04, SCOPE-05 | NFR-02 | manual |
| SRS-NFR-03 | Planning and review modes must preserve the same recursive harness identity and evidence standards as execution mode. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-03 | manual |
| SRS-NFR-04 | Any mode-specific mutation restrictions must fail closed. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
