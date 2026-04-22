# Canonical Render Truth And Projection Convergence - Software Design Description

> Persist typed authored responses end-to-end and make live/replay stream projections converge on one canonical render/projection contract.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage makes the typed authored response the only canonical answer artifact.
Instead of flattening a `RenderDocument` into prose for durable storage and later
guessing structure back during replay, the completion path persists the typed
response contract directly. Transcript replay and live projection updates then
consume the same response shape, while reducer/version semantics give surfaces a
deterministic way to reconcile missed updates.

## Context & Boundaries

### In Scope

- completion/checkpoint persistence for typed authored responses
- transcript replay hydration from typed persisted response data
- projection update reducer or version contract for stream reconciliation
- convergence tests across live and replayed render state

### Out of Scope

- adapter/tool-loop ownership changes
- broad chamber extraction from application services
- surface redesign beyond adopting the canonical projection contract

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│  synthesizer -> AuthoredResponse -> durable completion record      │
│                                  -> transcript replay              │
│                                  -> projection reducer/version     │
│                                  -> live stream consumers          │
└────────────────────────────────────────────────────────────────────┘
          ↑                                         ↑
      final answer                           replay / stream
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `AuthoredResponse` / `RenderDocument` | internal | Typed final-answer contract that must become the durable source of truth | current |
| Trace recorder / completion checkpoints | internal | Durable storage for completed-turn artifacts | current |
| Transcript replay and projection update paths | internal | Consumers that must read the same persisted response contract | current |
| Existing TUI/web stream clients | internal | Surfaces that will reconcile against the canonical projection contract | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical completion artifact | Persist typed authored response data, not only flattened prose | Eliminates heuristic render reconstruction on replay |
| Replay source | Hydrate transcript render state from the typed completion artifact | Makes replay identical to what the synthesizer produced |
| Stream reconciliation | Introduce reducer/version semantics in the projection contract | Gives surfaces a deterministic stale-state recovery path |
| Test strategy | Compare live emitted projection state with replayed state for the same turn | Captures the actual regression shape behind stream rendering bugs |

## Architecture

1. The final-answer path produces an `AuthoredResponse` with a typed
   `RenderDocument`.
2. Completion recording serializes that typed response contract into the durable
   trace/checkpoint artifact.
3. Transcript replay reads the stored typed response directly and preserves
   metadata such as response mode and citations.
4. Live projection updates expose ordered reducer/version semantics so stream
   clients can reconcile against the same canonical transcript/render state.
5. Convergence tests assert that the live and replayed paths agree for a
   completed turn.

## Components

`StructuredCompletionRecorder`
: Persists the authored response contract and associated metadata into the
durable completion path.

`TranscriptReplayHydrator`
: Rebuilds assistant transcript rows from persisted typed completion data
without reparsing plain text.

`ProjectionReducerContract`
: Defines the ordered live-update contract surfaces use to reconcile state.

`RenderConvergenceTests`
: Exercises both live emission and replay to ensure they resolve to the same
render truth.

## Interfaces

Candidate internal interfaces:

- `record_completion(response: &AuthoredResponse, ...)`
- `replay_conversation_transcript(task_id) -> ConversationTranscript`
- `projection_update_for_* (...) -> versioned/deterministic projection update`

Candidate compatibility expectation:

- existing surfaces may continue to render plain text views from the typed
  render document, but they must not recreate render structure heuristically

## Data Flow

1. A completed turn produces an `AuthoredResponse`.
2. The completion recorder stores the typed response plus metadata.
3. Transcript replay rehydrates assistant rows from that stored response.
4. Live projection updates publish ordered reconciliation signals derived from
   the same authoritative state.
5. Tests compare live and replayed results for equality of render blocks and
   metadata.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Typed response is missing from a completion record | Replay tests or runtime guards fail when hydrating assistant rows | Fail closed to explicit fallback handling and keep the missing-state visible | Repair the completion path before removing legacy reconstruction |
| Stream client misses an update | Version/reducer mismatch during reconciliation | Trigger authoritative replay for the conversation/task | Rebuild state from canonical replay rather than UI-local repair |
| Live and replayed render states diverge | Convergence tests fail | Block the slice until the canonical contract is aligned | Fix the persistence or replay boundary that drifted |
