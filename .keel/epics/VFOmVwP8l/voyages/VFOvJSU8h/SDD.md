# Context Pressure Tracking And Events - Software Design Description

> Track context truncation and emit pressure signals as turn events

**SRS:** [SRS.md](SRS.md)

## Overview

Add `ContextPressure` and `PressureTracker` types to the domain model. Instrument truncation points (operator memory, artifact envelopes, thread summaries) to report truncation events to a tracker. Emit `TurnEvent::ContextPressure` after interpretation context assembly. Render in the TUI at verbose=1+.

## Context & Boundaries

```
┌────────────────────────────────────────────────────────────┐
│  Truncation Sites                                          │
│  ┌───────────────┐ ┌──────────────┐ ┌──────────────────┐  │
│  │Agent Memory   │ │  Artifact    │ │ Thread Summary   │  │
│  │ 12k char cap  │ │  Envelope    │ │ 80 char cap      │  │
│  └───────┬───────┘ └──────┬───────┘ └────────┬─────────┘  │
│          │                │                   │            │
│          └────────┬───────┴───────────────────┘            │
│                   ↓                                        │
│          ┌────────────────┐                                │
│          │PressureTracker │ (accumulates factors)          │
│          └───────┬────────┘                                │
│                  ↓                                         │
│     ┌────────────────────────┐                             │
│     │TurnEvent::ContextPressure│                           │
│     └────────────┬───────────┘                             │
│                  ↓                                         │
│        TUI event rendering                                 │
└────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| TurnEvent enum | Internal | New ContextPressure variant | domain/model/turns.rs |
| Agent memory loader | Internal | Truncation instrumentation point | infrastructure/adapters/agent_memory.rs |
| ArtifactEnvelope | Internal | Truncation instrumentation point | paddles-conversation |
| format_turn_event_row | Internal | TUI rendering | infrastructure/cli/interactive_tui.rs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Simple threshold levels | 0=Low, 1-2=Medium, 3-5=High, 6+=Critical | Easy to understand, can refine later with weighted scoring |
| Tracker passed through context assembly | PressureTracker is a mutable accumulator | Lightweight, no allocation overhead when no truncation occurs |
| Informational only | Pressure does not alter turn flow | Observation before intervention — separate concern |

## Components

### ContextPressure (domain/model/context.rs)

```rust
#[derive(Clone, Debug, Serialize)]
pub struct ContextPressure {
    pub level: PressureLevel,
    pub truncation_count: usize,
    pub factors: Vec<PressureFactor>,
}

#[derive(Clone, Debug, Serialize)]
pub enum PressureLevel { Low, Medium, High, Critical }

#[derive(Clone, Debug, Serialize)]
pub enum PressureFactor {
    MemoryTruncated,
    ArtifactTruncated,
    ThreadSummaryTrimmed,
    EvidenceBudgetExhausted,
}
```

### PressureTracker (domain/model/context.rs)

Mutable accumulator that collects `PressureFactor` events and computes `ContextPressure` on finalize.

### TurnEvent::ContextPressure (domain/model/turns.rs)

New variant carrying the `ContextPressure` struct. min_verbosity = 1.

## Data Flow

1. `PressureTracker::new()` created at start of context assembly
2. Truncation sites call `tracker.record(PressureFactor::MemoryTruncated)` etc.
3. After assembly, `tracker.finalize()` returns `ContextPressure`
4. `trace.emit(TurnEvent::ContextPressure { pressure })` emitted
5. TUI renders as "Context pressure: Medium (2 truncations: memory, artifact)"

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Tracker not wired to a truncation site | Missing instrumentation | Pressure underreported (safe default) | Add instrumentation in follow-up |
| Extremely high truncation count | Factor count > 20 | Still reports Critical level | Capped at Critical, no overflow |
