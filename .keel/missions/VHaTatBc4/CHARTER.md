# Support Hosted Transit-Backed First-Party Paddles Service Mode - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that makes hosted Transit the authoritative persistence, replay, and integration surface for deployed Paddles by adding a service-oriented runtime mode, a versioned external Transit contract, hosted cursor/materialization resume semantics, and consumer-facing replay-derived projections. | board: VHaTau3dH |

## Constraints

- Hosted Transit must become the authoritative persistence and replay boundary for deployed or first-party service mode. Embedded `transit-core` and in-memory recorders may remain only as explicit local/dev fallbacks.
- External integration with deployed Paddles must be grounded on Transit streams and hosted materializations rather than Paddles HTTP endpoints. HTTP UI, debug, and operator surfaces may remain, but they are not the canonical integration contract.
- Projection state must remain replay-derived and reproducible from authoritative Transit history. Resume optimizations may use hosted cursors and materialization checkpoints, but they must not invent alternate truth.
- The contract must carry explicit external provenance for account, session, workspace, route, and request identity without shifting auth ownership into Paddles.
- Deployed service mode must be long-lived and non-interactive, with explicit hosted Transit endpoint, namespace, service identity, readiness, and failure reporting.
- Transit hosted primitives must be used consistently. Paddles must not reopen embedded local Transit storage as a second authority for the same hosted workload.
- Update the owning docs, ADR, and planning artifacts in the same slices that change hosted Transit authority, contract, or runtime mode behavior.

## Halting Rules

- DO NOT halt while epic `VHaTau3dH` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VHaTau3dH` is complete and board-linked evidence shows that Paddles can run in a hosted Transit authority mode, external consumers can submit and observe turns over Transit, restart resume uses hosted cursor/materialization state, and the production path no longer requires embedded local Transit storage.
- YIELD to the human if the remaining decision requires product direction on the exact consumer projection payload semantics, the stable public versioning policy for the external Transit contract, or the deployment identity/auth boundary between Paddles and downstream consumers.
