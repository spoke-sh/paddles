---
# system-managed
id: VFYc52uk8
status: done
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T10:53:05
# authored
title: Render TUI Transcript From The Canonical Conversation Plane
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 2
started_at: 2026-04-01T10:22:55
submitted_at: 2026-04-01T10:28:44
completed_at: 2026-04-01T10:53:05
---

# Render TUI Transcript From The Canonical Conversation Plane

## Summary

Move the TUI transcript bootstrap and live update logic onto the canonical conversation transcript plane. The TUI should render the same shared conversation transcript as the web UI instead of relying on local-only append paths or global external trace scraping.

## Acceptance Criteria

- [x] TUI transcript bootstrap uses canonical conversation replay [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Turns entered from the web UI appear in the TUI transcript for the same conversation without restart [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] TUI transcript updates reconcile to the canonical conversation plane instead of relying on local-only transcript append state [SRS-05/AC-03] <!-- verify: rg -n "shared_conversation_session|replay_conversation_transcript|register_transcript_observer|load_transcript|sync_transcript|pending_transcript_sync" /home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs, SRS-05:start:end, proof: ac-3.log-->
