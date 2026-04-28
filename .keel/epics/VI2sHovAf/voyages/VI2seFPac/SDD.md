# Stream Tool Output And Drop The 1.2k Cap - Software Design Description

> Replace buffered process::Output capture in planner_action_execution.rs with streamed stdout/stderr pipes that fan out to TurnEventSink and the planner request as bytes arrive; remove the trim_for_planner(_, 1_200) cap; raise any planner-bound budget to 32k+ with head+tail truncation; keep raw output uncut in the trace recorder.

**SRS:** [SRS.md](SRS.md)

## Overview

`run_planner_inspect_command` and `run_planner_shell_command` in `src/application/planner_action_execution.rs` (and the underlying `run_background_terminal_command_with_execution_hand_registry` in `src/infrastructure/terminal.rs`) currently spawn a child process, wait for it to finish, capture stdout and stderr into a single `process::Output`, render a combined string, and pass it through `trim_for_planner(&rendered, 1_200)` before returning a `GovernedPlannerCommandSummary` to the planner. Two operator-hostile consequences fall out of this: the TUI shows nothing while the command runs, and any non-trivial output (`cargo build`, `pytest`, `git log`, `grep -r`) is silently truncated to the first ~1.2k characters before the planner sees it.

This voyage replaces the buffered capture with a streamed `tokio::process::Command` that pipes stdout and stderr through `BufReader::lines()` (or equivalent chunking) and forwards each chunk to two destinations as it arrives:

1. **Operator UI** — a new `TurnEvent::ToolOutputChunk { call_id, stream, bytes }` (final shape TBD) so the TUI can render the live stream in the existing tool-output panel without waiting for command completion.
2. **Trace recorder** — every chunk is appended to the trace record for that tool invocation, so forensics holds the full unmodified output.

Once the command exits, the planner-bound summary is assembled from the accumulated buffer. The `trim_for_planner(&rendered, 1_200)` call is removed. If a context-budget cap is needed at all on the planner-bound copy, it is raised to a generous default (32k characters) and uses head+tail truncation with an explicit `…[truncated N bytes]…` marker. Operator-visible output and the trace remain uncut.

The streaming change is local to the planner-action execution and terminal modules. The execution governance contract, command validation (`validate_inspect_command`'s ban on `&&`, `||`, `;`, `>`, `<`), sandbox enforcement, and `GovernedPlannerCommandSummary` shape stay the same — only the body of the summary changes (no aggressive truncation) and a new event type carries chunks during the run.

## Components

- `src/infrastructure/terminal.rs` — replace `Command` + `Output` capture with `tokio::process::Command` piping stdout/stderr; expose a callback for chunk delivery.
- `src/application/planner_action_execution.rs` — wire the chunk callback into the `TurnEventSink` and the trace record; remove the `trim_for_planner(&rendered, 1_200)` call; either drop trimming entirely or relocate it behind a 32k+ head+tail truncation helper.
- `src/domain/model/turns.rs` (or sibling) — add `TurnEvent::ToolOutputChunk` (or extend an existing variant) with `call_id`, `stream` (`stdout`/`stderr`), and chunk bytes.
- TUI rendering (`src/infrastructure/cli/interactive_tui.rs`) — append streamed chunks to the active tool-output panel as they arrive.
- Trace recorder adapters — persist full unmodified chunks; integration tests verify that operator-visible output and the trace record both contain the full stream regardless of the planner-bound budget.

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
