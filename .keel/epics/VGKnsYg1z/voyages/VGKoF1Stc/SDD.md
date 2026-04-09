# Deliver HTTP And SSE Transports - Software Design Description

> Add stateless HTTP request/response and SSE streaming transports on top of the shared transport layer so simple and streaming integrations both have first-class native paths.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage implements the first two concrete native transports on top of the shared transport substrate. Stateless HTTP request/response and SSE streaming should share lifecycle, diagnostics, and configuration semantics while remaining distinct in connection behavior and verification.

## Context & Boundaries

The work stays inside the paddles runtime transport layer and its owning docs/tests. HTTP request/response and SSE are in scope because they cover simple and server-push integrations. WebSocket and Transit remain for the next voyage, and unrelated UI or planner transport behavior should not be redesigned here.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Shared transport model | repo contract | Supplies lifecycle, config, auth, and diagnostics semantics | voyage VGKoF0hsS |
| Existing runtime HTTP server surface | repo runtime | Hosts the concrete HTTP and SSE endpoints | current repo |
| Web runtime / diagnostics docs | repo docs | Expose how operators enable and inspect transports | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| One substrate, two adapters | HTTP and SSE both bind through the shared transport model | The semantics should stay aligned even though the protocols differ. |
| Treat SSE as a first-class transport, not a response mode | Separate lifecycle/readiness from plain HTTP | Streaming needs its own diagnostics and verification path. |
| Verify enablement and observability with behavior | Tests cover bind/readiness/failure plus docs | Operators need proof, not just implementation. |

## Architecture

HTTP and SSE adapters should be thin protocol layers over the shared transport core.

```text
shared transport config + diagnostics
                 |
     +-----------+-----------+
     |                       |
     v                       v
HTTP request/response    SSE stream adapter
     |                       |
     +-----------+-----------+
                 v
           runtime server/router
```

## Components

- `http transport adapter`
  Purpose: Handle stateless request/response native transport calls using the shared transport lifecycle contract.
- `sse transport adapter`
  Purpose: Maintain server-sent streaming sessions while reporting readiness/failure through the shared diagnostics surface.
- `transport verification surface`
  Purpose: Prove operator-visible behavior for enablement, bind targets, stream establishment, and failure conditions.

## Interfaces

The voyage should expose:

- A configured HTTP endpoint contract for stateless requests.
- A configured SSE endpoint contract for long-lived server push streams.
- Shared diagnostics describing whether each transport is disabled, binding, ready, degraded, or failed.
- Operator-facing docs explaining how transport configuration maps onto runtime behavior.

## Data Flow

At startup, the shared transport model resolves whether HTTP and/or SSE are enabled. The runtime binds the requested HTTP endpoint(s), reports readiness into diagnostics, and then serves stateless requests or SSE streams through the appropriate adapter. Failures during bind or stream setup flow back into the same shared diagnostics envelope.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| HTTP transport binds but does not reflect shared lifecycle state | Integration tests or diagnostics mismatch | Route all readiness reporting through the shared transport model | Re-run HTTP transport verification |
| SSE stream establishes without transport visibility | Missing diagnostics/tests for stream readiness | Add explicit SSE readiness and failure reporting | Keep SSE disabled until the diagnostics contract is satisfied |
| Operator docs drift from actual endpoints/configuration | Doc tests or behavioral verification fail | Update docs in the same slice as runtime changes | Rebuild and re-run transport verification |
