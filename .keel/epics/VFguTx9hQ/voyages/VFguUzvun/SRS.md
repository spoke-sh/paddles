# Unified Projection Store And Product-Route Sync - SRS

## Summary

Epic: VFguTx9hQ
Goal: Replace duplicated web bootstrap and multi-endpoint refresh logic with a single shared conversation projection contract, then rebuild the React runtime and product-route E2E around that contract.

## Scope

### In Scope

- [SCOPE-01] A canonical conversation projection snapshot/update contract covering transcript, forensic, manifold, and transit trace state for a shared interactive session
- [SCOPE-02] A unified web bootstrap endpoint and session-scoped live event stream for that projection
- [SCOPE-03] A shared React-side projection store/hook for the primary TanStack runtime
- [SCOPE-04] React/TanStack implementations of the current chat, transit, and manifold routes with visual and behavioral parity
- [SCOPE-05] Product-route browser E2E that verifies live external turn injection, route continuity, and reload recovery
- [SCOPE-06] `just test` and governor verification wiring for the full browser suite
- [SCOPE-07] Documentation updates describing the simplified ownership and data-flow model

### Out of Scope

- [SCOPE-08] Visual redesigns or route taxonomy changes
- [SCOPE-09] Hosted telemetry or remote frontend state services
- [SCOPE-10] TUI redesign work beyond consuming the shared projection correctly
- [SCOPE-11] Optional performance work not required for correctness or verification
- [SCOPE-12] New product features beyond transcript, transit, and manifold parity

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The application boundary exposes a canonical `ConversationProjectionSnapshot` and `ConversationProjectionUpdate` for a shared interactive session, carrying transcript, forensic, manifold, and trace-graph state from the same underlying read models. | SCOPE-01 | FR-01 | test |
| SRS-02 | The web adapter exposes a unified bootstrap endpoint and one session-scoped live event stream for the canonical projection instead of panel-specific bootstrap/event ownership. | SCOPE-02 | FR-02 | test |
| SRS-03 | The primary TanStack runtime uses a shared React-side projection store/hook rather than mounting raw HTML or relying on global imperative bootstrap logic. | SCOPE-03 | FR-03 | test |
| SRS-04 | `/`, `/transit`, and `/manifold` render chat, transit, and manifold surfaces from the shared projection store while preserving current design and route behavior. | SCOPE-04 | FR-04 | manual |
| SRS-05 | Turns entered through another surface attached to the shared conversation session appear live in the open web transcript, transit trace, and manifold without reload. | SCOPE-01, SCOPE-02, SCOPE-04 | FR-05 | test |
| SRS-06 | Product-route browser E2E keeps the page open, injects an external turn, verifies live updates across routes, and proves continuity after reload. | SCOPE-05 | FR-06 | test |
| SRS-07 | `just test` and the governor verification path run the full browser suite needed to protect the cross-surface contract. | SCOPE-06 | FR-07 | test |
| SRS-08 | Foundational and public docs describe the simplified projection architecture, route ownership, and verification model accurately. | SCOPE-07 | FR-08 | review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Replay is sufficient to rebuild transcript, transit, and manifold state after missed live updates without panel-local repair heuristics. | SCOPE-01, SCOPE-02, SCOPE-04 | NFR-01 | test |
| SRS-NFR-02 | The cutover preserves the current operator-facing layout, typography, controls, and route semantics unless a separate human decision changes them. | SCOPE-04 | NFR-02 | manual |
| SRS-NFR-03 | The new architecture reduces duplicated bootstrap, routing, fetch, and event wiring instead of reproducing the same logic in multiple layers. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-03 | review |
| SRS-NFR-04 | Browser verification remains hermetic enough to run consistently through repo-owned commands and the pre-commit governor. | SCOPE-05, SCOPE-06 | NFR-04 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
