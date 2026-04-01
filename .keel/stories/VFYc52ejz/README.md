---
# system-managed
id: VFYc52ejz
status: backlog
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T09:28:09
# authored
title: Expose Conversation-Scoped Transcript Replay From The Application Service
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 1
---

# Expose Conversation-Scoped Transcript Replay From The Application Service

## Summary

Add an application-owned transcript replay path for a single conversation identity using durable trace-backed prompt and completion records. This story also pins down how TUI, web, and CLI attach to the same conversation/task identity instead of each inventing separate transcript state.

## Acceptance Criteria

- [ ] Application service can replay a single conversation transcript from durable prompt and completion records [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] Prompt submission from multiple interfaces can target the same conversation identity [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
- [ ] Conversation-scoped replay is sufficient to recover transcript state without global trace scraping [SRS-NFR-03/AC-03] <!-- verify: test, SRS-NFR-03:start:end -->
- [ ] The new replay/attachment path preserves `process_prompt_in_session_with_sink(...)` as the canonical turn execution flow [SRS-NFR-02/AC-04] <!-- verify: review, SRS-NFR-02:start:end -->
