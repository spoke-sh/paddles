# Transcript-Driven Interactive Terminal - Software Design Description

> Replace the plain interactive loop with a Codex-style TUI that renders styled user, assistant, and action transcript cells while preserving paddles routing and one-shot CLI behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

Replace the legacy `stdin` line loop in `src/main.rs` with a small terminal UI
layer. The new UI will own terminal setup/teardown, transcript state, event
collection, and rendering. The existing application/controller stack remains
authoritative for routing and final response generation.

## Context & Boundaries

The TUI is a presentation boundary around the existing `MechSuitService`.

- In scope: interactive terminal lifecycle, transcript state, row styling, live
  event display, progressive final-answer rendering, and one-shot path
  preservation.
- Out of scope: changing controller semantics, adding true model token
  streaming, or importing Codex’s full application shell.

```
┌──────────────────────────────────────────────┐
│          Interactive TUI Voyage             │
│                                              │
│  ┌──────────────┐  ┌──────────────────────┐ │
│  │ TUI Runtime  │  │ Transcript Renderer  │ │
│  └──────────────┘  └──────────────────────┘ │
│          │                    │              │
│          └──────────┬─────────┘              │
│                     ▼                        │
│          ┌──────────────────────┐            │
│          │ MechSuitService      │            │
│          │ + TurnEvent bridge   │            │
│          └──────────────────────┘            │
└──────────────────────────────────────────────┘
                ↑                    ↑
         [crossterm/ratatui]   [existing paddles runtime]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `crossterm` | Rust crate | Terminal events, raw mode, alternate screen | current crate release |
| `ratatui` | Rust crate | Structured terminal rendering and layout | current crate release |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Terminal stack | `crossterm` + `ratatui` | Matches the Codex architectural shape without importing Codex itself |
| UI architecture | Small paddles-owned transcript app | Keeps maintenance local and bounded |
| One-shot mode | Keep plain stdout path | Avoids breaking scripts and shell usage |
| Assistant rendering | Progressive reveal of finalized response | Improves interaction feel without pretending true model token streaming exists |

## Architecture

The voyage adds a `tui` module subtree that sits beside the application layer.

- `tui::app` owns transcript state, current input, pending turn state, and
  event ingestion.
- `tui::style` centralizes user/assistant/action colors and terminal-background
  adaptation.
- `tui::render` draws transcript rows and composer state.
- `tui::runtime` manages raw mode, alternate screen, input loop, and redraws.
- The application layer emits turn events through a sink implementation that the
  TUI can forward into transcript rows.

## Components

- **Terminal Runtime**
  Purpose: initialize/restore terminal state, run the event loop, and bridge
  keyboard input with asynchronous turn execution.
- **Transcript State**
  Purpose: persist ordered user, assistant, and action/event rows plus any
  pending in-flight response content.
- **Renderer**
  Purpose: draw transcript rows, composer prompt, and status affordances using
  distinct styling.
- **Turn Bridge**
  Purpose: connect existing `TurnEvent` emission and final `process_prompt`
  results to the TUI without changing controller semantics.

## Interfaces

- `TuiApp::submit_prompt(String)` starts an asynchronous paddles turn.
- `TuiApp::push_event(TurnEvent)` appends/updates action rows.
- `TuiApp::push_response_chunk(&str)` progressively reveals the final assistant
  answer in the transcript.
- `run_interactive_tui(service, runtime_config)` replaces the legacy interactive
  `stdin` loop in `src/main.rs`.

## Data Flow

1. User types in the composer and submits.
2. TUI app appends a styled user row and starts a background paddles turn.
3. A TUI-backed turn event sink receives live `TurnEvent`s and appends action
   rows.
4. When `process_prompt` completes, the final response is progressively revealed
   into an assistant row.
5. Transcript remains visible for scrolling/reference; one-shot mode bypasses
   the TUI entirely.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Terminal init fails | `crossterm`/terminal setup returns error | abort interactive mode with error | terminal state stays untouched |
| Background paddles turn fails | async task returns error | append an error/action row in transcript | allow next prompt without restarting app |
| Terminal resize/input noise | resize or unsupported key events | ignore or redraw | continue event loop |
| Early exit/panic | loop break or error path | restore raw mode and screen state in drop/teardown | user regains normal shell |
