# Decouple Harness Interfaces From Model Assumptions - Product Requirements

## Problem Statement

Paddles still encodes too many provider-specific transport, rendering, and execution assumptions, so the recursive harness does not generalize cleanly across improving models or interchangeable execution environments.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Promote the session from an optional trace sink into the durable runtime object that survives harness restarts, context-window churn, and replay-driven recovery. | Session replay, slice, and resume semantics are defined and exercised through default runtime recording. | The default runtime can recover from persisted session state rather than treating recording as optional metadata. |
| GOAL-02 | Replace provider-name branching with negotiated capability contracts for planning, rendering, tool calling, and context management. | Core runtime behavior is driven by capabilities rather than static provider-specific conditionals. | New model providers can fit the harness by implementing capability contracts instead of forking controller logic. |
| GOAL-03 | Decouple the recursive brain from the local hands that execute edits, shell work, transport actions, and future execution surfaces. | Workspace/editor/terminal/transport execution all report through a shared hand lifecycle and diagnostics vocabulary. | The harness can swap, recover, and reason about execution hands without baking in one environment shape. |
| GOAL-04 | Make harness policies adaptive instead of stale by introducing explicit harness profiles, session-queryable context slices, and optional specialist brains. | Steering and compaction behavior can vary by profile and model shape without changing Paddles' recursive core semantics. | The runtime can adapt to stronger or weaker models while preserving the same recursive context harness identity. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Primary User | Maintainers extending Paddles across local and remote model providers. | Stable runtime interfaces that let new models fit the harness without provider-specific rewrites. |
| Secondary User | Operators using Paddles across different planner/synthesizer/gatherer combinations. | Consistent recursive behavior, recovery, and observability even when the backing models differ. |
| Tertiary User | Security- and reliability-minded contributors hardening the runtime. | Structural boundaries around credentials, execution hands, and session durability instead of prompt-only safeguards. |

## Scope

### In Scope

- [SCOPE-01] Define a durable session contract with replay, wake/resume, checkpoint, and selective event-slice semantics.
- [SCOPE-02] Promote a persistent recorder path into the default runtime posture for turn/session lineage.
- [SCOPE-03] Define capability-negotiated interfaces for planner actions, final-answer rendering, tool calling, and related harness features.
- [SCOPE-04] Introduce a shared execution-hand lifecycle for workspace editing, terminal commands, transports, and adjacent action surfaces.
- [SCOPE-05] Add structural credential-isolation patterns for local hands and transport/tool mediation.
- [SCOPE-06] Expose session-queryable context slices and adaptive harness profiles for compaction, replay, and model-specific controller behavior.
- [SCOPE-07] Model optional specialist brains as bounded session-scoped capabilities rather than fixed hard-coded runtime assumptions.
- [SCOPE-08] Update docs, diagnostics, tests, and board evidence for the new meta-harness contracts.

### Out of Scope

- [SCOPE-09] Turning Paddles into a hosted managed-agent service or moving execution control out of the local runtime.
- [SCOPE-10] IDE-fed context, editor-specific agent state, or VPC-only infrastructure assumptions.
- [SCOPE-11] Replacing the recursive planner/controller loop with a different product metaphor.
- [SCOPE-12] Generic plugin marketplaces or unrelated protocol expansion beyond what the interface refactor requires.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must expose a stable session interface that supports durable recording, replay, checkpoint-aware recovery, and selective event interrogation outside the active model context window. | GOAL-01, GOAL-04 | must | Durable, queryable session state is the foundation for a meta-harness that survives changing model behavior. |
| FR-02 | Paddles must drive planning, rendering, and tool-call behavior from negotiated runtime capabilities instead of provider-name-specific branches wherever the behavior is conceptually shared. | GOAL-02 | must | Provider-shaped branching goes stale as models improve; capability contracts are the longer-lived abstraction. |
| FR-03 | Paddles must define a shared execution-hand contract that covers lifecycle, provisioning, execution, recovery, and diagnostics for local action surfaces. | GOAL-03 | must | Decoupling the brain from the hands requires a common interface for actions and failures. |
| FR-04 | Credentials and privileged transport/tool state must be mediated so generated code and local shell execution do not receive more authority than required. | GOAL-03 | must | Structural security boundaries generalize better than prompt-only assumptions about what models will not do. |
| FR-05 | The runtime must support explicit harness profiles and session-queryable context slices so steering, compaction, and recovery behavior can adapt across model strengths without changing the recursive core loop. | GOAL-04 | must | Adaptive profiles let the harness retire stale policies while preserving the same underlying product identity. |
| FR-06 | Optional specialist brains must plug into the same session and capability contracts rather than bypassing the recursive planner/controller architecture. | GOAL-02, GOAL-04 | should | Additional brains should expand the harness, not fragment it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain local-first operation for the full refactor. | GOAL-01, GOAL-03, GOAL-04 | must | The meta-harness should stay true to Paddles’ core runtime philosophy. |
| NFR-02 | Keep all new session, hand, capability, and profile states observable through existing trace, diagnostics, and UI projection surfaces. | GOAL-01, GOAL-02, GOAL-03, GOAL-04 | must | The harness should become more general without becoming more opaque. |
| NFR-03 | Preserve backward compatibility at the operator surface while the internal interfaces migrate. | GOAL-02, GOAL-03 | should | Operators should not pay the cost of architectural cleanup through abrupt workflow regressions. |
| NFR-04 | Guard the new contracts with focused tests and board-linked verification so stale harness assumptions can be retired intentionally rather than by accident. | GOAL-01, GOAL-02, GOAL-04 | must | The whole point of the refactor is to make future changes safer and more measurable. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Session durability and interrogation | Recorder, replay, recovery, and context-resolution tests plus runtime proofs | Story-level verification artifacts and voyage compliance reports |
| Capability negotiation | Contract tests across provider/model combinations and shared rendering/planning paths | Story-level verification artifacts and updated runtime docs |
| Execution hands and credential isolation | Adapter tests, controlled runtime diagnostics, and failure-path proofs | Story-level verification artifacts and architecture/configuration updates |
| Adaptive profiles and specialist brains | Controller/profile tests plus replay/compaction proofs across model shapes | Story-level verification artifacts and narrative docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The existing recursive planner/controller loop is the right product essence to preserve while the surrounding interfaces are generalized. | The epic could optimize the wrong architectural boundary. | Voyage one keeps the session/capability work anchored to the existing recursive loop contract. |
| A shared execution-hand model can cover workspace edits, shell work, and transport adapters without collapsing important differences. | The refactor may need smaller interface families instead of one unified hand contract. | Voyage two validates the hand surface against concrete adapters before wider rollout. |
| Harness profiles can retire stale policies without reintroducing provider-specific sprawl. | The profile layer could become another pile of hard-coded exceptions. | Voyage three defines profiles as explicit contracts with verification hooks. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which session behaviors should become mandatory defaults immediately versus gated migrations? | Epic owner | Open |
| How coarse or fine should the execution-hand interface be for transports versus local tools? | Epic owner | Open |
| Which harness-profile dimensions matter most first: model size, latency budget, context budget, or tool reliability? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles runs on a default durable session object instead of treating recording as optional metadata.
- [ ] Core runtime behavior is negotiated through capability contracts rather than provider-name branches where the underlying behavior is conceptually shared.
- [ ] Execution hands expose one observable lifecycle and security posture across local tools and transports.
- [ ] Adaptive harness profiles and session-queryable context slices let the recursive harness generalize across model shapes without losing its local-first recursive identity.
<!-- END SUCCESS_CRITERIA -->
