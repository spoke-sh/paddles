# Transit-Aligned Turn Recording - Product Requirements

## Problem Statement

Paddles currently renders turn activity to interactive surfaces but does not preserve a stable lineage-aware trace contract or recorder boundary, leaving future transit integration dependent on UI prose and ephemeral state.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define a stable `paddles` trace contract for recursive turns, branches, tool activity, and checkpoints. | The runtime has a typed trace model with stable ids and lineage semantics that does not depend on UI prose | Verified contracts and tests |
| GOAL-02 | Separate durable trace recording from transcript rendering. | `TurnEventSink` remains a renderer-facing surface while a new recorder port persists the same turn lineage through a structured trace boundary | Verified architecture and tests |
| GOAL-03 | Prepare large turn payloads for future durable storage without forcing everything inline. | Artifact envelope support exists for large prompts, tool outputs, model outputs, and graph traces | Verified contract/tests |
| GOAL-04 | Prove that `paddles` can record turns locally through embedded `transit-core`. | An embedded recorder adapter and replay proof work without requiring a networked `transit` server | Verified proof and runtime tests |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using `paddles` interactively with local models and recursive planning enabled. | Durable turn and branch history that can be replayed and audited without reverse-engineering transcript text. |
| Runtime Maintainer | An engineer evolving planner, gatherer, and synthesis boundaries. | A clean recorder seam that can persist typed runtime state without coupling core logic to one storage implementation. |
| Evaluator / Researcher | A person comparing retries, branches, tool paths, or graph-mode retrieval outcomes. | Stable lineage-aware traces that make branch comparison and replay practical. |

## Scope

### In Scope

- [SCOPE-01] Define a `paddles`-owned trace contract covering task roots, planner branches, tool request/result pairs, merge-or-selection artifacts, and completion checkpoints.
- [SCOPE-02] Replace string-heavy branch and turn state with recorder-ready structured trace data inside the runtime.
- [SCOPE-03] Introduce a `TraceRecorder` port with `noop` and in-memory adapters separate from `TurnEventSink`.
- [SCOPE-04] Add artifact-envelope support for large prompts, model outputs, tool outputs, and rich graph traces.
- [SCOPE-05] Add an embedded `transit-core` recorder adapter plus replay proof.
- [SCOPE-06] Update foundational docs to explain the recorder boundary, embedded/server distinction, and transit alignment.

### Out of Scope

- [SCOPE-07] Making a networked `transit` server a required runtime dependency for `paddles`.
- [SCOPE-08] Replacing the gatherer boundary or graph-mode mission with recorder work in this slice.
- [SCOPE-09] Full cross-run analytics, multi-user shared storage, or a complete branch-merge UI.
- [SCOPE-10] Leaking raw `transit` kernel/server/client types through `paddles` domain contracts.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | `paddles` must define a typed trace contract that can represent task roots, planner branches, tool request/result pairs, evaluator or selection artifacts, and completion checkpoints with stable machine-readable identifiers. | GOAL-01 | must | Transit-style durable recording depends on stable lineage-aware entities, not transcript text. |
| FR-02 | Planner loop state and turn execution state must preserve structured branch and lineage data instead of relying on string-only pending branches or renderer-oriented summaries. | GOAL-01, GOAL-02 | must | Recorder integration should not need to reconstruct structure from prose. |
| FR-03 | The runtime must expose a `TraceRecorder` port distinct from `TurnEventSink`, with `noop` and in-memory adapters available before storage-specific integration. | GOAL-02 | must | Rendering and durable recording are different responsibilities and should stay separated. |
| FR-04 | The trace contract must support artifact envelopes for large prompts, model outputs, tool outputs, and rich graph traces through logical references and inline metadata rather than assuming inline-only payloads forever. | GOAL-03 | must | Larger AI payloads should remain replayable without bloating the hot append path. |
| FR-05 | `paddles` must be able to record traces through embedded `transit-core` without requiring a networked `transit` server, using a mapping that preserves lineage semantics without leaking raw `transit` types across the domain boundary. | GOAL-04 | must | The first durable path should stay local-first and storage-backed, not service-dependent. |
| FR-06 | Foundational docs and proof artifacts must explain the recorder boundary, transit alignment, embedded/server distinction, and remaining gaps honestly. | GOAL-04 | must | Operators and maintainers need a stable mental model before the runtime grows around it. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Recording must remain local-first, bounded, and fail closed when a recorder adapter is unavailable or storage integration fails. | GOAL-02, GOAL-04 | must | Durable traces cannot destabilize live turn execution. |
| NFR-02 | The `paddles` trace contract must remain generic across repositories and evidence domains rather than overfitting to Keel or one workflow. | GOAL-01, GOAL-02 | must | The recorder boundary should strengthen the mech suit generally, not one product integration. |
| NFR-03 | Operator-facing transcripts and event streams must remain observable and concise even after durable trace data becomes richer than the rendered UI. | GOAL-02, GOAL-04 | should | The recorder should add structure without degrading the interactive experience. |
| NFR-04 | The trace and artifact contracts must stay storage-model-neutral inside `paddles` even when the first adapter targets embedded `transit-core`. | GOAL-03, GOAL-04 | should | Preserves future flexibility and keeps the domain boundary clean. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Trace contract + runtime state | Unit tests, contract review, and story-level proofs | Story evidence showing stable ids, structured branches, and recorder-ready runtime data |
| Recorder boundary | Tests plus in-memory proof | Story evidence showing `TraceRecorder` usage independent of transcript rendering |
| Embedded transit proof | Runtime proof and replay verification | Story evidence showing local recording and replay through embedded `transit-core` |
| Docs and mental model | Doc review plus proof artifact | Updated foundational docs and recorder-boundary explanation |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| `transit`'s current embedded lineage model is mature enough to serve as the first durable recorder backend for `paddles`. | The mission may need to stop at a recorder boundary plus in-memory proof. | Validate through compile/runtime proof work. |
| A `paddles`-owned trace contract is worth maintaining instead of writing directly against `transit` record shapes. | The mission may add unnecessary mapping overhead. | Validate by keeping the first adapter small and the domain types stable. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much of the current turn-event surface should be represented as durable trace entities versus renderer-only projections? | Runtime maintainer | Open |
| Which merge/selection artifacts should be first-class in the first recorder slice, and which can remain deferred? | Runtime maintainer | Open |
| When graph-mode gatherers land, should graph trace payloads stay inline or immediately use artifact envelopes? | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles` has a typed recorder-ready trace contract with stable lineage identifiers.
- [ ] Runtime state and event emission no longer depend on UI prose as the only representation of recursive work.
- [ ] Embedded `transit-core` can record and replay representative `paddles` turns locally.
- [ ] Foundational docs explain the recorder boundary and its current limits clearly.
<!-- END SUCCESS_CRITERIA -->
