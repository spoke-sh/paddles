# Mode-Aware Planning And Review Workflows - Product Requirements

## Problem Statement

Paddles runs one dominant recursive interaction style today, so it cannot switch cleanly between execution, planning, and review behaviors or ask the user for structured clarification inside a bounded workflow.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Introduce explicit collaboration modes for planning, execution, and review without splitting the recursive harness into separate products. | Mode choice changes runtime behavior, permissions, and prompts in a typed, replayable way. | Operators can intentionally choose how Paddles should behave before work begins. |
| GOAL-02 | Create a first-class review workflow that is findings-first, diff-aware, and grounded in concrete evidence. | Review mode consistently emits actionable findings with file references and separates findings from summaries. | "Review" becomes a real workflow rather than a prompt suggestion. |
| GOAL-03 | Add bounded structured user-input requests for workflows that genuinely require clarification. | Plan-oriented or approval-oriented turns can ask concise, typed questions instead of falling back to vague conversational probing. | The harness can stay autonomous without becoming presumptuous. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive Operator | A user who sometimes wants direct execution and other times wants planning or review. | A predictable way to tell the harness which mode of work is desired. |
| Reviewer | A maintainer asking Paddles to inspect a diff, worktree, or change set. | High-signal findings, references, and residual-risk callouts instead of generic summaries. |
| Runtime Maintainer | A contributor formalizing behavioral contracts. | Explicit mode semantics that are testable and replayable. |

## Scope

### In Scope

- [SCOPE-01] Define mode semantics for at least planning, execution, and review workflows.
- [SCOPE-02] Gate behavior, mutation permissions, and output contracts through the selected mode.
- [SCOPE-03] Add a review workflow that prioritizes findings with file/line grounding and explicit residual risks.
- [SCOPE-04] Add bounded structured user-input requests for workflows that require clarification or approval.
- [SCOPE-05] Surface mode transitions and mode-specific constraints in traces and operator-facing UIs.

### Out of Scope

- [SCOPE-06] Persona-only stylistic variations that do not materially change workflow semantics.
- [SCOPE-07] Hosted ticketing or PR-system features beyond what review mode needs to inspect changes locally.
- [SCOPE-08] Multi-agent delegation semantics beyond what is necessary to keep mode behavior coherent.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must model collaboration mode as typed state that can influence prompting, permissions, and output expectations. | GOAL-01 | must | Modes need structural effect, not just labels. |
| FR-02 | Planning mode must support non-mutating exploration plus bounded structured clarification when the runtime genuinely needs user input. | GOAL-01, GOAL-03 | must | Planning should feel different from execution in both behavior and safety. |
| FR-03 | Review mode must inspect local changes and emit findings-first output with file/line references, followed by residual risks or gaps when needed. | GOAL-02 | must | Codex-like review quality depends on a dedicated output contract. |
| FR-04 | Execution mode must remain the default mutation path while honoring mode-specific permissions and escalation rules. | GOAL-01, GOAL-03 | must | Modes should guide execution, not break it. |
| FR-05 | Mode entry, exit, and any structured user-input exchanges must be visible in runtime traces and operator-facing surfaces. | GOAL-01, GOAL-03 | should | Replay should explain why the harness paused or changed stance. |
| FR-06 | Invalid or unavailable mode requests must degrade honestly rather than silently reverting to default execution behavior. | GOAL-01 | must | Hidden fallback would make modes untrustworthy. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Mode semantics must remain concise enough that operators can predict behavior without reading large prompts. | GOAL-01, GOAL-02, GOAL-03 | must | Explicit modes should reduce confusion, not add it. |
| NFR-02 | Review findings and structured user-input requests must be auditable through replay and transcript projections. | GOAL-02, GOAL-03 | must | Review quality depends on traceability. |
| NFR-03 | Planning and review modes must preserve the same recursive harness identity and evidence standards as execution mode. | GOAL-01, GOAL-02 | must | Mode awareness should steer behavior, not fragment the architecture. |
| NFR-04 | Any mode-specific mutation restrictions must fail closed. | GOAL-01, GOAL-03 | must | The safe posture of planning/review modes should be structural. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Mode state and gating | Unit tests and runtime contract tests | Story-level verification artifacts and command logs |
| Review workflow | Local-diff review tests and findings-oriented transcript proofs | Story-level review captures and file-reference checks |
| Structured user input | Integration tests over bounded request/response flows and UI traces | Story-level verification artifacts and projection proofs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Explicit mode semantics will make behavior more legible than one highly adaptive but opaque default path. | The harness may need a smaller set of modes or different naming. | Validate through transcript proofs and operator-facing docs during decomposition. |
| Review quality improves materially when output structure is enforced at the workflow level. | Review mode may still need stronger evidence or diff-selection primitives. | Exercise review on real local changes early in the first voyage. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which structured question shapes are worth supporting first: approvals, plan clarification, or environment selection? | Epic owner | Open |
| How strict should planning mode be about mutation bans when the user clearly asks for implementation mid-plan? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles exposes explicit planning, execution, and review modes with typed behavioral effects.
- [ ] Review mode produces findings-first output with grounded file references and residual-risk notes.
- [ ] Structured user-input requests exist for workflows that need clarification or approval.
- [ ] Mode changes and restrictions are visible in traces, docs, and operator-facing surfaces.
<!-- END SUCCESS_CRITERIA -->
