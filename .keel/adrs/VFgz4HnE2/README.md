---
# system-managed
id: VFgz4HnE2
index: 2
status: proposed
decided_at: 2026-04-03T16:30:00
supersedes: []
superseded-by: null
# authored
title: Model Harness Runtime As An Engine With A Governor
context: null
applies-to: []
---

# Model Harness Runtime As An Engine With A Governor

## Status

**Proposed** — This ADR records the current architectural direction for the
runtime harness vocabulary and supervision model.

## Context

`paddles` already had strong local concepts for planning, evidence gathering,
instruction obligations, authored responses, rendering, transcript projection,
and steering-signal visualization. What it lacked was a first-class vocabulary
for the runtime as a whole.

That gap showed up in three ways:

1. UI surfaces had to infer "what the system is doing" from incidental turn
   events instead of consuming one typed supervisory state.
2. Timeout, stall, and intervention semantics were scattered across local
   timers and presentation heuristics instead of belonging to one explicit
   supervisory layer.
3. The term "engine" was already the most accurate mental model in operator
   conversation, but the codebase did not yet encode it directly.

## Decision

The turn-processing runtime is modeled as an **engine** composed of typed
**chambers**, supervised by a **governor**.

### Chambers

The runtime exposes the currently active chamber as part of typed harness
state:

- `interpretation`
- `routing`
- `planning`
- `gathering`
- `tooling`
- `threading`
- `rendering`
- `governor`

Additional chambers may be introduced later as the engine grows, but UI and
operator surfaces should project chamber ownership from typed state rather than
guessing it from unrelated event text.

### Governor

The governor supervises the pace and health of the engine:

- owns timeout phase (`nominal`, `slow`, `stalled`, `expired`)
- owns intervention state
- provides one typed surface for "the engine is still healthy" vs "the engine
  is spinning or blocked"

### Harness Manifold

The engine state is projected through typed `harness_state` events derived from
ordinary turn events. These events become the shared manifold for TUI, web, and
future API clients.

The harness manifold carries:

- active chamber
- governor status
- timeout state
- optional intervention/detail text

## Consequences

## Constraints

- The engine/governor vocabulary must remain a top-level orchestration model and
  must not erase precise lower-level contracts such as `RenderDocument`,
  `InstructionFrame`, or `WorkspaceAction`.
- Supervisory state must be derived from typed runtime events rather than
  reconstructed from surface-specific string heuristics.
- New chambers and governor states must remain serializable across TUI, web, and
  future API projections without introducing presentation-only variants.

### Positive

- TUI and web can project one explicit runtime state instead of maintaining
  separate inference heuristics.
- Timeout/stall language becomes part of the domain model instead of implicit UI
  behavior.
- Future work on a richer supervisory manifold has a clear seam to extend.

### Negative

- Runtime events now include a derived supervisory layer that must stay in sync
  with chamber ownership semantics.
- Some existing in-flight labels and event renderers need to be re-grounded on
  harness state rather than historical heuristics.

### Neutral

- This ADR does not replace precise lower-level terms such as `RenderDocument`,
  `InstructionFrame`, or `WorkspaceAction`. "Engine" is the top-level runtime
  vocabulary; precise domain contracts remain underneath it.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| `TurnEvent::HarnessState` exists and serializes with `harness_state` | automated | Domain test validates the event key |
| Real turns emit planning and rendering harness states | automated | Application test verifies derived harness events beside ordinary turn events |
| TUI renders governor chamber summaries | automated | TUI test verifies `format_turn_event_row(...)` for `HarnessState` |
| Web renders governor event rows | automated | Vitest verifies `eventRow(...)` for `harness_state` |
