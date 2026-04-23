---
# system-managed
id: VHaTauxe4
index: 3
status: accepted
decided_at: 2026-04-22T22:02:42
supersedes: []
superseded-by: null
# authored
title: Hosted Transit As The Authoritative Boundary For Deployed Paddles
context: "runtime"
applies-to: ["transit", "service-mode"]
mission: VHaTatBc4
---

# Hosted Transit As The Authoritative Boundary For Deployed Paddles

## Status

**Proposed** — This ADR records the intended runtime authority boundary for the
hosted first-party deployment path.

## Context

`paddles` currently treats Transit as a native HTTP transport layered onto the
web listener while the default recorder path remains embedded local
`transit-core`. That works for local development, but it is the wrong authority
model for a hosted first-party deployment.

A downstream platform wants to deploy Paddles as a long-lived service and use
hosted Transit as the integration bus between the platform, Paddles, and
consumer surfaces. Transit already exposes hosted append/read primitives,
durable consumer cursors, and hosted materialization checkpoint/resume
semantics.

The current Paddles runtime does not align with that model:

1. The canonical integration seam is still documented as `POST /native-transports/transit` rather than a stable Transit stream contract.
2. Embedded local `transit-core` is still the default recorder authority.
3. Session restore and projection rebuild semantics are oriented around local recorder wake/replay flows rather than hosted cursor/materialization resume.
4. Deployed runtime mode is still biased toward local or interactive surfaces instead of a non-interactive first-party service contract.

## Decision

For deployed and first-party service mode, **hosted Transit becomes the
authoritative persistence, replay, and integration surface for Paddles**.

This decision has six parts:

1. Paddles will add a hosted Transit authority mode backed by `transit-client`. In this mode, hosted Transit is the authoritative event store, replay source, cursor owner, and materialization boundary.
2. Embedded local `transit-core` and in-memory recorders remain available only as explicit local/dev fallback modes. They are not required for production first-party deployment.
3. External integration will be defined as a stable, versioned Transit contract covering bootstrap, turn submission, progress, projection rebuilds, completion/failure, and restore.
4. Replay-derived session and projection views will resume from hosted consumer cursors and hosted materialization checkpoints instead of depending on full replay or local-only recorder state on every restart.
5. Paddles will expose a non-interactive service mode with explicit Transit endpoint, namespace, service identity, readiness, and failure semantics.
6. HTTP UI, debug, and operator surfaces may remain, but they are optional operator tools rather than the canonical first-party integration boundary.

## Constraints

- **MUST:** Treat hosted Transit as the single authority for deployed service mode persistence, replay, and projection derivation.
- **MUST:** Keep projection state replay-derived and reproducible from authoritative Transit history even when resume uses hosted checkpoints and cursors.
- **MUST:** Carry explicit external provenance for account, session, workspace, route, request, and workspace posture through command and projection envelopes.
- **MUST:** Keep external auth ownership outside Paddles. Provenance is for correlation and posture, not for moving auth responsibilities into the Paddles runtime.
- **MUST NOT:** Require embedded local `transit-core` storage for the production first-party deployment path.
- **MUST NOT:** Reopen embedded local Transit storage as a second authority for the same hosted workload.
- **SHOULD:** Retain HTTP UI/debug/operator surfaces as optional manual tools where they remain useful for local development and troubleshooting.
- **SHOULD:** Version the external Transit contract and projection payloads explicitly rather than relying on incidental runtime/event serialization.

## Consequences

### Positive

- Downstream platforms can integrate with Paddles over a durable Transit substrate instead of depending on web-specific control planes.
- Restart behavior can become operationally correct and faster by using hosted cursor and materialization resume primitives.
- Consumer surfaces get a stable, typed projection surface suitable for transcript/detail rendering and deterministic restore.
- The deployed authority boundary becomes explicit: hosted Transit in production, embedded local storage only in local/dev fallback paths.

### Negative

- Recorder, session, projection, and configuration seams need a larger refactor than a simple transport wrapper swap.
- The external Transit contract becomes a compatibility surface that must be versioned deliberately.
- Hosted service mode introduces additional operational failure modes around connectivity, cursor ownership, and checkpoint coordination.

### Neutral

- Local and operator-facing HTTP surfaces can remain where useful, but they no longer define the primary integration contract.
- Replay remains the source of truth; hosted checkpoints and cursors are an acceleration and resumption mechanism, not a semantic change to that model.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Hosted authority mode exists and does not require embedded local storage for deployed service mode | automated | Configuration and runtime tests prove hosted Transit can boot as the authoritative recorder path |
| External Transit envelopes are versioned and cover bootstrap, submission, progress, projection, completion/failure, and restore | automated | Contract tests validate the stream payloads and version markers |
| Restart resume uses hosted consumer cursors and materialization checkpoints without duplicate turn effects | automated | Restart/resume tests verify no-loss/no-duplication behavior |
| Projection payloads remain replay-derived and reproducible from authoritative Transit history | automated | Projection comparison tests validate replay fidelity |
| Docs and config guidance make hosted Transit the first-party deployment path and local recorders an explicit fallback | manual | Review of configuration, architecture, and runtime docs |

## References

- `CONFIGURATION.md`
- `ARCHITECTURE.md`
- `src/infrastructure/adapters/trace_recorders.rs`
- `/home/alex/workspace/spoke-sh/transit/crates/transit-client/README.md`
