---
# system-managed
id: VJXwmlSZT
status: done
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T17:06:33
# authored
title: Preserve Edit Obligations And Steering In Unified Loop
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlE718
index: 2
started_at: 2026-05-13T17:03:05
completed_at: 2026-05-13T17:06:33
---

# Preserve Edit Obligations And Steering In Unified Loop

## Summary

Preserve the behavior that made the old initial decision carry extra metadata:
edit obligations, candidate-file hints, bootstrap paths, steering reviews, and
fail-closed recovery.

## Acceptance Criteria

- [x] Known-edit and candidate-file metadata survive on the unified decision envelope and still create an applied-edit instruction frame. [SRS-04/AC-01] <!-- verify: cargo test unified_loop_preserves_known_edit_instruction_frame --lib, SRS-04:start:end, proof: ac-1.log-->
- [x] Commit, review, repository-grounding, and known-edit bootstrap paths still force bounded workspace evidence when a terminal answer would be unsafe. [SRS-04/AC-02] <!-- verify: cargo test unified_loop_preserves_bootstrap_guardrails --lib, SRS-04:start:end, proof: ac-2.log-->
- [x] Invalid replies, unavailable planners, mutation-disabled modes, and unresolved mutation targets still fail closed with typed terminal behavior. [SRS-05/AC-03] <!-- verify: cargo test unified_loop_fail_closed_paths --lib, SRS-05:start:end, proof: ac-3.log-->
- [x] Loop observability still records terminal actions, workspace actions, evidence, and steering reviews after the migration. [SRS-NFR-02/AC-04] <!-- verify: cargo test unified_loop_observability_preserves_agent_actions --lib, SRS-NFR-02:start:end, proof: ac-4.log-->
