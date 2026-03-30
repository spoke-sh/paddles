# VOYAGE REPORT: Model-Driven Thread Split And Merge UX

## Voyage Metadata
- **ID:** VFHIV59Hn
- **Epic:** VFHIUOcFc
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 5/5 stories complete

## Implementation Narrative
### Define Thread Decision And Merge Contract
- **ID:** VFHIX0DKd
- **Status:** done

#### Summary
Define the paddles-owned contract for model-driven steering-prompt threading so
the runtime can distinguish between continuing the current thread, opening a
child thread, and reconciling work back to the mainline without falling back to
product-specific heuristics.

#### Acceptance Criteria
- [x] A bounded thread decision contract exists for steering prompts and can express continue current thread, open child thread, and merge/reconcile outcomes with rationale and stable ids. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Merge/reconcile intent is part of the same bounded contract rather than an ad hoc later-stage escape hatch. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] The contract remains generic across evidence domains and does not encode Keel-specific thread types. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFHIX0DKd/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFHIX0DKd/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFHIX0DKd/EVIDENCE/ac-3.log)
- [llm-judge-a-bounded-thread-decision-contract-exists-for-steering-prompts-and-can-express-continue-current-thread-open-child-thread-and-merge-reconcile-outcomes-with-rationale-and-stable-ids.txt](../../../../stories/VFHIX0DKd/EVIDENCE/llm-judge-a-bounded-thread-decision-contract-exists-for-steering-prompts-and-can-express-continue-current-thread-open-child-thread-and-merge-reconcile-outcomes-with-rationale-and-stable-ids.txt)
- [llm-judge-merge-reconcile-intent-is-part-of-the-same-bounded-contract-rather-than-an-ad-hoc-later-stage-escape-hatch.txt](../../../../stories/VFHIX0DKd/EVIDENCE/llm-judge-merge-reconcile-intent-is-part-of-the-same-bounded-contract-rather-than-an-ad-hoc-later-stage-escape-hatch.txt)
- [llm-judge-the-contract-remains-generic-across-evidence-domains-and-does-not-encode-keel-specific-thread-types.txt](../../../../stories/VFHIX0DKd/EVIDENCE/llm-judge-the-contract-remains-generic-across-evidence-domains-and-does-not-encode-keel-specific-thread-types.txt)

### Persist Transit Thread Branches And Artifacts
- **ID:** VFHIX0uKc
- **Status:** done

#### Summary
Create the paddles-owned conversation/thread layer and project its thread
creation, reply, backlink/summary, merge, and checkpoint transitions through
the existing recorder boundary so embedded `transit-core` can durably replay
threaded work without turning `transit-core` into a conversation API.

#### Acceptance Criteria
- [x] A paddles-owned conversation/thread layer exists above the recorder boundary and owns the thread DTOs needed by runtime and UX code. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] The layer consumes the new upstream `transit-core` metadata, branch replay, and artifact helper APIs where they simplify low-level plumbing, without turning `transit-core` into the conversation API boundary. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Thread-local replay reconstructs enough mainline and child-thread provenance for later planning and synthesis. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] The implementation works through the existing embedded recorder path, does not require a separate trace server, and remains extractable from paddles later. [SRS-NFR-04/AC-03] <!-- verify: manual, SRS-NFR-04:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFHIX0uKc/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFHIX0uKc/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFHIX0uKc/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VFHIX0uKc/EVIDENCE/ac-4.log)
- [llm-judge-a-paddles-owned-conversation-thread-layer-exists-above-the-recorder-boundary-and-owns-the-thread-dtos-needed-by-runtime-and-ux-code.txt](../../../../stories/VFHIX0uKc/EVIDENCE/llm-judge-a-paddles-owned-conversation-thread-layer-exists-above-the-recorder-boundary-and-owns-the-thread-dtos-needed-by-runtime-and-ux-code.txt)
- [llm-judge-the-implementation-works-through-the-existing-embedded-recorder-path-does-not-require-a-separate-trace-server-and-remains-extractable-from-paddles-later.txt](../../../../stories/VFHIX0uKc/EVIDENCE/llm-judge-the-implementation-works-through-the-existing-embedded-recorder-path-does-not-require-a-separate-trace-server-and-remains-extractable-from-paddles-later.txt)
- [llm-judge-the-layer-consumes-the-new-upstream-transit-core-metadata-branch-replay-and-artifact-helper-apis-where-they-simplify-low-level-plumbing-without-turning-transit-core-into-the-conversation-api-boundary.txt](../../../../stories/VFHIX0uKc/EVIDENCE/llm-judge-the-layer-consumes-the-new-upstream-transit-core-metadata-branch-replay-and-artifact-helper-apis-where-they-simplify-low-level-plumbing-without-turning-transit-core-into-the-conversation-api-boundary.txt)
- [llm-judge-thread-local-replay-reconstructs-enough-mainline-and-child-thread-provenance-for-later-planning-and-synthesis.txt](../../../../stories/VFHIX0uKc/EVIDENCE/llm-judge-thread-local-replay-reconstructs-enough-mainline-and-child-thread-provenance-for-later-planning-and-synthesis.txt)

### Route Steering Prompts Through Model-Driven Thread Selection
- **ID:** VFHIX1MLt
- **Status:** done

#### Summary
Capture steering prompts during active turns as structured candidates and route
them through the model-driven thread decision loop at safe checkpoints, with
bounded controller validation and honest fail-closed behavior.

#### Acceptance Criteria
- [x] Steering prompts received during an active turn are retained as structured thread candidates instead of opaque queue entries. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Active-turn steering prompts are routed through the model-driven thread decision loop at safe checkpoints instead of being silently appended to opaque queue state. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] Invalid model output or recorder failures degrade through bounded local-first fallback behavior instead of silently mutating thread structure. [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFHIX1MLt/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFHIX1MLt/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFHIX1MLt/EVIDENCE/ac-3.log)
- [llm-judge-active-turn-steering-prompts-are-routed-through-the-model-driven-thread-decision-loop-at-safe-checkpoints-instead-of-being-silently-appended-to-opaque-queue-state.txt](../../../../stories/VFHIX1MLt/EVIDENCE/llm-judge-active-turn-steering-prompts-are-routed-through-the-model-driven-thread-decision-loop-at-safe-checkpoints-instead-of-being-silently-appended-to-opaque-queue-state.txt)
- [llm-judge-invalid-model-output-or-recorder-failures-degrade-through-bounded-local-first-fallback-behavior-instead-of-silently-mutating-thread-structure.txt](../../../../stories/VFHIX1MLt/EVIDENCE/llm-judge-invalid-model-output-or-recorder-failures-degrade-through-bounded-local-first-fallback-behavior-instead-of-silently-mutating-thread-structure.txt)
- [llm-judge-steering-prompts-received-during-an-active-turn-are-retained-as-structured-thread-candidates-instead-of-opaque-queue-entries.txt](../../../../stories/VFHIX1MLt/EVIDENCE/llm-judge-steering-prompts-received-during-an-active-turn-are-retained-as-structured-thread-candidates-instead-of-opaque-queue-entries.txt)

### Document And Prove Auto-Thread Replay Behavior
- **ID:** VFHIX1vLx
- **Status:** done

#### Summary
Update the foundational documentation and produce proof artifacts that show how
thread creation, replay, and merge-back behave, including the current
concurrency limits and how explicit transit lineage keeps the behavior
replayable.

#### Acceptance Criteria
- [x] Foundational docs explain the thread decision contract, transit lineage mapping, merge-back semantics, and the remaining concurrency limits honestly. [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end, proof: ac-1.log-->
- [x] Proof artifacts demonstrate thread split, replay, and merge-back behavior in a way that makes regressions easy to spot. [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end, proof: ac-2.log-->
- [x] Operator-facing guidance remains concise even though the underlying thread lineage becomes richer. [SRS-NFR-03/AC-03] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFHIX1vLx/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFHIX1vLx/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFHIX1vLx/EVIDENCE/ac-3.log)
- [llm-judge-foundational-docs-explain-the-thread-decision-contract-transit-lineage-mapping-merge-back-semantics-and-the-remaining-concurrency-limits-honestly.txt](../../../../stories/VFHIX1vLx/EVIDENCE/llm-judge-foundational-docs-explain-the-thread-decision-contract-transit-lineage-mapping-merge-back-semantics-and-the-remaining-concurrency-limits-honestly.txt)
- [llm-judge-operator-facing-guidance-remains-concise-even-though-the-underlying-thread-lineage-becomes-richer.txt](../../../../stories/VFHIX1vLx/EVIDENCE/llm-judge-operator-facing-guidance-remains-concise-even-though-the-underlying-thread-lineage-becomes-richer.txt)
- [llm-judge-proof-artifacts-demonstrate-thread-split-replay-and-merge-back-behavior-in-a-way-that-makes-regressions-easy-to-spot.txt](../../../../stories/VFHIX1vLx/EVIDENCE/llm-judge-proof-artifacts-demonstrate-thread-split-replay-and-merge-back-behavior-in-a-way-that-makes-regressions-easy-to-spot.txt)

### Render Threaded Transcript And Merge-Back UX
- **ID:** VFHIX2LNJ
- **Status:** done

#### Summary
Extend the default transcript so operators can see when a steering prompt stays
on the mainline, opens a child thread, or merges back, without turning the TUI
into a raw recorder dump.

#### Acceptance Criteria
- [x] The default transcript surfaces thread split, active-thread state, and merge-back outcomes clearly enough to follow live. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Thread-local and mainline context remain visually distinguishable without overwhelming the transcript. [SRS-NFR-03/AC-02] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-2.log-->
- [x] Merge-back rendering uses explicit recorded outcomes instead of implying hidden history rewrites. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFHIX2LNJ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFHIX2LNJ/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFHIX2LNJ/EVIDENCE/ac-3.log)
- [llm-judge-merge-back-rendering-uses-explicit-recorded-outcomes-instead-of-implying-hidden-history-rewrites.txt](../../../../stories/VFHIX2LNJ/EVIDENCE/llm-judge-merge-back-rendering-uses-explicit-recorded-outcomes-instead-of-implying-hidden-history-rewrites.txt)
- [llm-judge-the-default-transcript-surfaces-thread-split-active-thread-state-and-merge-back-outcomes-clearly-enough-to-follow-live.txt](../../../../stories/VFHIX2LNJ/EVIDENCE/llm-judge-the-default-transcript-surfaces-thread-split-active-thread-state-and-merge-back-outcomes-clearly-enough-to-follow-live.txt)
- [llm-judge-thread-local-and-mainline-context-remain-visually-distinguishable-without-overwhelming-the-transcript.txt](../../../../stories/VFHIX2LNJ/EVIDENCE/llm-judge-thread-local-and-mainline-context-remain-visually-distinguishable-without-overwhelming-the-transcript.txt)


