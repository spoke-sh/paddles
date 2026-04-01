# Canonical Transcript Plane For Cross-Surface Conversations - Product Requirements

## Problem Statement

TUI, web, and CLI turns already share a central execution path, but each surface still assembles transcript state differently through local UI rows, progress events, and replay hacks. We need one canonical conversation-scoped transcript plane that every prompt source writes to and every surface reads from, with progress kept separate from transcript state.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Canonical transcript projection: prompts and final replies for a conversation come from one durable application-owned transcript model. | Same conversation renders the same transcript in TUI and web without reload or per-surface repair logic | First voyage |
| GOAL-02 | Shared conversation attachment: turns submitted from any attached interface land in the same conversation identity. | Prompt entered in one surface appears in the other surface's transcript for that conversation | First voyage |
| GOAL-03 | Clear plane separation: progress remains live telemetry, not transcript truth. | Transcript hydration no longer depends on `synthesis_ready`, SSE-only heuristics, or UI-local append paths | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer using paddles through TUI, web UI, or both at once | One coherent conversation transcript regardless of which surface submitted the prompt |
| Maintainer | Engineer evolving the interface stack | A clean architecture where transcript state and progress telemetry are separated and debuggable |

## Scope

### In Scope

- [SCOPE-01] Shared conversation identity and attachment rules for cross-surface prompt entry
- [SCOPE-02] Application-owned transcript replay/projection for a single conversation using durable trace-backed records
- [SCOPE-03] Transcript update notifications or deltas that are separate from `TurnEvent` progress telemetry
- [SCOPE-04] TUI and web adoption of the canonical transcript plane, including bootstrap and live-update paths
- [SCOPE-05] Removal of current transcript repair heuristics once the canonical plane is authoritative

### Out of Scope

- [SCOPE-06] Multi-user remote synchronization across machines or accounts
- [SCOPE-07] Redesigning trace DAG visualization or step telemetry beyond the transcript/progress split
- [SCOPE-08] Planner, gatherer, or synthesis behavior changes unrelated to transcript state unification
- [SCOPE-09] Collaborative merge/conflict semantics for simultaneous editing from multiple operators

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The application layer must expose conversation-scoped transcript replay that returns user prompts and assistant completions from durable trace-backed records. | GOAL-01 | must | Makes the transcript plane explicit and shared instead of reconstructing it inside each UI. |
| FR-02 | Prompt entry from TUI, web, or CLI must target a stable conversation identity that other surfaces can attach to. | GOAL-02 | must | Without shared identity, transcript projection cannot unify across interfaces. |
| FR-03 | The system must emit transcript update notifications or deltas independently of `TurnEvent` progress events. | GOAL-01, GOAL-03 | must | Prevents transcript visibility from depending on progress-event timing or race-prone inference. |
| FR-04 | The web UI must render transcript history and live updates from the canonical conversation transcript plane. | GOAL-01, GOAL-02 | must | Replaces DOM-local transcript truth with the shared projection. |
| FR-05 | The TUI must render transcript history and live updates from the canonical conversation transcript plane. | GOAL-01, GOAL-02 | must | Makes the terminal view converge with the browser view for the same conversation. |
| FR-06 | Progress events remain available for live trace/activity display but are no longer required to create prompt/response transcript rows. | GOAL-03 | should | Preserves observability while fixing the architectural seam. |
| FR-07 | Existing transcript repair paths and replay-after-progress heuristics must be removed or retired once the shared transcript plane is in use. | GOAL-03 | should | Reduces race conditions and architectural duplication after migration. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Cross-surface transcript updates must become visible without manual reload, restart, or operator-triggered replay. | GOAL-01, GOAL-02 | must | The primary failure mode today is stale transcript state unless a surface refreshes. |
| NFR-02 | `process_prompt_in_session_with_sink(...)` remains the canonical execution path for turn processing. | GOAL-01, GOAL-02 | must | Avoids splitting turn execution while transcript architecture changes. |
| NFR-03 | The new transcript plane must preserve local-first operation and add no external service dependency. | GOAL-01, GOAL-02 | must | Keeps the runtime aligned with repo architecture constraints. |
| NFR-04 | Migration from interface-local transcript state to the shared transcript plane must be debuggable and recoverable via conversation-scoped replay. | GOAL-01, GOAL-03 | should | Helps surfaces recover from missed live updates without reintroducing cross-surface hacks. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Application transcript plane | Unit/integration tests around transcript replay and update delivery | Story-level test evidence |
| Cross-surface behavior | Manual TUI/web session proof using the same conversation identity | Operator verification logs or screenshots |
| Architectural cleanup | Review of removed repair heuristics and explicit transcript/progress boundaries | Story-level review evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing trace recorder already captures enough durable prompt/response information to seed the canonical transcript plane. | We may need a dedicated transcript journal instead of a trace-backed projection. | Validate during the first application-layer story. |
| A stable conversation identity can be shared across surfaces without redesigning the session model from scratch. | We may need a broader session attachment refactor before UI unification lands. | Validate during application-layer design and implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should transcript updates be delivered as explicit delta events, versioned invalidation signals, or replay tokens? | Application layer | Open |
| How should TUI and web choose or expose the shared conversation identity when both can initiate turns? | UX / interfaces | Open |
| Is the trace model sufficient as the durable transcript substrate, or do we need a dedicated transcript journal later? | Architecture | Open |
| How much optimistic local rendering should remain once surfaces subscribe to the canonical transcript plane? | Interfaces | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A prompt entered in the TUI appears in the web transcript for the same conversation without reloading either surface
- [ ] A prompt entered in the web UI appears in the TUI transcript for the same conversation without restarting or replay hacks
- [ ] Both surfaces render the same prompt/response transcript for a shared conversation when replayed from the application layer
- [ ] Progress events remain available for trace visualization, but transcript hydration no longer depends on them
<!-- END SUCCESS_CRITERIA -->
