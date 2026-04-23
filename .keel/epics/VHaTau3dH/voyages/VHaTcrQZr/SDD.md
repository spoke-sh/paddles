# Hosted Cursor Resume And Projection Rebuild Semantics - Software Design Description

> Resume replay-derived session views and projections from hosted cursors and materialization checkpoints without depending on local recorder state or full replay on every restart.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds hosted resume mechanics to the session and projection sides of
the runtime. Consumers advance durable hosted cursors, projection rebuilders
advance hosted materialization checkpoints, and restart bootstraps from those
positions while preserving authoritative replay semantics and a full-replay
correctness path.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────┐
│         Hosted Resume And Projection Rebuild           │
│                                                        │
│  command/event consumers -> hosted cursors             │
│  projection reducers      -> hosted checkpoints        │
│  restart bootstrap        -> resume coordinator        │
└────────────────────────────────────────────────────────┘
            ↑                               ↑
      Hosted Transit                  Full Replay Baseline
```

### In Scope

- hosted cursor ownership for service consumers
- hosted checkpoint ownership for projection reducers
- restart bootstrap and resume validation

### Out of Scope

- public contract versioning
- local-only recovery modes as primary deployment semantics
- consumer UI details

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Hosted Transit cursors | External primitive | Durable consumer positions for command/event streams | Transit hosted API |
| Hosted materialization checkpoints | External primitive | Durable projection resume positions | Transit hosted API |
| `transit-client` | Rust crate | Access to cursor and materialization APIs | Current workspace revision |
| Paddles projection reducers | Internal components | Replay-derived session and transcript/detail views | Current runtime |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Session resume primitive | Hosted consumer cursors | Matches hosted authority and avoids local-only replay checkpoints |
| Projection resume primitive | Hosted materialization checkpoint/resume | Aligns projection rebuilds with hosted Transit materialization model |
| Correctness baseline | Full replay remains available and testable | Resume optimization must not replace replay truth |

## Architecture

The design introduces a `ResumeCoordinator` that mediates between runtime
consumers and hosted resume primitives:

1. `SessionCursorStore`
   Advances and resumes hosted consumer cursors for command and lifecycle
   consumers.
2. `ProjectionCheckpointStore`
   Advances and resumes hosted materialization checkpoints for projection
   reducers.
3. `ResumeCoordinator`
   Boots the service from hosted cursor/checkpoint state, validates revision
   continuity, and falls back to authoritative replay when necessary.

## Components

- `SessionCursorStore`
  Purpose: own the hosted cursor position for session and lifecycle consumers.
  Interface: load cursor, commit cursor, expose current cursor metadata.
  Behavior: advances only after corresponding effects are durably published.

- `ProjectionCheckpointStore`
  Purpose: own hosted materialization checkpoint state for replay-derived
  projections.
  Interface: load checkpoint, commit checkpoint, report checkpoint metadata.
  Behavior: resumes materializations without inventing non-replay state.

- `ResumeCoordinator`
  Purpose: orchestrate startup and determine whether to resume from hosted state
  or re-run from authoritative history.
  Interface: startup bootstrap and diagnostic reporting.
  Behavior: detects missing or incompatible hosted state and chooses safe
  recovery.

## Interfaces

- Cursor interface
  - identify hosted consumer
  - load current cursor position
  - commit cursor after durable side effects

- Checkpoint interface
  - identify hosted materialization
  - load current checkpoint/resume token
  - commit checkpoint after projection advancement

- Diagnostic metadata
  - current replay revision
  - resumed cursor ids/positions
  - resumed checkpoint ids/positions

## Data Flow

1. Service starts in hosted authority mode.
2. `ResumeCoordinator` loads hosted consumer cursors and materialization
   checkpoints.
3. Session/lifecycle consumers resume from their hosted cursor positions.
4. Projection reducers resume from hosted checkpoints and continue publishing
   replay-derived views.
5. If hosted resume state is missing or incompatible, the coordinator falls
   back to authoritative replay and re-establishes hosted state from there.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cursor missing for existing hosted workload | Resume bootstrap | Fall back to authoritative replay from safe baseline | Recreate hosted cursor state after replay |
| Checkpoint missing or incompatible | Materialization resume startup | Rebuild projection from authoritative history | Publish fresh checkpoint after rebuild |
| Cursor advances before durable side effects | Restart/replay verification failure | Treat as correctness bug and fail verification | Fix commit ordering and re-run replay |
| Projection checkpoint diverges from replay revision metadata | Resume validation | Reject checkpoint and rebuild from authoritative history | Recompute and republish checkpoint |
