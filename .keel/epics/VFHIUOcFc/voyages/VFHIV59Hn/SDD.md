# Model-Driven Thread Split And Merge UX - Software Design Description

> Route steering prompts during active turns through a model-driven thread decision loop backed by transit lineage, explicit thread/merge artifacts, and a thread-aware operator experience.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the recursive harness with explicit thread structure instead
of treating steering prompts as anonymous queued input. A steering prompt that
arrives while a turn is running becomes a structured thread candidate. At the
next safe checkpoint, the planner model decides whether that candidate belongs
to the current thread, should open a child branch, or should be merged or
reconciled back into the mainline through an explicit summary or merge record.

The controller continues to own bounded execution, recorder safety, and replay
plumbing. The model owns the classification decision through a constrained
contract. Embedded `transit-core` remains the first durable backend, but the
conversation-specific thread API lives in a paddles-owned layer or crate so it
can be extracted later rather than pushed into `transit-core`.

## Context & Boundaries

This voyage covers:
- turning steering prompts into structured thread candidates
- model-driven thread decision selection
- a paddles-owned conversation/thread layer above the recorder boundary
- durable recorder projection into embedded `transit-core`
- thread replay and merge-back transcript rendering

This voyage does not cover:
- arbitrary parallel local generation against the same model session
- server-only trace infrastructure
- product-specific thread heuristics for Keel or other one workspace domain

```text
┌─────────────────────────────────────────────────────────────────────┐
│                         This Voyage                                │
│                                                                     │
│  Steering prompt                                                    │
│        │                                                            │
│        v                                                            │
│  Thread candidate queue ──> model thread decision ──> controller    │
│        │                                   │              │          │
│        │                                   │              ├─ continue mainline
│        │                                   │              ├─ open child thread
│        │                                   │              └─ merge/reconcile
│        v                                   v                         │
│  trace records / artifact envelopes ──> embedded transit-core ──────┘
│        │
│        v
│  thread-aware transcript + replay
└─────────────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `transit-core` | local library | Durable embedded append/branch/replay/checkpoint backend for thread lineage, now with stronger branch metadata helpers, branch replay views, and artifact descriptor helpers | existing workspace dependency |
| paddles conversation/thread layer | paddles-owned crate or module | Extractable conversation API over recorder and transcript semantics | introduced by this voyage |
| `TraceRecorder` boundary | paddles port | Keeps recorder mapping storage-neutral inside the runtime | existing paddles port |
| Planner lane | paddles runtime | Selects bounded thread decisions from interpretation context | existing planner contract, extended |
| Interactive TUI | paddles CLI | Renders thread split, active-thread, and merge-back state | existing transcript frontend, extended |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Thread classifier | Model-driven through a constrained contract | Matches the recursive harness direction and avoids hardcoded routing heuristics |
| Durability | Use a paddles-owned conversation layer over embedded `transit-core` | Keeps transit primitive while letting paddles define the higher-level conversation/thread contract |
| Merge semantics | Explicit merge, backlink, or summary records instead of history rewrite | Keeps thread structure replayable and inspectable |
| Active-turn handling | Capture immediately, decide at safe checkpoints | Avoids pretending one local session can handle unbounded true concurrency |
| Upstream helper reuse | Consume `transit-core` branch metadata helpers, branch replay views, and artifact descriptor helpers inside the paddles-owned conversation layer | Reduces low-level glue while preserving the architecture boundary that keeps conversation APIs out of transit |

## Architecture

The voyage touches five cooperating layers:

1. Input capture
   The TUI keeps accepting prompts while a turn is active and projects them into
   structured thread candidates rather than plain strings.

2. Thread decision planner
   A model prompt sees operator memory, recent turn lineage, current thread
   state, and the steering candidate, then emits a bounded decision.

3. Controller and runtime state
   The controller validates the decision, opens/continues/merges thread state,
   and keeps execution budgets bounded.

4. Conversation layer and recorder projection
   Runtime transitions are first mapped into a paddles-owned conversation
   contract, then projected into paddles-owned trace records and artifact
   envelopes, and finally persisted through the existing recorder boundary into
   embedded `transit-core`.

5. Thread-aware transcript and replay
   The TUI renders the visible thread state while replay surfaces can rebuild
   branch-local context and merge outcomes from durable records.

## Components

### Thread Candidate Inbox

- Purpose: retain steering prompts received during active turns as structured
  candidates with provenance, timestamps, and thread-local context references.
- Interface: runtime-owned queue/state, not a raw UI-only structure.
- Behavior: supports safe checkpoint evaluation rather than immediate blind
  execution.

### Thread Decision Contract

- Purpose: let the model choose `continue`, `open-thread`, or `merge/reconcile`
  with rationale and stable ids.
- Interface: planner request/response DTOs in the domain/planning boundary.
- Behavior: invalid decisions fail closed to a bounded continue-current-thread
  or defer path rather than mutating thread structure unsafely.

### Thread Recorder Projection

- Purpose: map conversation-layer thread decisions into paddles-owned trace
  records and artifact envelopes before persisting through `TraceRecorder`.
- Interface: reuses the recorder boundary and embedded transit adapter without
  leaking conversation semantics into `transit-core`.
- Behavior: emits explicit branch creation, thread reply, backlink/summary,
  merge decision, and checkpoint records, reusing upstream helper APIs for
  branch metadata, replay views, and artifact descriptors where possible.

### Paddles Conversation Layer

- Purpose: own the conversation/thread semantics needed by the UI and runtime,
  while staying extractable into a shared crate later.
- Interface: paddles-owned thread candidate, decision, replay, and merge DTOs.
- Behavior: translates between planner/TUI needs and the lower-level recorder
  primitives.

### Thread Replay Loader

- Purpose: reconstruct branch-local context and its relationship to the mainline
  for later planning and synthesis.
- Interface: recorder/replay adapter plus runtime state hydration.
- Behavior: avoids hidden mutation by rebuilding from durable records.

### Threaded Transcript Renderer

- Purpose: keep the operator aware of which thread is active and what happened
  to branched work.
- Interface: TUI row/event rendering.
- Behavior: shows split/open/merge outcomes without drowning the transcript in
  raw recorder internals.

## Interfaces

Planned contract additions:

- `ThreadCandidate`
  - steering prompt
  - source turn id
  - active thread id
  - captured-at timestamp
  - optional provenance and citation context

- `ThreadDecision`
  - `continue-current-thread`
  - `open-child-thread`
  - `merge-thread` / `summarize-back`
  - rationale
  - optional target or parent thread id

- Recorder projections
  - branch declared / opened
  - thread reply appended
  - backlink or summary recorded
  - merge decision recorded
  - checkpoint recorded

These stay paddles-owned and should naturally fit into a future extracted crate.
Raw transit DTOs should not leak across the domain boundary.

## Data Flow

```text
1. User submits steering prompt while a turn is active
2. TUI/runtime captures it as ThreadCandidate
3. At a safe checkpoint, planner model receives:
   - AGENTS/foundational interpretation context
   - current thread/mainline state
   - candidate prompt
   - recent evidence / trace summary
4. Model returns bounded ThreadDecision
5. Controller validates decision
6. Runtime:
   - continues current thread, or
   - opens child branch, or
   - records merge/summarize-back intent
7. Conversation-layer records are projected into trace records and artifact
   envelopes, then persisted through TraceRecorder
8. Transcript renders the visible outcome
9. Replay can later rebuild mainline and child-thread context from the same records
```

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Invalid thread decision JSON or schema | Planner parse/validation failure | Fail closed to bounded continue/defer behavior and log an explicit event | Operator can resubmit steering prompt; runtime keeps the candidate lineage |
| Recorder write failure | `TraceRecorder` returns error | Keep live turn execution stable, surface warning, and avoid pretending the thread was durably recorded | Retry or continue with in-memory fallback if available |
| Merge-back target missing or stale | Validation against current thread state | Record explicit failure event and avoid silent history rewrite | Let the model/operator choose a new merge or summary target later |
| Replay payload too large | Artifact envelope threshold crossed | Store through artifact envelope path instead of forcing huge inline transcript data | Later replay fetches the envelope target |
