# Model-Driven Thread Split And Merge UX - SRS

## Summary

Epic: VFHIUOcFc
Goal: Route steering prompts during active turns through a model-driven thread decision loop backed by transit lineage, explicit thread/merge artifacts, and a thread-aware operator experience.

## Scope

### In Scope

- [SCOPE-01] Define a bounded thread decision contract that lets the model classify a steering prompt as continue-current-thread, open-child-thread, or merge/reconcile work back to the mainline.
- [SCOPE-02] Capture steering prompts during active turns as structured thread candidates with enough context for later replay, citation, and thread selection.
- [SCOPE-03] Add a paddles-owned conversational threading layer or crate that persists explicit thread branch, reply, backlink/summary, merge decision, and checkpoint records through the existing recorder boundary and embedded `transit-core`.
- [SCOPE-04] Render thread-aware operator feedback in the default transcript, including split, active-thread, and merge-back states.
- [SCOPE-05] Document and prove the delivered behavior and remaining concurrency limits.

### Out of Scope

- [SCOPE-06] Hardcoded Keel-specific thread logic or board-only first-class runtime intents.
- [SCOPE-07] Mandatory remote `transit` services or a server-only threading design.
- [SCOPE-08] Unlimited concurrent local generation on a single runtime session without checkpoints or bounded control.
- [SCOPE-09] A universal thread/merge policy for all future product surfaces beyond steering prompts during active turns.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | A thread decision contract exists for steering prompts and can express continue current thread, open child thread, and merge/reconcile intent with rationale and stable identifiers. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Steering prompts received during an active turn are retained as structured thread candidates rather than opaque queue strings, with enough source context to support later replay and citations. | SCOPE-02 | FR-02 | manual |
| SRS-03 | A paddles-owned conversational threading layer can persist thread branch creation, thread replies, backlinks or summaries, merge decisions, and completion checkpoints through embedded `transit-core` without leaking raw transit types into domain interfaces. | SCOPE-03 | FR-03 | manual |
| SRS-04 | Thread-local replay reconstructs enough branch-local and mainline provenance for the planner/synthesizer to continue work inside a child thread coherently. | SCOPE-02, SCOPE-03 | FR-04 | manual |
| SRS-05 | The default transcript and turn-event UX surface thread split, active-thread state, and merge-back outcomes clearly enough for an operator to follow them live. | SCOPE-04 | FR-05 | manual |
| SRS-06 | Merge-back behavior is represented through explicit recorded outcomes rather than hidden history rewrites, and the transcript renders that outcome clearly. | SCOPE-03, SCOPE-04 | FR-06 | manual |
| SRS-07 | Foundational docs and proof artifacts explain the thread decision contract, transit lineage mapping, merge-back semantics, and remaining concurrency limits honestly. | SCOPE-05 | FR-07 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Auto-threading remains local-first, bounded, and fail closed when model decisions or recorder writes are invalid or unavailable. | SCOPE-01, SCOPE-03 | NFR-01 | manual |
| SRS-NFR-02 | The thread contract and recorder mapping remain generic across evidence domains instead of assuming Keel or any one repository layout. | SCOPE-01, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-03 | The operator-facing transcript remains concise and scannable even though the underlying thread lineage and merge data are richer. | SCOPE-04 | NFR-03 | manual |
| SRS-NFR-04 | The first delivered implementation works with embedded `transit-core`, does not require a separate trace server, and keeps the conversation/thread layer extractable from paddles later. | SCOPE-03 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
