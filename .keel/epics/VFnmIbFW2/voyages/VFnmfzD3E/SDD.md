# Emit and Render Applied Edit Diffs - Software Design Description

> Show applied workspace changes as first-class diff artifacts across the runtime, web UI, and TUI so workspace editor agency is obvious.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes workspace editor activity visible by introducing a shared applied-edit artifact that originates at the `WorkspaceEditor` boundary, flows through runtime events and projection models, and renders as a diff on both the web and TUI surfaces.

## Context & Boundaries

### In Scope

- extending workspace editor results with structured diff payloads
- emitting a runtime applied-edit event or projection artifact
- rendering that artifact in the web runtime stream and TUI transcript stream
- testing the shared artifact contract end to end

### Out of Scope

- provider-specific edit rendering shortcuts
- changing how models author diffs
- adding review or commit workflows beyond making the applied edit visible

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

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `WorkspaceEditor` boundary | Internal port | Canonical source of applied edit results | current repo interface |
| Runtime event and projection models | Internal domain model | Carry applied-edit artifacts to UI surfaces | current repo types |
| Web runtime application | Internal UI | Render applied-edit diff cards or rows | current React/web runtime |
| TUI transcript renderer | Internal UI | Render applied-edit diff output in the terminal stream | current transcript surface |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Diff source of truth | Source diff payloads from workspace editor results, not provider adapters | Keeps visibility aligned with the provider-agnostic execution boundary |
| Shared artifact contract | Use one applied-edit artifact for runtime/projection/web/TUI | Prevents surface drift and makes testing meaningful |
| Synthetic diff support | Synthesize unified diff-like output for `write_file` and `replace_in_file` when no patch text exists | Makes all edit actions visually legible, not just `apply_patch` |

## Architecture

The architecture has three layers:
- workspace execution: `WorkspaceEditor` produces structured edit results
- runtime transport: application flow emits applied-edit artifacts into turn events and projections
- surface rendering: web and TUI consume the same artifact contract and render diff-oriented rows

## Components

- Workspace editor result model: carries path metadata, diff text, and summary information for successful edits.
- Runtime artifact emitter: turns a successful edit result into a projection-friendly applied-edit event.
- Web runtime renderer: shows a diff-oriented artifact in the live stream.
- TUI transcript renderer: shows the same artifact semantics in terminal output.

## Interfaces

- `WorkspaceActionResult` or equivalent result type gains structured edit payload fields.
- Runtime turn events and/or projection records gain an applied-edit representation.
- UI helpers consume the applied-edit payload directly instead of parsing prose summaries.

## Data Flow

1. Planner or controller chooses a workspace editor action.
2. `WorkspaceEditor` executes the action and returns a result with structured diff payload when the edit succeeds.
3. Application flow records that result in runtime events and projections.
4. Web and TUI renderers consume the shared artifact and display the diff inline.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Edit succeeds but diff payload is missing | Artifact/result tests fail | Block rollout and keep generic summary as fallback only during development | Add synthesis for the missing action path |
| Web and TUI render different semantics | Cross-surface contract tests or manual review fail | Treat as drift and reconcile both surfaces to the shared artifact | Update renderers against the same payload contract |
| Large diffs overwhelm the stream | Manual review or UI tests show unusable output | Introduce truncation/collapse behavior without removing file identity or changed lines | Tune presentation while keeping artifact fidelity |
