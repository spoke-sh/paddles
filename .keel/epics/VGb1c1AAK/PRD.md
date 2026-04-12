# Turn-Scoped Control API And Runtime Events - Product Requirements

## Problem Statement

Paddles has recursive execution and thread lineage, but it still lacks a first-class turn control plane for same-turn steering, interruption, fork/resume semantics, and streamed plan/diff state.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Model turn and thread lifecycle operations as a first-class runtime contract. | Turn, thread, and control operations exist as typed events and commands rather than prompt-only conventions. | The harness exposes a durable control plane instead of queued follow-up input. |
| GOAL-02 | Support same-turn steering, interruption, resume/fork/rollback semantics, and related thread control flows. | Operators and surfaces can steer a running turn without losing lineage or replayability. | Active-turn intervention becomes intentional and inspectable. |
| GOAL-03 | Stream live plan, diff, and control state to all operator surfaces. | TUI, web, and API consumers can observe plan updates, file changes, and control transitions in real time. | A running turn is legible without reading raw trace internals. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive Operator | A user steering Paddles while the harness is mid-turn. | The ability to interrupt or redirect work without waiting for a full turn boundary. |
| Surface Integrator | A maintainer building TUI, web, or HTTP/UI experiences on top of the harness. | A stable control contract that does not require surface-specific control semantics. |
| Runtime Maintainer | A contributor evolving session and replay behavior. | Durable, typed lifecycle state instead of one-off control branches. |

## Scope

### In Scope

- [SCOPE-01] Define typed turn and thread control operations such as start, steer, interrupt, fork/resume, and rollback/archive equivalents.
- [SCOPE-02] Route same-turn steering and interruption through replayable control records instead of only queued prompts.
- [SCOPE-03] Emit typed plan, diff, command, file-change, and control-state updates during active turns.
- [SCOPE-04] Expose a control surface that TUI, web, and API layers can consume consistently.
- [SCOPE-05] Update docs and proofs around the resulting turn/runtime contract.

### Out of Scope

- [SCOPE-06] A hosted multi-tenant app-server or remote orchestration service.
- [SCOPE-07] Arbitrary cross-machine collaboration or synchronization beyond one Paddles runtime.
- [SCOPE-08] Full multi-agent delegation semantics; that belongs to the multi-agent mission.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must expose typed turn and thread control operations rather than relying on surface-specific prompt conventions. | GOAL-01 | must | A control plane needs stable semantics independent of UI phrasing. |
| FR-02 | Active-turn steering and interruption must become replayable control events with bounded fallback when a requested control action cannot apply. | GOAL-01, GOAL-02 | must | Same-turn intervention is one of the largest current behavior gaps. |
| FR-03 | The control plane must support durable lineage for fork/resume and rollback/archive-style turn transitions. | GOAL-01, GOAL-02 | should | Surface-visible control is only trustworthy if it survives replay. |
| FR-04 | Plan updates, diff updates, command execution summaries, and file-change artifacts must be emitted as typed runtime items during active turns. | GOAL-03 | must | Operators need structured visibility into work-in-progress state. |
| FR-05 | TUI, web, and API surfaces must be able to render the same control and runtime item vocabulary without inventing divergent semantics. | GOAL-02, GOAL-03 | must | The control plane should unify surfaces, not fork them. |
| FR-06 | Invalid or stale control requests must degrade honestly with explicit status instead of mutating hidden thread state. | GOAL-02 | must | Interruptibility must not sacrifice trustworthiness. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Build on the existing recorder and replay model instead of inventing a parallel state store. | GOAL-01, GOAL-02 | must | The current lineage investment is the right foundation for control-plane work. |
| NFR-02 | Control transitions and live runtime items must remain readable enough for the default transcript and UI projections. | GOAL-03 | must | More control should not turn the harness into raw event soup. |
| NFR-03 | Control semantics must stay deterministic enough for focused tests and replay proofs. | GOAL-01, GOAL-02 | must | Interruptibility without reproducibility would be fragile. |
| NFR-04 | The control plane must preserve the local-first recursive execution model. | GOAL-01, GOAL-02, GOAL-03 | must | This mission should expose the harness, not replace it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Turn/thread control contract | Unit tests and protocol-level runtime tests | Story-level verification artifacts and command logs |
| Same-turn steering and interruption | Transcript proofs, replay checks, and targeted integration tests | Story-level transcript captures and replay outputs |
| Plan/diff/control visibility | TUI/web/API rendering proofs over shared runtime items | Story-level UI proofs and doc updates |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing thread-lineage and replay foundations are sufficient to host richer turn-control semantics. | The runtime may need a deeper session refactor before interruptibility becomes reliable. | Validate control records against current replay and projection layers during decomposition. |
| A shared runtime item vocabulary can serve TUI, web, and API surfaces without becoming too generic to be useful. | Surface-specific control branches may still creep back in. | Prototype item rendering in at least two surfaces during voyage planning. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which rollback/archive semantics belong in the first release versus later recovery-focused work? | Epic owner | Open |
| How aggressively should the harness allow interruption during unsafe execution windows such as in-flight edits or commands? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Turn and thread control operations exist as typed, replayable runtime semantics.
- [ ] Operators can steer or interrupt active turns without falling back to opaque queued prompts.
- [ ] Plan and diff state are available as first-class live runtime items across surfaces.
- [ ] Docs and proofs explain the control-plane behavior and its degradation rules honestly.
<!-- END SUCCESS_CRITERIA -->
