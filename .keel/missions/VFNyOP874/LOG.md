# Interactive Sift Search Progress - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-30T13:48:23

Decomposed mission into 1 epic (VFNyZ12IX), 1 voyage, and 4 stories. Story 1 unblocks the runtime with spawn_blocking + progress channel. Story 2 adds the TurnEvent. Story 3 renders progress in the TUI. Story 4 documents upstream sift callback requirements. The key insight: even without sift callbacks, we can show elapsed time heartbeats by running the blocking call on a separate thread with a periodic timer. Full progress (phase, file count, ETA) requires upstream sift changes.

## 2026-03-30T15:30:00

Mission completed. All 4 stories delivered across 4 commits:
- TurnEvent::GathererSearchProgress added with phase, elapsed_seconds, detail fields (min_verbosity=0).
- search_autonomous moved to spawn_blocking; Sift wrapped in Arc; tokio interval heartbeat timer sends elapsed seconds every 2s via mpsc channel; async select loop emits progress events.
- TUI progress rows update in-place via search_progress_row index; superseded by GathererSummary on completion.
- ADR-001 documents upstream sift callback requirements: std::sync::mpsc::Sender shape, 5 progress phases, typed phase data, and search_autonomous_with_progress integration seam.
Key decision: used tokio::spawn for the heartbeat timer instead of std::thread inside spawn_blocking to avoid blocking the join on timer sleep.
