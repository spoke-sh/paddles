# Coherent Rendering And Recursive Harness Boundaries - Product Requirements

## Problem Statement

Render structure is flattened and rebuilt heuristically, recursive loop ownership is split between the harness and adapters, and projection/read-model concerns leak into the domain, creating stream rendering bugs and architectural drift.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Canonical render truth: persist typed authored responses and project the same render document across live streams, durable completion records, and replayed transcripts. | Live turn output and replayed transcript projection converge without heuristic render reconstruction | Voyage 1 |
| GOAL-02 | Single recursive control plane: route workspace actions, retries, and replanning through one application-owned recursive harness instead of nested adapter loops. | No model adapter owns an independent tool-execution loop for repository actions | Voyage 2 |
| GOAL-03 | Hexagonal projection boundaries: keep domain logic free of UI read models and presentation formatting while decomposing orchestration into chamber-aligned application services. | Projection/presentation concerns no longer live in `domain/model`, and `MechSuitService` no longer acts as a monolith | Voyage 3 |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer using paddles through TUI, web runtime, or CLI while watching live stream output | One coherent answer/rendering stream that does not disagree with replayed or cross-surface state |
| Maintainer | Engineer evolving recursive planning, rendering, and runtime surfaces | Stable architectural seams so rendering bugs can be isolated without spelunking cross-layer heuristics |

## Scope

### In Scope

- [SCOPE-01] Persisting typed authored responses and render documents in durable completion paths
- [SCOPE-02] Canonical projection update and replay rules for transcript/stream convergence
- [SCOPE-03] Moving workspace action execution under an explicit application-owned recursive control plane
- [SCOPE-04] Separating response authoring from workspace mutation ports
- [SCOPE-05] Moving read-model and presentation formatting concerns out of the domain model
- [SCOPE-06] Decomposing `MechSuitService` into chamber-aligned orchestration seams where needed to support the refactor
- [SCOPE-07] Tests and proofs that live streaming, replay, and projection state converge on the same render truth

### Out of Scope

- [SCOPE-08] New end-user browser features unrelated to render/projection correctness
- [SCOPE-09] New hosted services or remote orchestration for planner/runtime execution
- [SCOPE-10] Broad model-behavior tuning unrelated to render persistence or loop ownership
- [SCOPE-11] Replacing trace/forensic/manifold product concepts with a new visualization paradigm

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The final-answer path must preserve a typed authored response/render document through synthesis, trace recording, transcript replay, and surface projection. | GOAL-01 | must | Eliminates live/replay drift caused by flattening structured answers into prose and rebuilding them later. |
| FR-02 | Stream and replay surfaces must derive transcript/render state from one canonical projection contract rather than interface-local reconstruction heuristics. | GOAL-01 | must | Rendering bugs often appear when live and replay paths disagree about truth ownership. |
| FR-03 | Workspace actions selected during recursive planning must execute through an application-owned execution boundary instead of a synthesizer-authoring port. | GOAL-02 | must | Keeps mutation and control in the harness rather than in response-authoring adapters. |
| FR-04 | Model adapters must not run independent recursive tool loops for repository actions once the application harness owns recursive execution. | GOAL-02 | must | Competing loops create inconsistent budgets, stop conditions, and rendering side effects. |
| FR-05 | Projection read models for transcript, manifold, forensics, or related stream surfaces must be application-owned or infrastructure-owned, not domain-owned. | GOAL-03 | must | Keeps the domain model focused on invariants instead of surface-specific projections. |
| FR-06 | Turn orchestration responsibilities must be decomposed enough that rendering/projection changes do not require editing a monolithic application service. | GOAL-03 | should | Reduces regression risk and makes chamber ownership explicit. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The refactor must preserve local-first execution and add no new remote dependency for rendering or control-plane correctness. | GOAL-01, GOAL-02, GOAL-03 | must | Architectural cleanup cannot violate the product's local-first constraint. |
| NFR-02 | Rendering, projection, and recursive-control changes must remain observable through existing trace, transcript, manifold, and forensic surfaces. | GOAL-01, GOAL-02 | must | Operators need the same or better visibility while ownership changes underneath. |
| NFR-03 | The new seams must be testable with deterministic replay and contract tests that compare live-streamed and replayed outcomes. | GOAL-01, GOAL-03 | must | Prevents future rendering regressions from hiding behind UI-only behavior. |
| NFR-04 | Transitional steps must avoid long-lived compatibility layers that preserve duplicate ownership of render truth or recursive control. | GOAL-01, GOAL-02, GOAL-03 | should | Duplicate ownership is the current failure mode; the rollout should trend toward one canonical path. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Render truth | Contract/unit tests plus replay/live projection comparison evidence | Story-level test evidence and verification notes |
| Recursive control ownership | Architectural review plus execution-path tests showing one control plane | Story-level test and review evidence |
| Boundary cleanup | Code review and targeted tests over extracted read models/services | Story-level review evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing trace model can carry typed rendered responses without requiring a separate durable store. | We may need a broader trace artifact extension before render convergence can land. | Validate in Voyage 1. |
| The current adapter-owned loops can be retired without losing essential model-provider capabilities. | We may need a narrower adapter abstraction or staged executor split. | Validate in Voyage 2. |
| Projection and presentation code can move out of `domain/model` without destabilizing recorder or replay semantics. | We may need an intermediate application read-model package first. | Validate in Voyage 3. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should projection updates become versioned deltas, invalidation tokens, or reducer events as the canonical stream contract? | Application/runtime owner | Open |
| How much service extraction is necessary to make `MechSuitService` coherent without causing churn for every call site? | Architecture owner | Open |
| Is any provider-specific tool/retry behavior still valuable after removing nested adapter loops? | Model integration owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The same typed assistant render document can be observed in live output and replayed transcript projection for a completed turn
- [ ] No model adapter owns an independent repository tool loop once the recursive harness control plane is refactored
- [ ] Projection and presentation types for stream/transcript surfaces no longer live in `domain/model`
- [ ] At least one active implementation slice exists with explicit story-level verification paths for render truth, control ownership, and boundary cleanup
<!-- END SUCCESS_CRITERIA -->
