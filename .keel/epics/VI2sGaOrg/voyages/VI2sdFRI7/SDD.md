# Preserve Planner Rationale Through Trace Pipeline - Software Design Description

> Stop overwriting decision.rationale at recursive_control.rs:147-152; route controller-derived signal summaries to a sibling field; verify the model's own rationale flows verbatim into structured trace, forensics, and manifold projections.

**SRS:** [SRS.md](SRS.md)

## Overview

The current `RecursiveControlChamber::execute_recursive_planner_loop` flow at `src/application/recursive_control.rs:121-152` calls the planner, then runs the result through `review_decision_under_signals`, `sanitize_recursive_planner_decision_for_collaboration`, and `merge_instruction_frame_with_edit_signal` — and finally **overwrites** `decision.rationale` with `compile_recursive_paddles_rationale(...)` before recording the planner action in the trace. The model's own rationale is silently discarded; what reaches the forensic and manifold UIs is a controller-authored summary that the model never produced.

This voyage flips that polarity. The planner's `RecursivePlannerDecision` keeps its `rationale` field as the model's verbatim text. A new sibling field — proposed name `controller_signal_summary` (final name TBD during implementation) — carries the controller-derived narrative that `compile_recursive_paddles_rationale` already generates from accumulated evidence and deliberation signals. `TurnEvent::PlannerActionSelected` is widened to carry both fields; downstream forensic, transcript, and manifold projections render them as distinct rows so an operator inspecting a turn can tell the model's reasoning apart from controller annotations at a glance.

The change is local to the application layer and the trace event schema. It does not touch planner adapters, governance, or the planner-action execution path. The controller still computes its signal summary and may still *reject* a decision via existing channels — it simply stops *rewriting* the model's text. This makes good on the README contract that paddles "lets the model reason first" and "keeps every step visible," and is the load-bearing fix that downstream improvements (plan mode, streaming output) build on.

## Components

- `src/application/recursive_control.rs` — remove the assignment to `decision.rationale`; route the compiled controller summary to the new sibling field; preserve all other validation and merge behavior.
- `src/domain/model/turns.rs` (or sibling) — extend `TurnEvent::PlannerActionSelected` with `controller_signal_summary: Option<String>`; emit both fields from the trace site.
- `src/domain/model/read_model/forensics.rs` and `read_model/manifold.rs` — render the model's rationale and the controller summary as separate fields in projections.
- Tests — add coverage that proves model-supplied rationale text is byte-identical between planner output and emitted `TurnEvent`, and that the controller summary appears on the sibling field.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

<!-- Component relationships, layers, modules -->

## Components

<!-- For each major component: purpose, interface, behavior -->

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
