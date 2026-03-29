# Transit-Backed Auto-Threading Harness - Product Requirements

## Problem Statement

Paddles can now keep the interactive composer live during active turns, but steering prompts are only queued in memory. The runtime cannot yet let the model decide whether a new steering prompt belongs on the current conversational thread or should open a child branch with explicit lineage, replayable artifacts, and merge-back semantics in the operator UX.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make steering prompts during active turns model-classified thread decisions instead of opaque queue entries. | The runtime can ask the model whether a steering prompt should continue the current thread, open a child branch, or wait for later merge/reconciliation | Verified planner contract, runtime tests, and transcript proof |
| GOAL-02 | Introduce a paddles-owned conversational threading layer that persists thread structure durably through the existing recorder boundary using embedded `transit-core`. | Thread creation, replies, backlinks, summaries, merge decisions, and checkpoints are recorded with stable lineage identifiers and can be replayed without hidden side tables, while the conversation API remains extractable from paddles later | Verified recorder tests and replay proof |
| GOAL-03 | Give operators a thread-aware transcript and merge-back UX. | The default TUI shows mainline/thread state clearly enough to follow split, active, and merged work without losing grounded turn provenance | Verified TUI tests and transcript proof |
| GOAL-04 | Keep the integration generic, local-first, and honestly documented. | Foundational docs describe model-driven auto-threading as a recursive-harness capability, not a Keel-specific special case, and runtime fallbacks remain bounded and explicit | Verified docs and proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer steering `paddles` interactively while a local model is already working on a turn. | The ability to interrupt, refine, or branch work without waiting for the current response to finish or losing the context of earlier branches. |
| Runtime Maintainer | An engineer evolving the recursive harness, recorder boundary, TUI, and future extractable conversation layer. | A generic thread contract that composes with planning, evidence gathering, and recorder/replay surfaces without forcing conversation-specific APIs down into `transit-core`. |
| Small-Model Router | The person trying to lift smaller local models through better harness structure. | A model-driven thread split/merge loop that uses recorded context, summaries, and replay to improve coherence without forcing a larger default model. |

## Scope

### In Scope

- [SCOPE-01] Define a model-facing thread decision contract for steering prompts that can express continue-current-thread, open-child-thread, and merge/reconcile intent with explicit rationale.
- [SCOPE-02] Capture steering prompts during active turns as first-class thread candidates and route them through a bounded model-driven decision loop at safe checkpoints.
- [SCOPE-03] Create a paddles-owned conversational threading layer or crate that persists thread lineage, replies, backlinks, summaries, merge decisions, and checkpoints through the existing `TraceRecorder` boundary and embedded `transit-core` adapter.
- [SCOPE-04] Make thread-local replay and merge-back visible in the operator transcript and turn-event surfaces.
- [SCOPE-05] Update foundational docs and proof artifacts so the auto-threading architecture and current concurrency limits are legible.

### Out of Scope

- [SCOPE-06] Hardcoded Keel-specific thread routing or board-specific first-class runtime intents.
- [SCOPE-07] Mandatory remote `transit` server deployment for local threaded interaction.
- [SCOPE-08] Arbitrary concurrent generation on a single local model session without safe checkpoints or bounded control.
- [SCOPE-09] A universal merge strategy for every future workflow beyond the explicit thread-backlink/summary/merge behavior needed for steering prompts.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must expose a model-facing thread decision contract for steering prompts that can express at least continue current thread, open child thread, and merge/reconcile intent, with controller validation and bounded fallback. | GOAL-01, GOAL-04 | must | The model should own the classification decision, but the runtime still needs a constrained, safe contract. |
| FR-02 | Steering prompts received during an active turn must be retained as structured thread candidates rather than opaque in-memory queue entries, including enough context for later decision, replay, and citation. | GOAL-01, GOAL-02 | must | Auto-threading depends on durable candidate context, not just ephemeral UI input. |
| FR-03 | The first implementation must introduce a paddles-owned conversational threading layer that records explicit thread branch creation, thread replies, backlinks/summaries, merge decisions, and completion checkpoints with stable lineage identifiers while keeping raw `transit-core` usage behind the recorder boundary and reusing upstream metadata/replay/artifact helpers where they fit. | GOAL-02 | must | Hidden thread state would break replay and reduce trust in the threaded harness, baking conversation APIs into `transit-core` would blur the storage/application boundary, and reusing new low-level helpers should reduce unnecessary custom plumbing. |
| FR-04 | Thread-local replay must provide enough context for the planner/synthesizer to continue work inside a child thread without forgetting the mainline provenance that created it. | GOAL-01, GOAL-02 | must | Branches need reusable memory, not just archival storage. |
| FR-05 | The operator UX must show when work remains on the mainline thread, when it has branched into a child thread, and how merge-back or summary-to-mainline behavior is being applied. | GOAL-03 | must | Auto-threading that is invisible or inscrutable will feel like lost context rather than improved steering. |
| FR-06 | Merge-back behavior must be represented explicitly through recorded artifacts or trace records instead of rewriting thread history, and the transcript must render the outcome clearly. | GOAL-02, GOAL-03 | must | The mainline needs a trustworthy reconciliation story that remains replayable. |
| FR-07 | Foundational docs and proof artifacts must explain the thread decision contract, transit lineage mapping, thread replay behavior, merge-back UX, and remaining concurrency limits honestly. | GOAL-04 | must | The architecture needs to be legible without reverse-engineering the code. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Auto-threading must remain local-first, bounded, and fail closed when the model returns invalid thread decisions or the recorder path is unavailable. | GOAL-01, GOAL-04 | must | Threading should strengthen the harness without weakening runtime safety. |
| NFR-02 | The thread contract must remain generic across evidence domains and repositories rather than assuming Keel or any other one project layout. | GOAL-01, GOAL-04 | must | This should improve the generic recursive harness, not add another special-case mode. |
| NFR-03 | The default transcript and event stream must stay readable even when thread lineage, merge summaries, and replay metadata become richer. | GOAL-03, GOAL-04 | should | Operators need more visibility, not more noise. |
| NFR-04 | Embedded `transit-core` must be sufficient for the first end-to-end implementation; the design must not require a separate trace server and must leave the conversation/thread layer extractable from paddles later. | GOAL-02, GOAL-04 | should | Local operators should get the feature without extra infrastructure, and later extraction should not require rewriting the whole contract. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Thread decision contract | Unit tests and planner prompt/protocol review | Story evidence for valid continue/open/merge decisions and bounded fallback |
| Conversation layer + transit-backed recording and replay | Recorder tests plus replay proof | Story evidence for the paddles-owned conversation layer, branch creation, thread reply, backlink/summary, merge, and checkpoint records |
| Thread-aware operator UX | TUI tests and transcript proofs | Story evidence showing thread split, active child thread, and merge-back rendering |
| Docs and architecture guidance | Manual review plus proof artifact | Updated foundational docs and an auto-threading transcript/replay proof |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A model-driven thread decision contract will outperform a hardcoded thread heuristic for steering prompts in the current harness. | The runtime may need more fallback logic or a stronger planner lane than expected. | Validate with transcript proofs and runtime tests. |
| The current `TraceRecorder` boundary is sufficient to support a paddles-owned conversation layer without leaking raw transit types into domain code. | The mission may need to stop at a domain-contract refactor before full auto-threading. | Validate during story decomposition and recorder tests. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should merge-back default to a summary/backlink artifact, a stronger explicit merge artifact, or a model-selected choice between them based on the branch outcome? | Runtime maintainer | Open |
| How frequently can the runtime safely checkpoint for thread decisions without making the active-turn experience feel sluggish? | Runtime maintainer | Open |
| Which subset of thread-local context should be promoted back to the mainline by default so small models gain continuity without excessive duplication? | Runtime maintainer | Open |
| Upstream `transit-core` now ships stronger branch metadata, replay-view, and artifact helper APIs; the mission should consume those primitives where they simplify the paddles-owned conversation layer instead of recreating them locally. | Runtime maintainer | Acknowledged |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Steering prompts received during active turns are handled through a model-driven thread decision contract instead of an opaque queue-only UX.
- [ ] The first end-to-end implementation records explicit thread lineage and merge-back artifacts through embedded `transit-core`.
- [ ] The default transcript makes thread split, active-thread state, and merge-back behavior legible to the operator.
- [ ] Foundational docs explain the architecture, recorder mapping, and current limitations honestly.
<!-- END SUCCESS_CRITERIA -->
