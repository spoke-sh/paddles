# VOYAGE REPORT: Add Adaptive Harness Profiles And Specialist Brains

## Voyage Metadata
- **ID:** VGLDMvU4i
- **Epic:** VGLD4Iesy
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Harness Profile Model For Steering And Compaction
- **ID:** VGLDQBFqJ
- **Status:** done

#### Summary
Define the explicit harness-profile model that controls steering, compaction, and recovery policy. This story should replace hidden provider-shaped heuristics with a versionable profile contract.

#### Acceptance Criteria
- [x] The runtime defines explicit harness-profile semantics for steering and compaction instead of relying on untracked provider-specific heuristics [SRS-01/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test harness_profile -- --nocapture', SRS-01:start:end, proof: ac-1.log -->
- [x] Profile selection and downgrade behavior are explicit enough to be surfaced in docs, tests, and trace projections [SRS-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test task_root_trace_records_resolved_harness_profile_selection -- --nocapture && cargo test projects_prompt_and_completion_entries_from_trace_replay -- --nocapture && cargo test trace_graph_projection_replays_root_actions_signals_and_branches -- --nocapture && rg -n "Harness Profiles|recursive-structured-v1|prompt-envelope-safe-v1" README.md ARCHITECTURE.md CONFIGURATION.md', SRS-01:start:end, proof: ac-2.log -->

### Expose Session-Queryable Context Slices For Adaptive Replay
- **ID:** VGLDQBYqH
- **Status:** done

#### Summary
Expose queryable session slices that adaptive replay and compaction code can use without destructive prompt-only summarization. This story should turn the durable session into a real context object for the harness.

#### Acceptance Criteria
- [x] Session-queryable context slices support adaptive replay, rewind, or compaction-oriented access outside the live prompt window [SRS-02/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test trace_recording::tests:: -- --nocapture && cargo test recent_turn_summaries_prefer_session_context_slice_before_history_or_synth_fallback -- --nocapture', SRS-02:start:end, proof: ac-1.log -->
- [x] Slice semantics are explicit enough that later adaptive-profile work can reuse them without redefining replay behavior [SRS-02/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "query_session_context|AdaptiveReplay|CompactionWindow|Rewind" README.md ARCHITECTURE.md CONFIGURATION.md src/domain/ports/trace_recording.rs src/application/mod.rs', SRS-02:start:end, proof: ac-2.log -->

### Model Optional Specialist Brains Without Breaking The Recursive Planner
- **ID:** VGLDQCIrB
- **Status:** done

#### Summary
Model optional specialist brains as bounded session-scoped capabilities rather than alternate architectures. This story should protect the recursive planner/controller core while allowing future auxiliary brains to plug in cleanly.

#### Acceptance Criteria
- [x] Optional specialist brains plug into the same session and capability contracts instead of bypassing the recursive planner/controller architecture [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test specialist_brain -- --nocapture', SRS-03:start:end, proof: ac-1.log -->
- [x] The design keeps fallback behavior clear when a specialist brain is absent or unsupported for the active profile/model shape [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test planner_requests_include_specialist_brain_runtime_notes -- --nocapture && cargo test planner_requests_include_specialist_brain_fallback_for_prompt_envelope_safe_profiles -- --nocapture && rg -n "session-continuity-v1|Specialist brains|specialist-brain ids|query_session_context" README.md ARCHITECTURE.md CONFIGURATION.md src/application/mod.rs src/infrastructure/harness_profile.rs src/infrastructure/specialist_brains.rs', SRS-03:start:end, proof: ac-2.log -->


