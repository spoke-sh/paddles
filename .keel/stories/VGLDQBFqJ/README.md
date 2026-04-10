---
# system-managed
id: VGLDQBFqJ
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T18:28:35
# authored
title: Define Harness Profile Model For Steering And Compaction
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMvU4i
index: 1
started_at: 2026-04-09T18:16:32
completed_at: 2026-04-09T18:28:35
---

# Define Harness Profile Model For Steering And Compaction

## Summary

Define the explicit harness-profile model that controls steering, compaction, and recovery policy. This story should replace hidden provider-shaped heuristics with a versionable profile contract.

## Acceptance Criteria

- [x] The runtime defines explicit harness-profile semantics for steering and compaction instead of relying on untracked provider-specific heuristics [SRS-01/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test harness_profile -- --nocapture', SRS-01:start:end, proof: ac-1.log -->
- [x] Profile selection and downgrade behavior are explicit enough to be surfaced in docs, tests, and trace projections [SRS-01/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test task_root_trace_records_resolved_harness_profile_selection -- --nocapture && cargo test projects_prompt_and_completion_entries_from_trace_replay -- --nocapture && cargo test trace_graph_projection_replays_root_actions_signals_and_branches -- --nocapture && rg -n "Harness Profiles|recursive-structured-v1|prompt-envelope-safe-v1" README.md ARCHITECTURE.md CONFIGURATION.md', SRS-01:start:end, proof: ac-2.log -->
