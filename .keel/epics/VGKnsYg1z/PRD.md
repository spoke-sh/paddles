# Add Native Transport Connection Stack - Product Requirements

## Problem Statement

Paddles currently communicates through a narrow set of runtime-specific surfaces, so it cannot expose first-class native transport connections for stateless HTTP request/response, server-sent events, WebSockets, and Transit without ad hoc adapter work or duplicated protocol logic.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Introduce a shared native transport layer that can serve stateless HTTP request/response, streaming SSE, bidirectional WebSockets, and Transit without duplicating protocol logic across runtime surfaces. | All four named transports are represented by one shared connection model and have bounded adapter implementations. | The stack can expose the four transports through repo-owned contracts and transport-specific acceptance tests. |
| GOAL-02 | Preserve runtime observability and operator control while expanding transport reach. | Transport state, failures, and negotiated capabilities are visible through existing traces, diagnostics, and configuration surfaces. | Operators can inspect transport behavior without diving into bespoke adapter internals. |
| GOAL-03 | Keep the rollout local-first and incremental. | The first transport slice does not require hosted transport dependencies or product-wide redesign of the current UI/runtime loop. | Native transport support lands as an additive capability inside the current repo/runtime boundaries. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | Operators and integrators who need to connect Paddles through different native transport modes depending on local runtime constraints and embedding context. | A consistent way to speak to Paddles over HTTP request/response, SSE, WebSockets, or Transit without protocol-specific surprises. |
| Secondary User | Maintainers extending runtime communication, diagnostics, and connection surfaces inside the repo. | One shared transport contract that keeps lifecycle, auth, framing, and tracing coherent across transports. |

## Scope

### In Scope

- [SCOPE-01] Define a shared native transport model covering connection identity, capability negotiation, session lifecycle, transport errors, and observability.
- [SCOPE-02] Add stateless HTTP request/response support on top of that shared model.
- [SCOPE-03] Add SSE streaming support for transport surfaces that need server-push turn events.
- [SCOPE-04] Add WebSocket support for bidirectional native sessions.
- [SCOPE-05] Add Transit-native support using the same shared contracts and lifecycle semantics.
- [SCOPE-06] Update docs, diagnostics, tests, and board-linked verification so transport behavior is explicit and guarded.

### Out of Scope

- [SCOPE-07] Introducing unrelated protocols such as gRPC, MQTT, or external message brokers.
- [SCOPE-08] Redesigning the core planner/runtime semantics beyond what transport integration requires.
- [SCOPE-09] Changing product defaults for transport exposure without an explicit human decision.
- [SCOPE-10] Hosted transport infrastructure or IDE-fed connection state.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must define a shared transport contract that normalizes connection lifecycle, capability negotiation, session identity, and transport error semantics across the named native transports. | GOAL-01, GOAL-02, GOAL-03 | must | The transport stack cannot stay coherent if each adapter reinvents its own lifecycle and diagnostics model. |
| FR-02 | Paddles must support a stateless HTTP request/response transport that can execute bounded turns through the shared transport contract. | GOAL-01, GOAL-03 | must | HTTP request/response is the lowest-friction transport for simple integrations and compatibility surfaces. |
| FR-03 | Paddles must support SSE transport for streaming turn progress and output over the shared transport contract. | GOAL-01, GOAL-02 | must | Operators need a native streaming path without forcing WebSockets where one-way server push is sufficient. |
| FR-04 | Paddles must support WebSocket transport for bidirectional native sessions, reusing shared lifecycle, auth, and observability semantics. | GOAL-01, GOAL-02 | must | Bidirectional sessions need a first-class native path instead of one-off adapter logic. |
| FR-05 | Paddles must support a Transit-native transport path using the same shared transport semantics and trace visibility. | GOAL-01, GOAL-02, GOAL-03 | must | Transit is a named product requirement for the communication expansion and should not be treated as a special-case afterthought. |
| FR-06 | Transport configuration and diagnostics must make it clear which transports are available, enabled, negotiated, or failing in a given runtime. | GOAL-02, GOAL-03 | should | Operators need to understand transport state without reading implementation-specific logs. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new transport paths introduced by this epic. | GOAL-01, GOAL-02 | must | The transport expansion should widen access without making runtime failures opaque. |
| NFR-02 | Keep the transport additions local-first and repo-owned, with no required hosted infrastructure or out-of-repo coordination services. | GOAL-03 | must | This preserves the repo’s collaboration and runtime philosophy. |
| NFR-03 | Guard the shared transport contracts with focused tests and story evidence so regressions do not fragment the transport stack over time. | GOAL-01, GOAL-02, GOAL-03 | must | The new communication layer needs explicit guardrails to stay coherent across future additions. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Shared transport model | Contract tests, configuration/diagnostic checks, and board-authored requirements review | Story-level verification artifacts and voyage compliance reports |
| Transport adapters | Focused end-to-end tests per transport plus runtime evidence capture | Story-level verification artifacts and implementation logs |
| Operator visibility | Manual review and UI/diagnostic proofs for transport state and failure reporting | Story-level verification artifacts and updated docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing runtime boundaries can absorb a shared transport layer without a deep architectural rewrite of planner/controller semantics. | The epic may need a deeper foundation slice before transport adapters can land safely. | Voyage one validates the transport model and lifecycle contract first. |
| The four requested transports can share enough connection semantics to justify one common contract. | The epic may need to split into protocol-specific stacks with more duplication than desired. | Voyage one defines and tests the shared contract before adapter-heavy voyages proceed. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much transport configuration should be operator-facing by default versus advanced/internals-only? | Epic owner | Planned in voyage one |
| Does Transit need a distinct framing or auth story that strains the shared contract? | Epic owner | Planned in voyage three |
| Which transport should anchor the default integration documentation first? | Epic owner | Planned in voyage two |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles exposes one shared native transport model that underpins HTTP request/response, SSE, WebSocket, and Transit communication.
- [ ] The four named transports are covered by repo-owned stories, tests, and verification artifacts instead of ad hoc adapter work.
- [ ] Operators can understand transport availability, negotiation, and failure state through existing diagnostics and trace surfaces.
<!-- END SUCCESS_CRITERIA -->
