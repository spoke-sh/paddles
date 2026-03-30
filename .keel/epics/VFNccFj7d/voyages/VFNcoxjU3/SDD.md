# Step Timing Implementation - Software Design Description

> Deliver end-to-end step timing baselines: reservoir, persistence, classification, and colored rendering

**SRS:** [SRS.md](SRS.md)

## Overview

A new `step_timing` infrastructure module provides a `StepTimingReservoir` that accumulates per-event-type duration samples across sessions. The TUI records each step delta, classifies it against historical percentiles, and renders the delta text with pace-appropriate color. The reservoir persists to a JSON file in the user's cache directory.

## Context & Boundaries

```
┌──────────────────────────────────────────────────┐
│                  TUI Event Loop                  │
│                                                  │
│  TurnEvent ──→ ┌──────────────────┐ ──→ render   │
│    (delta)     │ StepTimingReservoir│    (pace)  │
│                └──────────────────┘              │
│                     ↕ flush/load                 │
└──────────────────────────────────────────────────┘
                      ↕
            ~/.cache/paddles/step_timing.json
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| serde / serde_json | Existing crate | Reservoir serialization | Already in Cargo.toml |
| std::collections::BTreeMap | Stdlib | Keyed storage | Stable |
| std::collections::VecDeque | Stdlib | Bounded FIFO window | Stable |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Window size | 50 per key | Large enough for stable percentiles, small enough for fast adaptation |
| Percentile method | Nearest-rank | Simple, no interpolation needed, correct for discrete samples |
| Minimum samples | 5 | Below this, percentiles are too noisy to be useful |
| Persistence format | JSON | Human-readable, already a dependency, trivial to debug |
| Flush timing | After TurnFinished | One write per turn, no UI thread blocking during render |
| Cache path | ~/.cache/paddles/ | XDG convention for derived/regenerable data |

## Components

### StepTimingReservoir

Core data structure. `BTreeMap<String, VecDeque<u64>>` where keys are event type tags and values are duration-in-milliseconds windows.

- `record(key: &str, delta: Duration)` — push delta, evict oldest if at cap
- `percentile(key: &str, p: u8) -> Option<Duration>` — nearest-rank percentile, None if < 5 samples
- `classify(key: &str, delta: Duration) -> Pace` — Fast/Normal/Slow using p50/p85
- `load(path) -> Self` — deserialize from JSON, fallback to empty
- `flush(path)` — serialize to JSON, create dirs if needed

### Pace enum

```rust
enum Pace { Fast, Normal, Slow }
```

Consumed by the rendering layer to select the style for the delta text span.

### TurnEvent::event_type_key()

Returns the serde tag string for each variant (e.g. `"tool_called"`). Implemented as a match on self that mirrors the `#[serde(rename_all = "snake_case")]` tags.

### Palette additions

Three new styles: `pace_fast`, `pace_normal` (alias for existing body style), `pace_slow`.

## Data Flow

1. **Boot**: `StepTimingReservoir::load(cache_path)` → reservoir stored on `InteractiveApp`
2. **TurnEvent arrives**: `handle_message` extracts delta from timing, calls `reservoir.record(event_type_key, delta)`
3. **Render**: `render_row_lines` calls `reservoir.classify(event_type_key, delta)` → selects pace style for the `(+Xs)` span
4. **TurnFinished**: `handle_message` calls `reservoir.flush(cache_path)`

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cache file missing | `fs::read` returns NotFound | Start with empty reservoir | Accumulates fresh data |
| Cache file corrupt | serde_json parse error | Start with empty reservoir | Overwrites on next flush |
| Cache dir missing | `fs::create_dir_all` on flush | Create directory tree | Normal operation |
| Write failure | `fs::write` error | Log warning, continue | Retry on next turn |
