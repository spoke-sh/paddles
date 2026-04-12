# Establish A Replayable Turn And Thread Control Substrate - Software Design Description

> Establish one replayable turn/thread control substrate with same-turn steering, durable lifecycle transitions, and shared live runtime items for all surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the current recursive harness into a steerable runtime rather
than a prompt loop with a few side channels. The design introduces typed
turn-control and thread-control operations, routes same-turn steering through
replayable control records, and promotes live plan and diff state into shared
runtime items that every surface can observe.

The slice is intentionally local-first. It builds on the existing recorder,
replay, and thread-lineage primitives instead of introducing a second session
state model. The outcome should be one control vocabulary that TUI, web, and
HTTP or API projections can all consume without inventing divergent semantics.

## Context & Boundaries

In scope are:
- typed turn and thread control contracts
- replayable same-turn steering and interruption
- lineage-aware fork, resume, and rollback or archive transitions
- shared runtime items for plan, diff, command, file, and control state
- projection hooks and docs that keep the control plane legible

Out of scope are:
- hosted orchestration or remote control services
- cross-machine collaboration semantics
- full multi-agent delegation
- replacing the existing recursive core loop with a different execution model

```
┌────────────────────────────────────────────────────────────┐
│          This Voyage: Turn / Thread Control Plane         │
│                                                            │
│ Surface Control Input -> Turn Control Coordinator          │
│                               ↓                            │
│                  Recursive Planner / Runtime Loop          │
│                   ↙           ↓           ↘                │
│          Control Records   Runtime Items   Thread Lineage │
│                   ↘           ↓           ↙                │
│              Replay / Transcript / TUI-Web-API            │
└────────────────────────────────────────────────────────────┘
        ↑                                         ↑
   Existing recorder                         Existing surfaces
   and replay model                          and transport layers
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing recorder and replay model | Internal runtime | Persist control records and runtime items without a parallel state store | current repo |
| Current thread-lineage model | Internal runtime | Preserve fork, resume, and rollback lineage semantics | current repo |
| Projection and event pipelines | Internal runtime | Render shared control/runtime items across TUI, web, and API surfaces | current repo |
| Recursive planner and controller loop | Internal runtime | Remain the single execution loop that control operations steer | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Control-plane placement | Add typed control contracts around the current recursive loop | Keeps one runtime model and avoids a second orchestration substrate |
| Steering semantics | Model steer and interrupt as replayable control records, not queued prompt conventions | Same-turn intervention must be inspectable and durable |
| Thread lifecycle | Preserve fork, resume, and rollback as lineage-aware transitions | Operators need trustworthy replay for thread control |
| Runtime item vocabulary | Emit shared plan, diff, command, file, and control items | Surfaces should render the same semantics instead of inventing their own |
| Failure behavior | Return explicit stale, rejected, or unavailable control status | Honest degradation is necessary for trust in interruptibility |

## Architecture

The voyage introduces a thin control layer around the existing controller and
threading substrate.

1. Surface-originated control requests are normalized into typed turn or thread
   operations.
2. A control coordinator validates whether the request can apply to the active
   runtime state.
3. Accepted operations emit replayable control records and steer the existing
   recursive loop or thread lineage model.
4. The runtime emits shared runtime items for plan updates, diff changes,
   command summaries, file changes, and control status.
5. Projection layers render those items consistently across TUI, web, and API
   surfaces.

## Components

- `TurnControlOperation`
  Purpose: represent typed same-turn operations such as steer, interrupt,
  continue, or cancel.
  Interface: surface-to-runtime request contract.
  Behavior: captures the desired control action plus bounded metadata required
  to apply or reject it honestly.

- `ThreadControlOperation`
  Purpose: represent lifecycle operations such as fork, resume, archive, or
  rollback-style transitions.
  Interface: lineage-aware thread control contract.
  Behavior: preserves explicit thread identity and parent-child relationships.

- `TurnControlCoordinator`
  Purpose: evaluate and apply turn and thread control operations.
  Interface: runtime entry point for control requests.
  Behavior: maps accepted requests onto the current recursive loop and emits
  structured rejected or stale status when requests cannot apply.

- `RuntimeItemEmitter`
  Purpose: project plan, diff, command, file-change, and control-state updates.
  Interface: existing event and trace sinks.
  Behavior: keeps work-in-progress state visible during active turns.

- `ControlProjectionAdapters`
  Purpose: render the shared control vocabulary across TUI, web, and API
  surfaces.
  Interface: existing projection and transport layers.
  Behavior: consumes one item vocabulary and degrades unknown states honestly.

## Interfaces

- `submit_turn_control(operation) -> ControlResult`
- `submit_thread_control(operation) -> ControlResult`
- `record_control_event(operation, result) -> TraceRecord`
- `emit_runtime_item(item) -> RuntimeEvent/ProjectionUpdate`
- `replay_control_state(task_or_thread) -> ControlSnapshot`

## Data Flow

1. A surface issues a typed turn or thread control request.
2. The control coordinator resolves the active turn or thread state from the
   current recorder and runtime context.
3. The request is validated against the active execution window.
4. If accepted, the coordinator records a replayable control event and steers
   the active recursive loop or thread lifecycle.
5. The runtime emits shared plan, diff, command, file, and control items while
   work continues.
6. Replay and projection layers consume the same records and runtime items to
   reconstruct the live or historical control state.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Control request targets a stale or missing turn/thread | Coordinator cannot resolve an applicable active target | Return explicit stale or unavailable control status and record the outcome | Refresh the surface view and retry against the current state |
| Requested control action is invalid in the current execution window | Runtime detects an unsafe or unsupported transition | Reject the operation with a typed explanation instead of mutating hidden state | Wait for a safe window or choose a supported action |
| Runtime item projection lacks a surface renderer | Projection layer receives an unhandled item kind | Preserve the item in trace and render a degraded generic summary | Extend the owning surface vocabulary without changing the control contract |
| Replay cannot reconstruct a control transition due to missing lineage | Replay validation detects incomplete control records | Fail loudly in tests and surface replay diagnostics | Repair the recording path before expanding surface use |
