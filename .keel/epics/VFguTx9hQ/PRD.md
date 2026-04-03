# Shared Conversation Projection Architecture For Web And TUI - Product Requirements

## Problem Statement

The web runtime currently mounts a monolithic imperative shell inside TanStack, duplicates bootstrap and live projection logic across multiple fetch/SSE paths, and lacks a single product-path cross-surface test proving that turns entered through the shared conversation session appear live in chat, transit, and manifold views.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Replace fragmented web-side ownership with one canonical conversation projection contract that can drive transcript, transit, and manifold from the same shared session state. | Web routes bootstrap from one authoritative snapshot/update model rather than panel-local fetch orchestration | First voyage |
| GOAL-02 | Make the TanStack runtime the true owner of the primary web UI without changing the current operator-facing design or route semantics. | `/`, `/transit`, and `/manifold` render from React-owned state with the current visual behavior preserved | First voyage |
| GOAL-03 | Prove cross-surface live sync, not just web-originated turns. | A browser test can keep the page open, inject a turn from outside the page, and observe chat/transit/manifold update live | First voyage |
| GOAL-04 | Ensure the governor catches these regressions before commit. | `just test` and the pre-commit path both run the product-route browser suite under the same environment contract | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer using paddles through TUI and web during active turns. | Confidence that the web UI reflects the same live conversation state regardless of where the turn was entered. |
| Maintainer | Engineer evolving the runtime or test harness. | A simpler ownership model with one projection abstraction and one reliable end-to-end verification path. |

## Scope

### In Scope

- [SCOPE-01] A canonical conversation projection snapshot/update contract carrying transcript, forensic, manifold, and transit-trace state for a shared interactive session
- [SCOPE-02] A unified web bootstrap endpoint and session-scoped live event stream for that projection
- [SCOPE-03] A TanStack-owned React runtime store and route model that consumes the projection directly
- [SCOPE-04] React implementations of the current chat, transit, and manifold surfaces with visual and behavioral parity
- [SCOPE-05] Product-route browser E2E for live external turn injection, route continuity, and reload recovery
- [SCOPE-06] Verification and governor wiring so full browser E2E runs through `just test`
- [SCOPE-07] Foundational/public docs that describe the simplified ownership and cross-surface projection model honestly

### Out of Scope

- [SCOPE-08] New visual redesigns or route taxonomy changes
- [SCOPE-09] Hosted telemetry systems or remote frontend state services
- [SCOPE-10] TUI UI redesign work unrelated to shared projection correctness
- [SCOPE-11] Optional performance tuning beyond what is needed to keep the new projection architecture correct and verifiable
- [SCOPE-12] Feature expansions beyond transcript/transit/manifold parity

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The application/web boundary must expose one canonical conversation projection snapshot/update contract for shared interactive sessions, covering transcript, forensic, manifold, and transit trace state. | GOAL-01 | must | This removes panel-local reconstruction and creates one source of truth for the web runtime. |
| FR-02 | The web adapter must expose a unified bootstrap endpoint and a session-scoped live event stream for the canonical conversation projection. | GOAL-01, GOAL-03 | must | The browser should hydrate and stay live from one contract rather than many loosely coupled endpoints. |
| FR-03 | The primary React/TanStack runtime must own `/`, `/transit`, and `/manifold` directly, without a raw HTML bridge or iframe/proxy layer. | GOAL-02 | must | The current wrapper architecture is not a durable ownership model. |
| FR-04 | Chat, transit, and manifold routes must render from one shared React-side projection store while preserving the current design and route behavior. | GOAL-01, GOAL-02 | must | The user explicitly wants the same look/feel with a cleaner architecture underneath. |
| FR-05 | Turns entered through any surface attached to the shared conversation session must appear live in the open web UI without reload, including transcript, transit trace, and manifold updates. | GOAL-01, GOAL-03 | must | This is the broken product promise the mission needs to restore. |
| FR-06 | Product-route browser E2E must keep a page open, inject a turn from outside the page, and verify live transcript/transit/manifold updates plus reload continuity. | GOAL-03 | must | This is the missing test that would have caught the current drift. |
| FR-07 | `just test` and the governor verification path must run the full browser E2E suite under the same repo-owned environment contract. | GOAL-04 | must | Verification needs one owner and one execution path. |
| FR-08 | Foundational and public docs must describe the simplified projection architecture, route ownership, and verification model. | GOAL-01, GOAL-04 | should | The docs should match the real system after simplification. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Replay must remain authoritative and sufficient to rebuild the web projection after missed live updates without panel-local repair heuristics. | GOAL-01, GOAL-03 | must | The system needs a robust recovery path when live updates are missed. |
| NFR-02 | The React cutover must preserve current operator-facing layout, typography, controls, and route semantics unless a separate human decision changes them. | GOAL-02 | must | This mission is about ownership simplification, not visual drift. |
| NFR-03 | The simplified architecture must reduce duplicated bootstrap, routing, fetch, and event wiring rather than moving the same duplication into React. | GOAL-01, GOAL-02 | must | DRY simplification is part of the mission, not a side effect. |
| NFR-04 | Browser verification must be hermetic enough to run consistently from repo-owned commands and the pre-commit governor. | GOAL-04 | must | If the test path is not stable, the governor cannot protect the contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Canonical projection contract | Unit/integration tests around snapshot/update payloads and replay rebuild behavior | Story-level test evidence |
| React route ownership and parity | Product-route browser E2E plus targeted UI tests | Story-level browser/test proof |
| Cross-surface live sync | External turn-injection browser tests plus adapter/service integration tests | Story-level proof |
| Verification/governor path | Workflow contract tests and repo command verification | Story-level proof |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The current runtime shell behavior can be preserved while porting ownership to a React/TanStack app. | The cutover may require additional parity slices or human decisions on acceptable drift. | Validate during the parity port story. |
| Existing transcript/forensic/manifold/trace read models are rich enough to support a single projection snapshot/update contract. | The mission may need a deeper application-layer projection extension first. | Validate in the first projection story. |
| A single browser suite can reliably inject external turns against the shared conversation session in the current local-first harness. | The E2E contract may need harness work before it can guard regressions. | Validate in the cross-surface E2E story. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How should the browser identify or attach to the shared interactive session for external-turn E2E without introducing misleading session semantics? | Architecture / web | Open |
| Can the current giant imperative runtime shell be ported incrementally without leaving a long-lived mixed-ownership state? | Web / architecture | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The primary web runtime boots from one canonical conversation projection snapshot/update contract instead of duplicated panel-local fetch/SSE orchestration
- [ ] `/`, `/transit`, and `/manifold` are true TanStack/React routes with the current design and behavior preserved
- [ ] An open browser page updates live when a turn is entered through another surface attached to the shared conversation session
- [ ] `just test` and the governor both run the product-route browser suite that proves the cross-surface contract
<!-- END SUCCESS_CRITERIA -->
