# Hosted Transit Authority And External Service Contract - Product Requirements

## Problem Statement

Paddles still treats Transit primarily as a web transport plus embedded local recorder, so downstream services cannot use hosted Transit as the authoritative first-party service boundary for turn submission, replay-derived projection state, restart resume, and consumer-facing transcript restore.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Hosted Transit authority: deployed Paddles can run against hosted Transit through `transit-client`, with hosted Transit as the authoritative persistence and replay substrate rather than embedded local `transit-core`. | A deployed runtime can boot, accept work, and resume using hosted Transit without requiring embedded local storage | Voyage 1 and Voyage 3 |
| GOAL-02 | Stable external Transit contract: downstream services can submit turns and observe lifecycle/projection state over versioned Transit streams instead of treating Paddles HTTP endpoints as the canonical boundary. | One versioned contract covers bootstrap, turn submission, progress, projection rebuilds, completion/failure, and restore | Voyage 2 |
| GOAL-03 | Hosted resume semantics: session and projection views resume from hosted consumer cursors and hosted materialization checkpoints without losing or duplicating work after restart. | Restart tests show deterministic resume from hosted cursor/checkpoint state with no duplicated turn effects | Voyage 3 |
| GOAL-04 | Service-oriented deployment path: Paddles exposes a stable non-interactive first-party service mode with explicit readiness/failure reporting while retaining HTTP UI/debug surfaces only as optional operator tools. | Operators can configure and run a long-lived hosted service mode without depending on interactive or web-only control paths | Voyage 1 |
| GOAL-05 | Consumer-facing projection surface: Paddles emits a typed projection payload carrying transcript rows, turn status, replay revision metadata, trace/manifold availability, and external provenance needed for deterministic restore. | A downstream consumer can render and restore from the projection contract without scraping HTTP UI state | Voyage 2 and Voyage 3 |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Hosted Service Integrator | Engineer deploying Paddles as a first-party hosted service for a downstream platform. | A stable hosted Transit contract and runtime mode that fit the deployment bus and operational model. |
| Paddles Runtime Maintainer | Engineer responsible for recorder, replay, session, and projection architecture. | Clear authority boundaries, restart semantics, and tests that preserve replay-derived truth. |
| Projection Consumer | Engineer or UI surface that needs transcript/detail projections from Paddles. | A typed projection payload with enough provenance and revision data to render and restore deterministically. |

## Scope

### In Scope

- [SCOPE-01] Hosted Transit authority mode using `transit-client` for authoritative append/read/projection operations in deployed service mode
- [SCOPE-02] Explicit service-mode configuration for Transit endpoint, namespace, service identity, readiness, and failure reporting
- [SCOPE-03] Versioned external Transit stream contract for bootstrap, turn submission, turn progress, projection rebuilds, completion/failure, and restore
- [SCOPE-04] Provenance envelopes carrying account, session, workspace, route, request, and workspace-posture identity through command and projection flows
- [SCOPE-05] Hosted consumer cursor and hosted materialization checkpoint/resume semantics for session and projection resumption
- [SCOPE-06] Consumer-facing projection payloads derived from authoritative Transit history
- [SCOPE-07] Documentation, ADR, configuration guidance, and contract tests for the new authority boundary and service path

### Out of Scope

- [SCOPE-08] Downstream deployment automation, catalog, or environment wiring
- [SCOPE-09] Downstream consumer frontend implementation details or UI presentation wiring
- [SCOPE-10] External auth materializer changes for accounts or sessions
- [SCOPE-11] Replacing optional HTTP UI/debug/operator surfaces where they remain useful for local or manual inspection
- [SCOPE-12] Non-Transit integration surfaces as the primary first-party deployment contract

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must expose a hosted Transit authority mode that uses hosted Transit through `transit-client` as the authoritative persistence, replay, and integration boundary for deployed service mode. | GOAL-01, GOAL-04 | must | This is the core architectural change requested by the downstream integration. |
| FR-02 | Embedded local `transit-core` and in-memory recorders must remain available only as explicit local/dev fallbacks and must not be required for the production first-party deployment path. | GOAL-01, GOAL-04 | must | Deployed hosted integration must not depend on local embedded storage. |
| FR-03 | Paddles must define a stable, versioned external Transit contract that covers session bootstrap, turn submission, turn progress, projection rebuilds, completion/failure, and session restore. | GOAL-02, GOAL-05 | must | Downstream consumers need a canonical contract that is not derived from web endpoints. |
| FR-04 | The Transit contract must carry explicit provenance fields for account, session, workspace, route, request identity, and workspace posture so downstream consumers can correlate Paddles state without delegating auth ownership to Paddles. | GOAL-02, GOAL-05 | must | Provenance is necessary for deterministic correlation and restore. |
| FR-05 | Paddles must emit a typed projection payload over Transit that is sufficient for consumer transcript/detail rendering, including transcript rows, turn status, replay revision metadata, trace/manifold availability, and restore identity. | GOAL-02, GOAL-05 | must | Downstream consumers need a stable projection shape rather than shell or channel-specific state. |
| FR-06 | Session and projection consumers must resume from hosted consumer cursors and hosted materialization checkpoints instead of depending on full replay or local-only recorder state on every restart. | GOAL-01, GOAL-03 | must | Hosted resume primitives are a central part of the requested deployment model. |
| FR-07 | Restart resume must avoid losing, reordering, or duplicating turn work and must preserve replay-derived reproducibility from authoritative Transit history. | GOAL-03, GOAL-05 | must | Resume correctness matters more than raw startup speed. |
| FR-08 | Paddles must provide a stable non-interactive service mode suitable for long-lived deployment, with explicit configuration for hosted Transit connectivity and operational readiness/failure reporting. | GOAL-01, GOAL-04 | must | First-party deployment requires a service-oriented runtime shape. |
| FR-09 | Existing HTTP Transit/web surfaces may remain for debug or operator use, but downstream integration must not depend on them as the canonical control plane or replay surface. | GOAL-02, GOAL-04 | must | Preserves useful local surfaces without confusing the production integration boundary. |
| FR-10 | Configuration guidance, ADRs, and automated contract tests must describe the hosted authority mode, stream contract, provenance envelope, projection payloads, and resume semantics. | GOAL-01, GOAL-02, GOAL-03, GOAL-04, GOAL-05 | must | The integration is architectural and operational; it needs explicit documentation and guardrails. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Projection state must remain replay-derived and reproducible from authoritative Transit history even when checkpoints and cursors are used to accelerate resume. | GOAL-01, GOAL-03, GOAL-05 | must | Resume optimizations must not create a second source of truth. |
| NFR-02 | Hosted and local recorder modes must have explicit authority boundaries; the runtime must not open embedded local storage as a second authority for the same hosted workload. | GOAL-01, GOAL-04 | must | Dual authority would corrupt replay assumptions and operational safety. |
| NFR-03 | The stream contract and projection payloads must be versioned and backwards-disciplined so downstream integrations can evolve without scraping incidental runtime details. | GOAL-02, GOAL-05 | must | This boundary is intended to be long-lived and consumable by other systems. |
| NFR-04 | Service mode must expose readiness and failure states that operators can observe without attaching the TUI or relying on manual browser inspection. | GOAL-01, GOAL-04 | must | Deployed service mode needs first-class operational signals. |
| NFR-05 | Hosted resume and projection updates must be testable through deterministic contract and restart scenarios that cover no-loss/no-duplication behavior. | GOAL-03, GOAL-05 | must | Restart correctness is a primary downstream acceptance criterion. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Hosted authority mode | Adapter/service integration tests, config proofs, and operator review | Story-level tests plus config/run evidence |
| Transit contract | Contract tests for command, event, and projection envelopes | Story-level tests and contract snapshots |
| Resume semantics | Restart/resume tests using hosted cursors and materialization checkpoints | Story-level deterministic resume evidence |
| Projection fidelity | Projection tests and replay comparison against authoritative history | Story-level replay/projection verification |
| Docs and operational guidance | Review of ADR, config docs, and runtime guidance | Story-linked documentation diffs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Hosted Transit endpoints, namespaces, cursor APIs, and hosted materialization primitives are stable enough to serve as the first-party authority boundary for Paddles. | The service-mode design may need deeper abstraction or a staged rollout. | Validate in Voyage 1 and Voyage 3. |
| Downstream services can consume a versioned Transit contract more safely than today’s HTTP/web-specific surfaces. | The contract may need additional compatibility or payload shape work. | Validate in Voyage 2. |
| Paddles can preserve replay-derived truth while using hosted checkpoints and cursors to accelerate resume. | Resume may require tighter invariants or coarser recovery semantics. | Validate in Voyage 3. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| What stream-family naming and versioning policy should we freeze for the first public external Transit contract? | Runtime/integration owner | Open |
| How should consumer projection payload revisions evolve when new transcript/manifold metadata is added? | Runtime/integration owners | Open |
| Which readiness/failure signals belong in Transit projections versus local process health surfaces? | Runtime owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles can run as a long-lived first-party service with hosted Transit as the authoritative persistence layer.
- [ ] Given a Transit endpoint and namespace, a downstream consumer can submit a turn over Transit and observe bootstrap and projection updates over Transit.
- [ ] Paddles restart resumes correctly from hosted cursor/materialization state without losing or duplicating work.
- [ ] Projection state remains replay-derived and reproducible from authoritative Transit history.
- [ ] The production path does not require embedded local Transit storage.
- [ ] Existing HTTP Transit/web surfaces may remain for manual/debug use, but downstream integration does not depend on them.
- [ ] At least one active implementation slice exists with story-level verification paths for authority mode, contract/projection payloads, and hosted resume semantics.
<!-- END SUCCESS_CRITERIA -->
