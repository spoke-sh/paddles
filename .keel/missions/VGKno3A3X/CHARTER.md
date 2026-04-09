# Expand Native Transport Connections - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that expands Paddles communication with first-class native transport connections for Transit, stateless HTTP request/response, server-sent events, and WebSockets, so transport behavior becomes a repo-owned capability instead of an ad hoc adapter edge. | board: VGKnsYg1z |

## Constraints

- Preserve the local-first runtime contract. New transport additions must remain repo-owned and must not depend on IDE-fed context, hosted brokers, or external transport orchestration services.
- Treat the transport layer as one shared capability surface: connection lifecycle, authentication, framing, session identity, and observability should converge on common contracts rather than four unrelated adapters.
- Keep existing CLI, TUI, web, and recorder boundaries debuggable. Each transport must surface enough diagnostics to make connection state, failures, and replay evidence visible in the existing runtime traces.
- Favor additive transport integration over protocol sprawl. The first slice should cover the four named transports without expanding into unrelated protocol families or generic plugin marketplaces.
- Update the owning docs and contracts when behavior changes. Transport semantics, configuration, and verification expectations must live in the canonical repo documents and board artifacts.

## Halting Rules

- DO NOT halt while epic `VGKnsYg1z` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VGKnsYg1z` is complete and the native transport stack is backed by board-linked implementation evidence for Transit, HTTP request/response, SSE, and WebSocket paths.
- YIELD to the human if the remaining decision requires product direction on transport exposure, such as which transports should be enabled by default or how broadly remote connection modes should be surfaced in the operator UI.
