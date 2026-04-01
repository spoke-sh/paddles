---
# system-managed
id: VFYc52ejz
status: done
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T10:28:44
# authored
title: Expose Conversation-Scoped Transcript Replay From The Application Service
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 1
started_at: 2026-04-01T09:31:42
completed_at: 2026-04-01T10:28:44
---

# Expose Conversation-Scoped Transcript Replay From The Application Service

## Summary

Add an application-owned transcript replay path for a single conversation identity using durable trace-backed prompt and completion records. This story also pins down how TUI, web, and CLI attach to the same conversation/task identity instead of each inventing separate transcript state.

## Acceptance Criteria

- [x] Application service can replay a single conversation transcript from durable prompt and completion records [SRS-01/AC-01] <!-- verify: cargo test -q replay_conversation_transcript_projects_prompt_and_completion_records, SRS-01:start:end, proof: ac-1.log-->
- [x] Prompt submission from multiple interfaces can target the same conversation identity [SRS-02/AC-02] <!-- verify: cargo test -q shared_conversation_session_reuses_live_session_state, SRS-02:start:end, proof: ac-2.log-->
- [x] Conversation-scoped replay is sufficient to recover transcript state without global trace scraping [SRS-NFR-03/AC-03] <!-- verify: cargo test -q replay_conversation_transcript_returns_empty_for_known_session_without_trace_records, SRS-NFR-03:start:end, proof: ac-3.log-->
- [x] The new replay/attachment path preserves `process_prompt_in_session_with_sink(...)` as the canonical turn execution flow [SRS-NFR-02/AC-04] <!-- verify: rg -n "process_prompt_with_session_and_sink|process_prompt_in_session_with_sink" /home/alex/workspace/spoke-sh/paddles/src/application/mod.rs /home/alex/workspace/spoke-sh/paddles/src/infrastructure/web/mod.rs /home/alex/workspace/spoke-sh/paddles/src/main.rs, SRS-NFR-02:start:end, proof: ac-4.log-->
