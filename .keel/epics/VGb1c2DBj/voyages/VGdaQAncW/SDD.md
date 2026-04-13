# Establish A Replayable Multi-Agent Delegation Substrate - Software Design Description

> Establish one replayable multi-agent delegation substrate with explicit lifecycle operations, ownership boundaries, and parent-visible worker artifacts across shared surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the existing subagent and thread-lineage foundations into a
replayable delegation substrate instead of a set of prompt conventions. The
design introduces typed worker lifecycle operations, role and ownership
contracts, lineage-aware coordination, and parent-visible worker artifacts that
all operator surfaces can inspect.

The slice stays intentionally local-first and recursive. It builds on the
current harness identity, specialist-brain substrate, recorder, and projection
pipelines rather than introducing an out-of-band orchestrator. The outcome is a
bounded multi-agent model where the parent remains responsible for integration
while workers operate through explicit contracts.

## Context & Boundaries

In scope are:
- typed lifecycle operations for spawn, follow-up input, wait, resume, and
  close
- explicit role metadata and ownership guidance for delegated work
- lineage-aware parent and worker coordination inside the recursive harness
- parent-visible worker artifacts, tool calls, and completion summaries
- projection hooks and docs that keep multi-agent state legible

Out of scope are:
- autonomous swarms or self-replicating delegation
- hidden parallel writes without ownership and integration rules
- hosted orchestration or remote cluster management
- replacing the recursive harness with a second orchestration product

```
┌─────────────────────────────────────────────────────────────┐
│       This Voyage: Replayable Delegation Substrate         │
│                                                             │
│ Parent Turn -> Delegation Contracts -> Worker Coordinator   │
│                    ↓                    ↓                   │
│         Role / Ownership Records   Thread Lineage           │
│                    ↓                    ↓                   │
│       Worker Artifacts / Summaries / Explicit Status        │
│                    ↓                                        │
│        Transcript / TUI / Web / API Projections             │
└─────────────────────────────────────────────────────────────┘
        ↑                                          ↑
   Existing recursive                         Existing surface
   runtime and recorder                       and projection layers
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Thread-lineage model | Internal runtime | Preserve durable parent/worker relationships and lifecycle replayability | current repo |
| Specialist-brain substrate | Internal runtime | Provide the existing bounded role-oriented delegation foundation this voyage generalizes | current repo |
| Recorder and replay model | Internal runtime | Persist lifecycle operations, worker artifacts, and integration summaries without a parallel state store | current repo |
| Transcript and projection pipelines | Internal runtime | Render shared delegation vocabulary across transcript, TUI, web, and API surfaces | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delegation placement | Extend the current recursive harness with typed delegation contracts | Preserves one runtime identity and avoids a hidden orchestration sidecar |
| Worker lifecycle | Treat spawn, follow-up input, wait, resume, and close as explicit operations | Operators need durable, auditable lifecycle control |
| Ownership model | Require role metadata, write ownership, and parent integration responsibility | Safe parallel work depends on visible boundaries |
| Worker evidence | Record worker outputs, tool calls, and final summaries as parent-visible artifacts | Delegation only helps if the parent can inspect and integrate results |
| Surface vocabulary | Share one delegation vocabulary across transcript and UI surfaces | Multi-agent state must stay coherent across every operator view |
| Failure posture | Return explicit rejected, stale, conflicting, or unavailable states | Honest degradation is required for trust in parallel work |

## Architecture

The voyage adds a thin delegation layer around the existing recursive control
loop and thread-lineage substrate.

1. Parent-originated delegation requests are normalized into typed worker
   lifecycle operations plus role and ownership metadata.
2. A worker coordinator validates whether the requested role, ownership, and
   lifecycle transition can apply safely to the current runtime state.
3. Accepted operations create or update lineage-aware worker records and attach
   delegated work to the same recursive harness.
4. Worker execution emits parent-visible artifacts for tool calls, summaries,
   and integration state.
5. Replay and projection layers consume the same records to reconstruct live or
   historical multi-agent state across transcript and operator surfaces.

## Components

- `WorkerLifecycleOperation`
  Purpose: represent spawn, follow-up input, wait, resume, and close as typed
  delegation operations.
  Interface: parent-to-runtime delegation request contract.
  Behavior: captures the requested lifecycle action plus the minimum metadata
  needed to accept, reject, or defer it honestly.

- `DelegationRoleContract`
  Purpose: define worker role, ownership boundaries, and parent integration
  responsibility.
  Interface: shared runtime contract attached to delegated work.
  Behavior: makes authority and write boundaries explicit before workers begin.

- `WorkerCoordinationController`
  Purpose: evaluate lifecycle transitions and route delegated work through the
  existing recursive harness.
  Interface: runtime entry point for worker lifecycle operations.
  Behavior: preserves lineage, enforces ownership rules, and emits explicit
  status when a request cannot apply safely.

- `WorkerArtifactRecord`
  Purpose: capture worker outputs, tool calls, summaries, and integration
  results as parent-inspectable records.
  Interface: recorder, replay, and projection sinks.
  Behavior: keeps delegated work auditable without inspecting hidden worker
  state.

- `DelegationProjectionAdapters`
  Purpose: render delegation state across transcript, TUI, web, and API
  surfaces.
  Interface: existing projection and transport layers.
  Behavior: exposes active workers, roles, ownership, progress, and completion
  or integration state using one shared vocabulary.

## Interfaces

- `submit_worker_lifecycle(operation) -> DelegationResult`
- `record_worker_artifact(worker_id, artifact) -> TraceRecord`
- `list_active_workers(parent_thread) -> DelegationSnapshot`
- `integrate_worker_result(parent_thread, worker_id) -> IntegrationResult`
- `replay_delegation_state(task_or_thread) -> DelegationSnapshot`

## Data Flow

1. A parent turn issues a typed worker lifecycle operation with role and
   ownership guidance.
2. The worker coordinator resolves the current parent thread, active workers,
   and ownership envelope from the recursive runtime state.
3. The request is validated against lineage, governance, and ownership rules.
4. If accepted, the runtime records the lifecycle event and creates or updates
   the worker inside the same recursive harness.
5. Worker execution emits tool-call summaries, artifacts, progress, and final
   summaries that remain visible to the parent.
6. Parent turns continue local non-overlapping work, wait for workers when
   needed, and integrate or refine returned results through explicit lineage.
7. Transcript and projection layers consume the same records to render live and
   replayable delegation state.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Worker lifecycle request targets a stale or missing parent or worker | Coordinator cannot resolve a valid lineage target | Return explicit stale or unavailable status and record the rejected transition | Refresh runtime state and retry against the current lineage |
| Requested ownership overlaps an existing active owner | Ownership validation detects conflicting write scope or integration authority | Reject or defer the request with a conflicting-ownership status | Narrow the scope, reassign ownership, or wait for the active owner to finish |
| Worker artifacts fail to project on one surface | Projection layer receives an unhandled delegation item | Preserve the record in trace and emit a degraded generic summary | Extend the owning surface vocabulary without changing the delegation contract |
| Delegated work attempts to bypass parent governance or evidence policy | Runtime policy checks detect a mismatch between parent and worker constraints | Refuse the operation and surface an explicit policy failure | Keep delegation inside the parent policy envelope before retrying |
