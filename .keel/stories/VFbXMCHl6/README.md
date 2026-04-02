---
# system-managed
id: VFbXMCHl6
status: done
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T22:14:08
# authored
title: Expose Web Inspector Replay And Live Projection APIs
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 3
started_at: 2026-04-01T22:03:21
completed_at: 2026-04-01T22:14:08
---

# Expose Web Inspector Replay And Live Projection APIs

## Summary

Expose transit-backed forensic data to the browser through replay and live update projection APIs. The web layer should be able to rebuild the inspector from replay and receive provisional/final artifact updates during active turns without treating the DOM as the source of truth.

## Acceptance Criteria

- [x] The application/web layer exposes conversation- or turn-scoped replay for forensic artifacts, lineage edges, and force snapshots [SRS-04/AC-01] <!-- verify: cargo test -q forensic_routes_project_conversation_and_turn_replay_with_lifecycle_states, SRS-04:start:end, proof: ac-1.log-->
- [x] Replay payloads distinguish provisional, superseded, and final artifact states [SRS-04/AC-02] <!-- verify: cargo test -q replay_conversation_forensics_projects_superseded_and_final_records, SRS-04:start:end, proof: ac-2.log-->
- [x] Live updates deliver forensic artifact changes without requiring page reload and remain recoverable through replay [SRS-04/AC-03] <!-- verify: cargo test -q process_prompt_emits_forensic_updates_for_recorded_trace_artifacts, SRS-04:start:end, proof: ac-3.log-->
- [x] Replay is sufficient to rebuild the forensic inspector after missed live updates without UI-local repair heuristics [SRS-NFR-01/AC-04] <!-- verify: cargo test -q projection_marks_replaced_model_call_records_as_superseded, SRS-NFR-01:start:end, proof: ac-4.log-->
