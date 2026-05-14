---
# system-managed
id: VJZ8QTY4i
status: done
created_at: 2026-05-13T21:30:07
updated_at: 2026-05-13T23:21:14
# authored
title: Retire Runtime Lane Language From Public Surfaces
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 3
started_at: 2026-05-13T23:07:09
submitted_at: 2026-05-13T23:21:08
completed_at: 2026-05-13T23:21:14
---

# Retire Runtime Lane Language From Public Surfaces

## Summary

Retire runtime lane language from public surfaces after the internal rename is
complete. Any remaining old terms must be explicit compatibility aliases or
historical artifacts, not active product vocabulary.

## Acceptance Criteria

- [x] CLI help, TUI/web route copy, docs, and prompt prose no longer present planner, synthesizer, or gatherer as runtime lanes. [SRS-03/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --bin paddles cli_help_presents_turn_phase_flags_and_hides_legacy_lane_aliases && ! rg -n -i "planner lane|synthesizer lane|gatherer lane|runtime lanes|runtime lane|context-gathering lane|recursive planner lane" README.md ARCHITECTURE.md CONFIGURATION.md POLICY.md INSTRUCTIONS.md apps/docs src', SRS-03:start:end, proof: ac-1.log-->
- [x] String scans or targeted tests cover old public phrases such as "planner lane", "synthesizer lane", "gatherer lane", and "runtime lanes". [SRS-03/AC-02] <!-- verify: cargo test --all-targets agent_loop_prompt_vocabulary, SRS-03:start:end, proof: ac-2.log-->
- [x] Retained legacy aliases are documented as migration shims and point to action-selection, final-rendering, or retrieval terminology. [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] Tests prove turn-loop behavior remains covered for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-05/AC-04] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib unified_loop && cargo test --lib execute_planner_gather_step && cargo nextest run action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration direct_gatherer_returns_direct_retrieval_metadata_and_evidence --no-tests pass', SRS-05:start:end, proof: ac-4.log-->
