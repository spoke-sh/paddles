# Define Shared Native Transport Model - Software Design Description

> Codify one shared transport contract for connection lifecycle, capability negotiation, session identity, diagnostics, and auth so every named transport lands against the same runtime semantics.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage establishes the repo-owned transport substrate that every native connection path will share. The design should separate transport-agnostic lifecycle, capability negotiation, auth/configuration, and diagnostics from any protocol-specific adapter so HTTP, SSE, WebSocket, and Transit can all present the same operator and runtime semantics.

## Context & Boundaries

The work stays inside the paddles runtime and its authored configuration/docs surfaces. This voyage defines the common model and diagnostics boundary only; later voyages add protocol adapters on top of it. Existing runtime transport paths must continue to work unchanged unless a new native transport is explicitly configured.

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
| Existing runtime configuration model | repo contract | Host the authored transport settings and defaults | current repo |
| Current runtime diagnostics / trace surfaces | repo contract | Surface transport availability and failures consistently | current repo |
| Native transport adapter stories | planned follow-on work | Consume the shared transport model defined here | voyage VGKoF1Stc / VGKoF1utS |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Shared model first | Define one transport contract before any adapter lands | This prevents protocol-specific drift and duplicated operator semantics. |
| Diagnostics are part of the contract | Availability/failure state lives in the shared model, not in each adapter | Operators need one mental model across all transports. |
| Additive rollout | New transports remain opt-in until later voyages enable them | This preserves current behavior and local-first safety. |

## Architecture

The shared transport layer should sit between authored configuration/runtime startup and protocol-specific adapters.

```text
authored config / runtime options
              |
              v
     shared transport model
    /    lifecycle state    \
   / capability negotiation  \
  / auth + diagnostics state  \
 v                            v
HTTP/SSE adapters      WS/Transit adapters
              \        /
               runtime observability
```

## Components

- `transport capability vocabulary`
  Purpose: Define named lifecycle stages, capability flags, and session identity semantics shared by every native transport.
- `transport configuration and auth contract`
  Purpose: Represent authored enablement, bind targets, auth material, and transport-specific options behind one common envelope.
- `transport diagnostics model`
  Purpose: Report availability, negotiated mode, connection failures, and recovery state without adapter-specific wording.
- `contract tests and docs`
  Purpose: Lock the transport substrate before downstream adapter implementation stories depend on it.

## Interfaces

The core interface in this voyage is an internal repo contract rather than a wire protocol. It should expose:

- A shared enum or state model for lifecycle phases such as disabled, configured, binding, ready, degraded, and failed.
- A capability surface describing whether a transport supports request/response, server push, full duplex, resumability, or structured Transit payloads.
- A diagnostics envelope that carries transport id, bind target, negotiated capabilities, auth mode, and latest failure summary.
- Configuration inputs that let protocol-specific adapters derive their settings without redefining shared semantics.

## Data Flow

Runtime startup reads authored transport settings, resolves auth/configuration into the shared transport model, and publishes that state into diagnostics before any specific adapter binds. Later voyages will instantiate HTTP/SSE/WebSocket/Transit adapters using this shared state and report readiness/failure back through the same diagnostics channel.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Shared lifecycle semantics drift across adapters | Contract tests or docs fail | Stop adapter rollout until the shared model is corrected | Update the shared transport substrate before changing adapters |
| Auth/config schema is protocol-specific instead of shared | Review/tests expose duplicated fields or semantics | Normalize the fields back into the shared contract | Refactor before downstream adapter stories proceed |
| Diagnostics cannot explain a transport failure consistently | Runtime/contract tests lack shared failure detail | Extend the shared diagnostics envelope | Re-run transport verification against the normalized contract |
