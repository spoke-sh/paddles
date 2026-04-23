# Hosted Transit Authority And Service Runtime Mode - Software Design Description

> Make hosted Transit the authoritative runtime path for deployed Paddles while preserving embedded/local recorders only as explicit local fallback or dev mode.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a first-class runtime authority selection layer. In
hosted service mode, Paddles boots against hosted Transit through
`transit-client`, uses hosted Transit for append/read/replay operations, and
surfaces readiness/failure independently of the TUI or web UI. Embedded local
recorders remain available behind explicit local/dev configuration only.

## Context & Boundaries

```
┌───────────────────────────────────────────────────────┐
│              Hosted Service Runtime Mode              │
│                                                       │
│  Config/Lane Parsing  ->  Authority Selector          │
│                              -> Hosted Transit Store  │
│                              -> Service Readiness     │
│                              -> Optional HTTP UI      │
└───────────────────────────────────────────────────────┘
            ↑                          ↑
     Hosted Transit              Operator Surfaces
```

### In Scope

- runtime authority selection and config
- hosted service bootstrap
- recorder/replay binding for hosted mode
- readiness/failure exposure

### Out of Scope

- full contract payload definitions
- consumer projection shaping details
- downstream deployment automation

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `transit-client` | Rust crate | Hosted Transit append/read/cursor/materialization primitives | Current workspace revision |
| Hosted Transit service | External service | Authoritative persistence and replay surface in deployed mode | Transit hosted API |
| Existing recorder/session ports | Internal contracts | Runtime seams to rebind from embedded local storage to hosted authority mode | Current `paddles` runtime |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Recorder authority | Explicit hosted/local authority mode selection | Prevents accidental embedded fallback in deployed service mode |
| Service bootstrap | Non-interactive service runtime with explicit readiness/failure state | Supports long-lived deployment independent of TUI/web |
| Operator surfaces | Keep HTTP/TUI optional and non-authoritative | Preserves debug value without making them the primary contract |

## Architecture

The design introduces three cooperating components:

1. `TransitAuthorityConfig`
   Interprets runtime configuration into one of: hosted service authority,
   embedded local fallback, or in-memory/dev fallback.
2. `HostedTransitTraceStore`
   Owns hosted append/read/replay bindings using `transit-client`.
3. `ServiceRuntimeSupervisor`
   Starts the non-interactive service loop, reports readiness/failure, and
   optionally mounts HTTP operator surfaces without coupling them to authority.

## Components

- `TransitAuthorityConfig`
  Purpose: express the authority choice explicitly.
  Interface: runtime config parsing and validation.
  Behavior: rejects incomplete hosted-service config instead of silently
  degrading to embedded local authority.

- `HostedTransitTraceStore`
  Purpose: provide the hosted-backed implementation of recorder/replay ports.
  Interface: authoritative append, read, and resume operations expected by the
  runtime.
  Behavior: never opens embedded local `transit-core` for hosted workloads.

- `ServiceRuntimeSupervisor`
  Purpose: run Paddles as a long-lived non-interactive process.
  Interface: start, stop, readiness, and failure reporting.
  Behavior: can expose operator HTTP/UI surfaces when configured, but readiness
  does not depend on them.

## Interfaces

- Runtime config contract
  - Transit endpoint
  - Transit namespace
  - service identity
  - authority mode
  - optional operator HTTP enablement

- Hosted recorder/replay interface
  - append authoritative turn/session events
  - read authoritative streams
  - hand off cursor/materialization responsibilities to Voyage 3 seams

## Data Flow

1. Process starts in service mode.
2. Config is parsed into an explicit authority mode.
3. Hosted service mode validates endpoint, namespace, and service identity.
4. `HostedTransitTraceStore` opens hosted clients and binds recorder/replay
   ports.
5. `ServiceRuntimeSupervisor` reports readiness only after hosted authority is
   live.
6. Optional HTTP/UI operator surfaces mount on top of the running service
   without becoming the canonical control plane.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hosted Transit config missing or incomplete | Config validation at startup | Refuse hosted service-mode startup | Operator fixes config and restarts |
| Hosted Transit unavailable | Client/bootstrap failure | Report service not ready and surface failure details | Retry or restart after Transit recovery |
| Embedded recorder selected implicitly during hosted service mode | Authority selection validation | Fail fast instead of silently degrading | Operator chooses an explicit fallback mode |
| Optional HTTP/UI surface fails while hosted authority is healthy | Operator surface bootstrap error | Keep core service running; mark operator surface degraded | Restart operator surface independently where supported |
