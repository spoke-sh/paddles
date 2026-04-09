# Reframe Trace Surfaces As A Narrative Machine - Product Requirements

## Problem Statement

The current transit trace inspector and forensic inspector expose raw trace structure instead of a coherent causal story, so operators see too many records, modes, and filters without a simple narrative of how a turn moved through planning, steering, tools, and outcome.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make the default trace surfaces explain a turn as one causal machine instead of a collection of raw records and controls. | The primary transit/forensic view can be understood through one stage, one scrubber, and one detail surface. | The default operator path no longer requires parallel nav/list/detail panes or family/filter toggles to understand a turn. |
| GOAL-02 | Preserve the ability to inspect trace internals without forcing every operator through raw records first. | Raw payloads, record ids, and trace-node fidelity remain reachable through an explicit internals path. | Simplification does not remove access to underlying evidence. |
| GOAL-03 | Align transit and forensic surfaces around one shared vocabulary and interaction model. | Both routes use the same “machine moment” framing, selection behavior, and temporal navigation model. | Operators can move between transit and forensic without learning a second UI grammar. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | Operators and maintainers debugging a single turn after it ran through planner, steering, tools, and synthesis. | A coherent narrative of what happened, why it changed direction, and where it ended. |
| Secondary User | Maintainers evolving the runtime trace UI and trace projection contracts. | A simpler surface model that still preserves access to raw trace evidence and stable contracts. |

## Scope

### In Scope

- [SCOPE-01] Define a shared “machine moment” projection and vocabulary that maps trace graph steps plus forensic records into one causal timeline.
- [SCOPE-02] Rebuild the transit route around a simple machine stage, bottom scrubber, and focused moment detail surface.
- [SCOPE-03] Rebuild the forensic route around the same machine narrative with an optional internals path for raw payloads and record metadata.
- [SCOPE-04] Remove or reduce duplicate chrome, parallel selectors, and low-signal cards that distract from the causal story.
- [SCOPE-05] Update tests and documentation so the new narrative-machine contract is explicit and guarded.

### Out of Scope

- [SCOPE-06] Redesigning the steering gate manifold.
- [SCOPE-07] Changing the underlying trace generation model, planner semantics, or forensic recorder behavior beyond what is needed to project machine moments.
- [SCOPE-08] Removing raw trace access entirely.
- [SCOPE-09] General web UI redesign work unrelated to transit/forensic trace comprehension.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must expose a shared machine-moment projection that groups raw transit and forensic artifacts into operator-meaningful moments such as planning, evidence, steering, tool execution, jams, replans, and outputs. | GOAL-01, GOAL-03 | must | The UI cannot become coherent while raw records remain the first-class visual object. |
| FR-02 | The transit route must render the turn as a simple machine stage with temporal navigation, selected-moment detail, and visual treatment for diverters, jams, and outputs. | GOAL-01, GOAL-03 | must | Transit is currently the most visibly overloaded causal surface. |
| FR-03 | The forensic route must present the same machine narrative and keep raw payloads behind an explicit internals path rather than parallel always-on panes. | GOAL-01, GOAL-02, GOAL-03 | must | The forensic inspector should explain the turn first and expose record internals second. |
| FR-04 | Legacy chrome that duplicates the same narrative in multiple places must be removed or demoted when the new machine stage already carries that information. | GOAL-01 | should | Simplicity only lands if the older parallel surfaces stop competing with the main story. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve stable trace evidence links from each machine moment back to the underlying node ids, record ids, and payloads. | GOAL-02 | must | Narrative simplification cannot sever the debugging chain back to the source evidence. |
| NFR-02 | Keep the primary operator path visually legible without requiring domain-specific trace terminology or multiple simultaneous control clusters. | GOAL-01, GOAL-03 | must | The redesign is specifically about improving coherence and comprehension. |
| NFR-03 | Guard the new machine narrative with focused route tests and projection contracts so future changes do not regress back into raw-record sprawl. | GOAL-01, GOAL-02, GOAL-03 | must | The simpler model must be explicit and enforceable. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Narrative coherence | Route-level tests, projection-contract tests, and focused manual review of representative turns | Story-level verification artifacts and voyage compliance reports |
| Internals preservation | Selector/projection tests plus manual verification of internals drill-down | Story-level verification artifacts and detail-surface proofs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current trace graph and forensic projections already contain enough structure to derive meaningful machine moments without redesigning the recorder layer. | The epic may need deeper backend changes before the UI can simplify. | Voyage one validates the shared projection contract first. |
| Operators still need raw/internal trace access, but not as the default first surface. | The redesign could either hide too much or keep too much chrome. | Voyage three defines an explicit internals mode and evaluates the default path. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much of the current transit family/scope filtering survives as internals-only controls versus disappearing entirely? | Epic owner | Planned in voyage two |
| Which forensic comparisons remain essential in the default path versus detail drawer only? | Epic owner | Planned in voyage three |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Transit and forensic routes share one machine-moment vocabulary and temporal selection model.
- [ ] The default operator path for understanding a turn is stage → scrubber → moment detail, not nav → list → raw detail.
- [ ] Raw trace payloads and record metadata remain reachable through an explicit internals path instead of being the default information hierarchy.
<!-- END SUCCESS_CRITERIA -->
