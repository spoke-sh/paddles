# Planner Reasoning Events And TUI Rendering - Software Design Description

> Surface planner loop reasoning at each verbosity tier

**SRS:** [SRS.md](SRS.md)

## Overview

The recursive planner loop in `execute_recursive_planner_loop` emits events through `StructuredTurnTrace`. Today these events are either too low-level (verbose=1+) or absent (verbose=0). This design adds a new `PlannerStepProgress` event for live step tracking at verbose=0, enriches existing event rendering at verbose=1, and expands PlannerSummary at verbose=2.

## Context & Boundaries

```
┌─────────────────────────────────────────────────────────┐
│         execute_recursive_planner_loop                  │
│                                                         │
│  for each step:                                         │
│    ├─ trace.emit(PlannerStepProgress)    ← NEW (v=0)   │
│    ├─ trace.emit(PlannerActionSelected)  existing (v=1) │
│    ├─ execute action (search/inspect/tool/branch/stop)  │
│    │   └─ GathererSearchProgress         existing (v=0) │
│    │   └─ GathererSummary + evidence     existing (v=1) │
│    └─ loop_state updated                                │
│                                                         │
│  on exit:                                               │
│    └─ trace.emit(PlannerSummary)         enriched (v=1) │
└─────────────────────────────────────────────────────────┘
         ↓
    TUI event sink
         ↓
    ┌─────────────────────────────────────┐
    │ verbose=0: "Step 2/5: search — X"  │ ← in-place row
    │ verbose=1: + rationale, evidence    │ ← discrete rows
    │ verbose=2: + graph topology, budget │ ← expanded rows
    └─────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| TurnEvent enum | Internal | New variant PlannerStepProgress | domain/model/turns.rs |
| StructuredTurnTrace | Internal | Event emission in planner loop | application/mod.rs |
| format_turn_event_row | Internal | TUI rendering per event | interactive_tui.rs |
| search_progress_row | Internal | In-place row update mechanism | interactive_tui.rs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| New event vs enriching existing | New PlannerStepProgress at v=0, enrich existing at v=1+ | PlannerActionSelected is v=1 and carries different semantics (decision detail, not progress) |
| In-place progress coexistence | Planner progress and search progress are independent rows | Search progress is within a step; they naturally interleave |
| Progress row tracking | Separate planner_progress_row field alongside search_progress_row | Both use the same in-place mechanism but track independently |
| Budget in progress event | Include step_number/step_limit and evidence_count in PlannerStepProgress | Minimal cost, enables budget display at all verbosity levels |

## Components

### TurnEvent::PlannerStepProgress (domain/model/turns.rs)

New variant:
```rust
PlannerStepProgress {
    step_number: usize,      // 1-based current step
    step_limit: usize,       // max steps configured
    action: String,          // "search", "inspect", "read", "refine", "branch", "stop"
    query: Option<String>,   // target query or command, if applicable
    evidence_count: usize,   // accumulated evidence items so far
}
```

- `event_type_key()`: `"planner_step_progress"`
- `min_verbosity()`: 0 (always visible)

### Planner loop emission (application/mod.rs)

After the decision is made but before execution begins (so `action` and `query` are known):
```rust
trace.emit(TurnEvent::PlannerStepProgress {
    step_number: loop_state.steps.len() + 1,
    step_limit: budget.max_steps,
    action: action_summary,
    query: action_query,
    evidence_count: loop_state.evidence_items.len(),
});
```

### TUI in-place rendering (interactive_tui.rs)

`format_turn_event_row` for PlannerStepProgress:
- verbose=0: `"• Step 2/5: search — find auth middleware"`
- verbose=1+: `"• Step 2/5: search — find auth middleware [3 evidence items]"`

In-place tracking: new `planner_progress_row: Option<usize>` field. PlannerStepProgress events replace the previous planner progress row. Cleared on non-progress events or turn completion, same pattern as search_progress_row.

### Enriched PlannerActionSelected rendering (interactive_tui.rs)

At verbose=1, improve the existing rendering:
- Current: `"• Selected planner action\nstep N: action\nRationale: ..."`
- New: `"• Planner step N: action — query\nRationale: collapsed_rationale"`

### Enriched PlannerSummary rendering (interactive_tui.rs)

At verbose=2, expand the stats line with graph topology when available:
- Add node count, edge count to the existing stats line
- Show retained artifact paths from the active branch

## Data Flow

1. Planner loop selects next action → `PlannerStepProgress` emitted → TUI updates in-place row
2. Action executes (search) → `GathererSearchProgress` heartbeats → TUI updates search progress row
3. Action completes → `GathererSummary` emitted → TUI shows as discrete row (clears search progress)
4. Next loop iteration → `PlannerStepProgress` emitted → TUI updates planner progress row
5. Loop exits → `PlannerSummary` emitted → TUI shows final summary row (clears planner progress)

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Step limit reached | Budget check in loop | PlannerStepProgress shows final step | Normal loop exit with stop_reason |
| Gatherer failure | Error from gather_context | Step recorded as failed, loop continues | Existing fallback path |
| Action type not mappable to summary string | Unknown action variant | Use "unknown" as action label | Defensive default |
