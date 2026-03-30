---
# system-managed
id: VFNyqBbqo
status: icebox
created_at: 2026-03-30T13:47:31
updated_at: 2026-03-30T13:47:31
# authored
title: Move Sift Search To Blocking Thread With Progress Channel
type: feat
operator-signal:
scope: VFNyZ12IX/VFNyo7ahu
index: 1
---

# Move Sift Search To Blocking Thread With Progress Channel

## Summary

Wrap the synchronous `self.sift.search_autonomous()` call in `tokio::task::spawn_blocking` so it doesn't block the async runtime. Set up a `tokio::sync::mpsc` channel between the blocking thread and the async gather_context caller so progress events can flow back. The blocking thread sends periodic elapsed-time heartbeats while sift is working.

## Acceptance Criteria

- [ ] search_autonomous runs inside tokio::task::spawn_blocking [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] An mpsc channel connects the blocking thread to the async caller [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] The blocking thread sends periodic heartbeats (every ~2s) with elapsed time [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
- [ ] The TUI remains responsive during sift search (spinner continues) [SRS-01/AC-04] <!-- verify: manual, SRS-01:start:end -->
- [ ] Search results are returned correctly after spawn_blocking completes [SRS-01/AC-05] <!-- verify: manual, SRS-01:start:end -->
