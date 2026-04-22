# Chamber Services And Read-Model Boundaries - Software Design Description

> Decompose orchestration into chamber-aligned application services and move projection and presentation concerns out of the domain model.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the current monolithic turn service into a composed
application boundary with explicit chamber ownership and moves projection logic
to a read-model layer. The domain model continues to define events, traces, and
invariants, while transcript/forensic/manifold projections and runtime
presentation formatting become application or infrastructure concerns.

## Context & Boundaries

### In Scope

- chamber-aligned application service extraction
- application-owned conversation read models
- presentation projector relocation out of the domain model

### Out of Scope

- replacing recorder infrastructure
- changing product semantics for transcript/manifold/forensics
- new remote services for projection storage or rendering

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│ domain events/traces -> application read models -> infrastructure │
│ chamber services --------------------------------> compose turn     │
│                                                                    │
│ domain keeps invariants; surfaces get projectors                    │
└────────────────────────────────────────────────────────────────────┘
          ↑                                         ↑
      domain core                              TUI / web
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `MechSuitService` and current turn orchestration code | internal | Source of responsibilities to split into chambers | current |
| Existing conversation projection types | internal | Behavior and data shape to preserve while moving ownership | current |
| TUI/web adapters | internal | Consumers of runtime presentation/projector output | current |
| Trace recorder and replay model | internal | Authoritative substrate for application read models | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Service extraction style | Chamber-aligned services/modules under the application layer | Matches the repo's stated harness architecture |
| Projection ownership | Application read-model boundary | Projections are read-side composition, not domain invariants |
| Presentation ownership | Infrastructure/projector layer | Surface strings and badges are not domain semantics |
| Compatibility stance | Preserve outputs while moving ownership | Keeps the refactor measurable and low-risk |

## Architecture

1. Domain types continue to model events, traces, and invariants.
2. Application chamber services coordinate interpretation, routing, recursive
   control, synthesis, and read-model projection.
3. Conversation projection types move under an application read-model boundary.
4. Runtime event presentation moves behind projector helpers owned outside the
   domain core.
5. TUI and web keep consuming equivalent projected data through the new seams.

## Components

`TurnOrchestrationFacade`
: Top-level application service that composes smaller chamber services.

`InterpretationRouteAndLoopServices`
: Chamber-aligned application services extracted from the current monolith.

`ConversationReadModel`
: Application-owned projection layer for transcript, forensics, manifold, and
trace graph views.

`RuntimePresentationProjector`
: Surface-oriented formatter that turns typed runtime events into UI-facing
labels/details outside the domain model.

## Interfaces

Candidate internal interfaces:

- `replay_conversation_projection(task_id) -> ConversationProjectionSnapshot`
- `projection_update_for_* (...) -> ConversationProjectionUpdate`
- `project_runtime_event(event) -> RuntimeEventPresentation`

Ownership change:

- type signatures may stay familiar, but the owning module/package moves out of
  `domain/model`

## Data Flow

1. Domain events and trace records are recorded as before.
2. Application read models replay those records into conversation projections.
3. Infrastructure projectors format runtime items for TUI/web presentation.
4. The top-level application facade composes chamber services rather than
   directly implementing every responsibility.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Projection ownership move changes output shape | Snapshot or contract tests fail | Stop the move and preserve compatibility shims temporarily | Align the new read-model boundary with the previous output contract |
| Domain types still embed presentation strings after refactor | Review or compile-path inspection finds projectors still in `domain/model` | Keep the story open until ownership is clean | Finish moving formatters behind the projector boundary |
| Chamber extraction leaves the top-level facade still monolithic | Review shows unchanged responsibility concentration | Treat as incomplete slice | Continue extraction until chamber seams are explicit |
