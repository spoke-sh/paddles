# Search Progress Implementation - Software Design Description

> Unblock the sift search call, emit progress events, and render them in the TUI

**SRS:** [SRS.md](SRS.md)

## Overview

The synchronous `sift.search_autonomous()` call is moved to a blocking thread via `tokio::task::spawn_blocking`. A `tokio::sync::mpsc` channel connects the blocking thread back to the async `gather_context` caller. A timer thread sends periodic heartbeats with elapsed time. The async caller forwards heartbeats as `TurnEvent::GathererSearchProgress` events through the existing event sink, which the TUI renders as in-place updating progress rows.

## Context & Boundaries

```
┌──────────────────────────────────────────────┐
│           gather_context (async)             │
│                                              │
│  spawn_blocking ──→ sift.search_autonomous() │
│       │                                      │
│       ├─ heartbeat timer (2s) ──→ channel    │
│       │                                      │
│  channel rx ──→ TurnEvent::SearchProgress    │
│       │                                      │
│  join handle ──→ AutonomousSearchResponse     │
└──────────────────────────────────────────────┘
         ↓
    TUI event sink ──→ transcript
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| tokio::task::spawn_blocking | Stdlib | Run sift off async runtime | tokio 1.x |
| tokio::sync::mpsc | Stdlib | Progress channel | tokio 1.x |
| sift::search_autonomous | External crate | Blocking search call | git main |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Progress mechanism | Elapsed-time heartbeats from timer thread | Works today without sift changes |
| Channel type | mpsc unbounded | Heartbeats are infrequent (every 2s), no backpressure needed |
| In-place row updates | Replace last progress row instead of accumulating | Avoids transcript clutter during long waits |
| Future callback seam | Channel sender passed to sift when API supports it | Same channel, richer data |

## Components

### spawn_blocking wrapper (sift_autonomous_gatherer.rs)

Replaces the direct synchronous `self.search_autonomous(request)` call. Spawns sift on a blocking thread, starts a heartbeat timer, and selects between heartbeats and completion on the async side.

### Heartbeat timer

A simple loop that sends `SearchProgressHeartbeat { elapsed: Duration, phase: "searching" }` every 2 seconds until the search completes. Runs inside the spawn_blocking closure using a separate std::thread for the timer.

### TurnEvent::GathererSearchProgress

New variant on TurnEvent: `{ phase: String, elapsed_seconds: u64, detail: Option<String> }`. Emitted by the async gather_context caller as heartbeats arrive. min_verbosity = 0.

### TUI in-place progress rendering

In handle_message, when a GathererSearchProgress arrives, replace the last progress row (if any) instead of pushing a new one. When GathererSummary arrives, the progress row is naturally superseded.

## Data Flow

1. User prompt → planner selects search action → `gather_context` called
2. `gather_context` spawns blocking thread with sift call + heartbeat timer
3. Every 2s: heartbeat → channel → async caller → `TurnEvent::GathererSearchProgress` → TUI
4. TUI replaces last progress row with updated elapsed time
5. Sift completes → blocking thread returns → channel closed
6. `gather_context` returns `ContextGatherResult` → `GathererSummary` event replaces progress

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| spawn_blocking panics | JoinError from await | Return anyhow error | Planner loop handles gather failure |
| Sift returns error | Result::Err from search_autonomous | Propagate to caller | Existing fallback path |
| Channel closed early | RecvError on async side | Stop emitting progress, wait for join | Search result still returned |
