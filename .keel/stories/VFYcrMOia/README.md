---
# system-managed
id: VFYcrMOia
status: done
created_at: 2026-04-01T09:28:48
updated_at: 2026-04-01T10:53:06
# authored
title: Retire Progress-Driven Transcript Repair Paths
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 5
started_at: 2026-04-01T10:22:55
submitted_at: 2026-04-01T10:28:45
completed_at: 2026-04-01T10:53:06
---

# Retire Progress-Driven Transcript Repair Paths

## Summary

Finish the migration by removing the current replay-after-progress and cross-surface transcript repair heuristics once TUI and web both consume the canonical conversation plane. This is where the architecture becomes clean instead of merely functional.

## Acceptance Criteria

- [x] Transcript hydration no longer depends on `synthesis_ready` or similar progress events [SRS-06/AC-01] <!-- verify: rg -n "transcriptEventSource|transcript_update|refreshConversationTranscript|synthesis_ready" /home/alex/workspace/spoke-sh/paddles/src/infrastructure/web/index.html, SRS-06:start:end, proof: ac-1.log-->
- [x] Surface-specific transcript repair paths are removed or retired once the canonical conversation plane is authoritative [SRS-07/AC-02] <!-- verify: ! rg -n "scheduleTranscriptReplay|pending_external_sync|sync_external_transcript|replay_all_traces" /home/alex/workspace/spoke-sh/paddles/src/infrastructure/web/index.html /home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs, SRS-07:start:end, proof: ac-2.log-->
- [x] Cross-surface transcript updates appear without manual page reload, TUI restart, or operator-triggered replay commands [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->
- [x] The migration introduces no new external service or browser build dependency [SRS-NFR-04/AC-04] <!-- verify: git -C /home/alex/workspace/spoke-sh/paddles diff -- Cargo.toml Cargo.lock package.json pnpm-lock.yaml yarn.lock, SRS-NFR-04:start:end, proof: ac-4.log-->
