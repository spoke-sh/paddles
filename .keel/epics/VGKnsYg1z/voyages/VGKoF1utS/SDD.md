# Deliver WebSocket And Transit Transports - Software Design Description

> Add bidirectional WebSocket and Transit-native transport adapters using the same shared lifecycle, visibility, and failure semantics as the HTTP-facing transports.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds the bidirectional native transports after the shared model and HTTP/SSE surfaces exist. WebSocket and Transit should share the same transport lifecycle, auth, and diagnostics semantics while differing in framing and payload representation.

## Context & Boundaries

The work is constrained to bidirectional native transports. WebSocket covers session-oriented duplex communication; Transit covers a structured native transport format on top of the same lifecycle contract. This voyage should not reopen the shared model or earlier HTTP/SSE work except for compatibility seams required to support the new adapters.

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
| Shared transport model | repo contract | Supplies lifecycle/capability/auth/diagnostics semantics | voyage VGKoF0hsS |
| HTTP/SSE transport infrastructure | repo runtime | Provides reference patterns for binding and diagnostics | voyage VGKoF1Stc |
| Runtime tracing/diagnostics | repo contract | Surface bidirectional transport readiness, degradation, and failure | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bidirectional transports stay under one contract | WebSocket and Transit both report through the same lifecycle/diagnostics envelope | Operators should not need separate mental models for readiness and failure. |
| Keep Transit native and explicit | Transit is a distinct adapter, not an incidental payload mode | The user asked for native transport connections, so Transit needs first-class treatment. |
| Verify failure modes directly | Tests/docs must cover negotiation and session setup failure paths | Bidirectional transports are harder to reason about without explicit recovery evidence. |

## Architecture

Bidirectional adapters sit beside HTTP/SSE on the shared transport substrate.

```text
shared transport config + diagnostics
                 |
      +----------+-----------+
      |                      |
      v                      v
 WebSocket adapter      Transit adapter
      |                      |
      +----------+-----------+
                 v
           runtime session layer
```

## Components

- `websocket transport adapter`
  Purpose: Bind and manage duplex sessions with shared readiness, auth, and diagnostics semantics.
- `transit transport adapter`
  Purpose: Exchange structured Transit-native payloads while reusing the same lifecycle and diagnostics model.
- `bidirectional transport verification`
  Purpose: Exercise negotiation, setup, steady-state exchange, and failure reporting for both adapters.

## Interfaces

The voyage should define:

- A WebSocket transport entry point with explicit session lifecycle reporting.
- A Transit-native transport entry point or binding surface with structured payload expectations.
- Shared diagnostics that distinguish disabled, binding, ready, degraded, failed, and negotiation-error states for both adapters.
- Docs that map transport configuration to runtime-visible bind/session behavior.

## Data Flow

At startup, the shared transport model resolves whether WebSocket and Transit are enabled. Each adapter binds through the session layer, negotiates capabilities or structured payload expectations, and reports readiness or failure through the same diagnostics surface. Active session errors should degrade or fail the transport cleanly rather than leaving an ambiguous partially-ready state.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| WebSocket session negotiation succeeds outside the shared lifecycle contract | Tests/diagnostics show ready sessions without shared state updates | Route session establishment through the shared transport model | Re-run bidirectional transport verification |
| Transit payload handling bypasses the native transport contract | Contract tests or docs cannot explain payload shape/lifecycle | Normalize Transit through its dedicated adapter surface | Keep the adapter disabled until the contract is explicit |
| Bidirectional failures leave ambiguous partially-ready state | Diagnostics show stale ready state after session/setup failure | Mark the transport degraded or failed with the shared diagnostics envelope | Retry only through explicit recovery flow |
