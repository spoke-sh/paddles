---
# system-managed
id: VFYc53IkA
status: done
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T10:28:45
# authored
title: Separate Transcript Updates From Turn Progress Events
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 4
started_at: 2026-04-01T10:22:55
completed_at: 2026-04-01T10:28:45
---

# Separate Transcript Updates From Turn Progress Events

## Summary

Introduce a dedicated transcript update path so prompt/response visibility no longer depends on `TurnEvent` timing. Progress events stay available for trace/activity rendering, but transcript changes move onto their own signal path and reconciliation flow.

## Acceptance Criteria

- [x] Transcript update delivery is emitted independently of `TurnEvent` progress completion [SRS-03/AC-01] <!-- verify: cargo test -q process_prompt_emits_transcript_updates_for_prompt_and_completion, SRS-03:start:end, proof: ac-1.log-->
- [x] The dedicated transcript update path can notify surfaces without inferring transcript state from progress-event timing [SRS-03/AC-02] <!-- verify: cargo test -q transcript_update_for_current_task_requests_transcript_sync, SRS-03:start:end, proof: ac-2.log-->
