# Web Interface And HTTP API For Paddles - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | HTTP server runs alongside paddles with SSE-streamed turn events | board: VFKBCVjpo |
| MG-02 | Web chat interface reflects current conversation state from transit trace streams | board: VFKBDgewu |
| MG-03 | Railroad/turnstep visualization renders trace DAG with hexagonal nodes in real time | board: VFKBFMq8J |
| MG-04 | HTTP API design is captured as a keel research bearing | manual: bearing VFKApee25 assessed and laid |

## Constraints

- HTTP server is an infrastructure adapter, not a new application layer
- Web frontend and CLI are peers consuming the same domain events
- Transit trace streams are the source of truth for conversation history
- Local-first by default: server binds to localhost

## Halting Rules

- HALT when all epics (VFKBCVjpo, VFKBDgewu, VFKBFMq8J) are verified and bearing VFKApee25 is assessed
- YIELD to human if any goal requires product direction input
