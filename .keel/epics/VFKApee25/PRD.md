# HTTP API Design For Paddles - Product Requirements

> Paddles can serve its recursive planning harness over HTTP without introducing a new application layer. The existing domain ports (SynthesizerEngine, RecursivePlanner, TraceRecorder, TurnEventSink) are sufficient to power both a chat-style web interface and a real-time trace visualization, with SSE streaming TurnEvents to connected clients as they happen.

## Problem Statement

The CLI and TUI are the only interfaces into the paddles harness today. A web interface broadens access and enables rich visualization that terminals cannot support, specifically a railroad-style DAG view of transit trace streams showing branch/merge lineage, planner actions, and checkpoint status in real time.

The API design must answer: what is the right HTTP surface for a recursive in-context planning harness where turns are long-lived, events stream continuously, and conversation state includes explicit thread branching and merging?

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- [SCOPE-01] Deliver the bearing-backed capability slice for this epic.

### Out of Scope

- [SCOPE-02] Unrelated platform-wide refactors outside bearing findings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the web server share the same MechSuitService instance as the CLI, or run as a separate process? | Planner | Resolved: shared instance, single process |
| How should concurrent sessions be managed when the local model can only serve one inference at a time? | Planner | Resolved: sequential processing, tokio tasks for concurrency |
| What authentication/authorization model (if any) is appropriate for a local-first tool? | Planner | Resolved: none needed for local-first |
| Should trace replay support incremental fetching (cursor-based) for large conversation histories? | Planner | Resolved: full replay for now, cursor later |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Candidate API shape covers session lifecycle, turn submission, event streaming, and trace replay
- [ ] Streaming protocol decision (SSE vs WebSocket) is justified against the event flow model
- [ ] Visualization data model maps TraceRecord DAG to renderable nodes and edges
- [ ] API design respects hexagonal architecture (HTTP adapter, not new application layer)
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

## Findings


- MechSuitService methods map directly to REST endpoints without new application logic [SRC-01]
- TurnEvent Serialize derive enables typed SSE payloads with zero manual serialization [SRC-02]
- TraceRecord lineage DAG provides the graph structure needed for railroad visualization [SRC-03]
- axum integrates with existing tokio runtime and tower middleware at zero friction [SRC-04]


## Opportunity Cost


Minimal. The HTTP server runs as a background tokio task alongside the CLI/TUI. No existing functionality was modified or displaced.


## Dependencies


- tokio async runtime (already present) [SRC-04]
- tower-http for CORS (new, lightweight) [SRC-04]
- axum 0.8 for HTTP framework (new) [SRC-04]


## Alternatives Considered


- WebSocket instead of SSE: rejected because TurnEvent flow is unidirectional (server to client), SSE is HTTP-native with browser auto-reconnection, and no upgrade negotiation is needed [SRC-02]
- Separate process instead of shared MechSuitService: rejected because local model inference is sequential regardless of interface, and sharing the service avoids duplicate model loading [SRC-01]

## Research Provenance

*Source records from bearing evidence:*

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:code-review | src/application/mod.rs | 2026-03-29 | 2026-03-29 | high | high | MechSuitService exposes session lifecycle, turn processing, and event sink injection |
| SRC-02 | manual | manual:code-review | src/domain/model/turns.rs | 2026-03-29 | 2026-03-29 | high | high | TurnEvent enum defines 16 typed event variants suitable for SSE streaming |
| SRC-03 | manual | manual:code-review | src/domain/model/traces.rs | 2026-03-29 | 2026-03-29 | high | high | TraceRecord with lineage DAG provides visualization graph structure |
| SRC-04 | manual | manual:code-review | Cargo.toml | 2026-03-29 | 2026-03-29 | high | high | tower 0.5 and tokio 1.43 already present as dependencies |

---

*This PRD was seeded from bearing `VFKApee25`. See `bearings/VFKApee25/` for original research.*
