# Modularize React Runtime Application - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that decomposes the React runtime web UI into coherent app, chat, store, route, style, and contract modules so maintainers can evolve the web surface without navigating monolithic files. | board: VGEVm5Ibi |

## Constraints

- Preserve current runtime behavior across chat, inspector, manifold, and transit while modularizing the React application.
- Decompose by stateful domain boundary rather than producing a large number of shallow wrapper components.
- Keep shared state seams explicit, especially transcript streaming, composer behavior, prompt history, manifold turn selection, and runtime projection transport.
- Treat the embedded fallback shell as a contract boundary that must be documented and tested; do not assume it can be silently ignored or removed.
- Keep the work local-first and repo-owned. Do not introduce IDE-fed context, remote composition services, or network dependencies to support the decomposition.

## Halting Rules

- DO NOT halt while epic `VGEVm5Ibi` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VGEVm5Ibi` is complete and the modular runtime architecture is backed by board-linked implementation evidence.
- YIELD to the human if the remaining decision requires product direction on embedded-shell parity expectations or whether the fallback shell should be retired instead of maintained.
