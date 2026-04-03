# Unify Web Conversation Projection And Cross-Surface Live Sync - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver a mission-scoped epic that replaces duplicated web bootstrap and per-panel live refresh logic with one canonical conversation projection contract and shared-session live update path. | board: VFguTx9hQ |
| MG-02 | Rebuild the primary React/TanStack runtime so it owns chat, transit, and manifold routes directly instead of mounting a raw HTML runtime shell or duplicating route/runtime ownership. | board: VFguTx9hQ |
| MG-03 | Add product-route cross-surface verification proving that turns entered through the shared conversation session appear live in the web transcript, transit trace, and manifold without reload, and make that suite part of `just test` and the governor path. | board: VFguTx9hQ |

## Constraints

- Preserve the current look, feel, route semantics, and operator workflow of the web UI while simplifying ownership boundaries underneath.
- The shared conversation session remains the source of truth for interactive turns across TUI and web; the mission should not introduce separate per-surface conversation state.
- Replay-backed transcript, forensic, manifold, and trace projections remain authoritative recovery paths; the browser should not rely on ad hoc panel-local repair heuristics.
- The primary runtime must not depend on iframe proxy layers or raw-HTML runtime bridging once the mission is complete.
- Keep the system local-first and avoid mandatory hosted frontend services or remote state coordinators.
- Verification must run through repo-owned commands so local runs, CI, and the pre-commit governor exercise the same browser/product-route contract.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when epic VFguTx9hQ is verified and only follow-on polish or optional UI experiments remain
- YIELD to human when preserving exact UI parity conflicts with a proposed simplification, or when route/UX changes would alter operator workflow
