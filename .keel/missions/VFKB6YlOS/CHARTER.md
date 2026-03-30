# Web Interface And HTTP API For Paddles - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | HTTP server runs alongside paddles with SSE-streamed turn events | board: epic delivering axum server with session and turn endpoints |
| MG-02 | Web chat interface reflects current conversation state from transit trace streams | board: epic delivering browser-based chat UI consuming SSE events |
| MG-03 | Railroad/turnstep visualization renders trace DAG with hexagonal nodes in real time | board: epic delivering trace graph visualization with branch/merge rendering |
| MG-04 | HTTP API design is captured as a keel research bearing | board: bearing VFKApee25 assessed |

## Constraints

- HTTP server is an infrastructure adapter, not a new application layer
- Web frontend and CLI are peers consuming the same domain events
- Transit trace streams are the source of truth for conversation history
- Local-first by default: server binds to localhost

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human when only `metric:` or `manual:` goals remain
