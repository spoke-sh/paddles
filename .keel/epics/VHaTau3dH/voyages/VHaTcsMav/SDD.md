# Versioned Hosted Transit Contract And Projection Surface - Software Design Description

> Define the stable versioned hosted Transit contract, provenance envelopes, and consumer-facing projection payloads that replace web endpoints as the canonical integration boundary.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage defines the public Transit-facing contract for hosted external
integration. The design introduces versioned command, event, and projection
envelopes with explicit provenance and projection metadata, then routes
external integration through those streams instead of through Paddles web
endpoints.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────┐
│          Hosted External Transit Contract              │
│                                                         │
│  command streams -> paddles runtime -> event streams    │
│                                 -> projection streams   │
└─────────────────────────────────────────────────────────┘
            ↑                               ↑
      External Client               Projection Consumer
```

### In Scope

- versioned contract families and payloads
- provenance envelope shape
- consumer-facing projection payloads

### Out of Scope

- resume storage mechanics
- consumer UI implementation
- downstream deployment/auth wiring

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted Transit | External service | Durable stream authority for command/event/projection exchange | Transit hosted API |
| `transit-client` | Rust crate | Stream append/read/projection operations | Current workspace revision |
| Paddles replay/projection model | Internal architecture | Source for deriving projection payloads from authoritative history | Current runtime |
| Existing consumer projection patterns | External reference | Operational precedent for hosted projection reads and restore metadata | Current downstream ecosystem |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Public contract | Versioned Transit envelopes rather than web endpoint payloads | Makes Transit the canonical external integration boundary |
| Envelope shape | Common provenance envelope plus per-message body | Keeps identity/correlation consistent across command, event, and projection families |
| Projection contract | Consumer-facing typed payload rather than shell/channel transcript fragments | Supports deterministic restore and rendering |

## Architecture

The contract is split into three logical families:

1. `session/command`
   Commands that an external client writes to bootstrap sessions or submit turns.
2. `session/event`
   Runtime lifecycle signals such as acceptance, progress, completion, failure,
   and restore availability.
3. `session/projection`
   Replay-derived materialized views that consumer surfaces can consume for transcript/detail
   rendering and restore.

## Components

- `ContractEnvelope`
  Purpose: wrap every public message in a versioned, provenance-carrying frame.
  Interface: common headers plus message-specific payload body.
  Behavior: remains stable across stream families.

- `TurnLifecycleEvents`
  Purpose: communicate acceptance, progress, completion, failure, and rebuild
  activity back to external consumers and operators.
  Interface: typed event bodies on hosted Transit streams.
  Behavior: reflect runtime lifecycle without depending on HTTP/SSE.

- `ConsumerProjectionPayload`
  Purpose: publish a detail/transcript view suitable for consumer rendering and
  deterministic restore.
  Interface: projection stream/materialization output.
  Behavior: remains replay-derived and carries revision metadata.

## Interfaces

- Common envelope fields
  - `contract_version`
  - `service_identity`
  - `account_id`
  - `session_id`
  - `workspace_id`
  - `route`
  - `request_id`
  - `workspace_posture`
  - message-specific payload

- Contract families
  - bootstrap/restore commands
  - turn submission commands
  - progress/completion/failure events
  - projection rebuild events
  - transcript/detail projections

## Data Flow

1. An external client writes a versioned bootstrap or turn command onto the hosted Transit
   contract stream.
2. Paddles consumes the command, executes the turn, and emits lifecycle events
   back onto Transit.
3. Replay-derived projection reducers publish typed projection payloads carrying
   transcript rows, turn status, revision metadata, and trace/manifold
   availability.
4. A consumer surface reads the projection family to render or restore
   state without consulting Paddles web endpoints.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Contract version unsupported | Envelope validation | Reject command and emit typed failure event | Upgrade/downgrade the caller or add compatibility |
| Required provenance missing | Envelope validation | Reject or quarantine invalid message | Caller resubmits with complete provenance |
| Projection payload diverges from replay truth | Projection contract tests or runtime assertions | Fail verification and block rollout | Fix reducer/contract and replay from authority |
| Optional HTTP surface differs from Transit contract | Cross-surface comparison tests | Treat Transit as canonical and fix mirror surface | Align optional surface to canonical contract |
