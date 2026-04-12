# External Capability Fabric For Web, MCP, And Connectors - Product Requirements

## Problem Statement

Paddles remains mostly a local workspace-and-shell harness, so it cannot reach Codex-class capability breadth across web search, MCP servers, and connector-backed apps through one typed recursive tool fabric.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define a typed external capability fabric that can negotiate web, MCP, and connector-backed tools. | The planner/runtime can discover and reason about external tool capability classes through one contract. | New capability fabrics fit the recursive harness without bespoke controller branches. |
| GOAL-02 | Integrate external tool calls into the evidence-first recursive loop. | External calls produce evidence, diagnostics, and trace artifacts instead of raw unstructured output. | Operators can trust and cite external results the same way they trust local evidence. |
| GOAL-03 | Keep external capability use governed, optional, and honestly degraded. | Missing auth, unavailable tools, or denied permissions surface explicitly without breaking local-first operation. | Paddles remains useful even when external fabrics are absent. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive Operator | A user asking Paddles for current, external, or account-backed information. | A way to reach beyond the local workspace without leaving the recursive harness. |
| Capability Integrator | A maintainer adding new tool fabrics or connectors. | One typed surface for external tools instead of bespoke adapters per feature. |
| Runtime Maintainer | A contributor protecting local-first behavior while expanding tool breadth. | Explicit degradation and policy boundaries for external capabilities. |

## Scope

### In Scope

- [SCOPE-01] Define a capability contract for web search, MCP-backed tools, and connector-backed app actions.
- [SCOPE-02] Add planner/runtime discovery and invocation semantics for those external capability classes.
- [SCOPE-03] Convert external results into evidence items, source records, and trace/runtime artifacts.
- [SCOPE-04] Integrate auth, approval, and unavailability handling with the broader execution-governance model.
- [SCOPE-05] Update docs and operator-visible diagnostics around external tool use.

### Out of Scope

- [SCOPE-06] A generic plugin marketplace or arbitrary third-party extension ecosystem.
- [SCOPE-07] Hosted connector backplanes or cloud-only auth infrastructure that would break local-first operation.
- [SCOPE-08] Review-mode or multi-agent semantics except where needed to consume the external capability fabric.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must negotiate a typed capability descriptor for web search, MCP tools, and connector-backed app actions. | GOAL-01 | must | The harness needs a durable abstraction for external capability breadth. |
| FR-02 | The planner must be able to discover and invoke external capabilities through the same recursive action loop used for local work. | GOAL-01, GOAL-02 | must | External tool use should feel native to the harness rather than bolted on. |
| FR-03 | External results must be normalized into evidence/source artifacts with lineage, availability state, and operator-visible summaries. | GOAL-02, GOAL-03 | must | Evidence-first behavior depends on structured results and provenance. |
| FR-04 | External capability calls must compose with auth, approval, and sandbox governance rather than bypassing the local execution policy model. | GOAL-02, GOAL-03 | must | Tool breadth without governance would widen the current safety gap. |
| FR-05 | Tool absence, auth failure, or stale capability metadata must degrade honestly with explicit runtime state and no false success. | GOAL-03 | must | Trust depends on honest degradation. |
| FR-06 | Docs and runtime projections must explain which external fabrics are active and how to reason about their outputs. | GOAL-01, GOAL-03 | should | Operators need to understand which world the harness can currently reach. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve useful local-first operation when all external capability fabrics are absent or disabled. | GOAL-03 | must | External reach should be additive, not foundational. |
| NFR-02 | External capability metadata and results must be observable through trace, transcript, and API surfaces. | GOAL-02, GOAL-03 | must | Operators need uniform visibility across local and external work. |
| NFR-03 | External calls must respect the same policy and credential-boundary constraints as local execution hands. | GOAL-03 | must | External capability breadth is also a security boundary. |
| NFR-04 | Capability descriptors should remain generic enough to absorb new fabrics without reworking the recursive planner contract. | GOAL-01 | should | The abstraction should outlive the first three fabrics. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Capability negotiation | Contract tests and adapter-level validation across web, MCP, and connector fabrics | Story-level verification artifacts and command logs |
| Evidence normalization | Integration tests and transcript proofs over external result ingestion | Story-level evidence captures and UI proofs |
| Governance and degradation | Runtime tests for auth failure, disabled capability, and denied approval paths | Story-level verification artifacts and updated docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Web, MCP, and connector-backed actions can share enough structure to fit one capability fabric. | The harness may need multiple capability families with translation layers. | Validate the descriptor design against the first three fabrics during decomposition. |
| Operators benefit more from explicit external-tool provenance than from hiding tool boundaries behind a chat abstraction. | UX may need simplification if provenance feels noisy. | Exercise transcript and web projections early in voyage planning. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which web-search and MCP capability classes belong in the first slice versus later expansion? | Epic owner | Open |
| How should connector auth bootstrap locally without introducing a brittle cloud-only dependency? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles can negotiate and invoke web, MCP, and connector-backed capabilities through one typed recursive tool fabric.
- [ ] External results become evidence with provenance, not opaque side-channel output.
- [ ] Disabled, unauthenticated, or denied external capabilities degrade honestly without breaking the local harness.
- [ ] Docs and traces make active external reach and resulting evidence legible to operators.
<!-- END SUCCESS_CRITERIA -->
