# ADR-001: Upstream Sift Progress Callback Requirements

**Status:** Proposed
**Date:** 2026-03-30
**Mission:** VFNyOP874 (Interactive Sift Search Progress)
**Epic:** VFNyZ12IX

## Context

Paddles wraps `sift::Sift::search_autonomous()` to gather workspace context. This
call blocks for 30-60+ seconds during workspace indexing and multi-step graph search.
Today, paddles can only show elapsed-time heartbeats because sift exposes no progress
reporting mechanism. To display phase-level progress (indexing files, embedding,
planning, retrieving), sift must expose a callback contract that paddles can consume.

## Decision Drivers

- Paddles runs sift inside `tokio::task::spawn_blocking`; progress must cross thread
  boundaries safely.
- The callback contract must not impose a tokio dependency on sift (sift is runtime-agnostic).
- Heartbeats should be infrequent (~2s) to avoid TUI flicker and overhead.
- The contract must be additive (existing callers without callbacks continue to work).

## Recommended Callback Shape

### Option A (Recommended): Channel sender passed by caller

```rust
/// Caller-provided sender for progress updates during autonomous search.
pub type ProgressSender = std::sync::mpsc::Sender<SearchProgress>;

impl Sift {
    pub fn search_autonomous(
        &self,
        request: AutonomousSearchRequest,
    ) -> Result<AutonomousSearchResponse> { /* ... */ }

    /// Search with optional progress reporting.
    pub fn search_autonomous_with_progress(
        &self,
        request: AutonomousSearchRequest,
        progress: ProgressSender,
    ) -> Result<AutonomousSearchResponse> { /* ... */ }
}
```

Using `std::sync::mpsc::Sender` keeps sift runtime-agnostic. The caller (paddles)
creates a `std::sync::mpsc::channel`, passes the sender to sift, and bridges
received messages to tokio channels on the async side.

### Option B: Trait object

```rust
pub trait SearchProgressObserver: Send {
    fn on_progress(&self, progress: SearchProgress);
}
```

More flexible but heavier. Trait objects add vtable overhead and make it harder for
sift to batch updates. Not recommended unless sift needs pluggable observers for
other purposes.

### Option C: Closure

```rust
pub fn search_autonomous_with_progress(
    &self,
    request: AutonomousSearchRequest,
    on_progress: impl Fn(SearchProgress) + Send,
) -> Result<AutonomousSearchResponse> { /* ... */ }
```

Simple but less composable. Closure captures can complicate lifetime management.

## Progress Phases

Sift should report these phases during `search_autonomous`:

| Phase | Description | When Emitted |
|-------|-------------|--------------|
| `Indexing` | Building or refreshing the workspace file index | Start of search when index is stale |
| `Embedding` | Computing vector embeddings for new/changed files | After indexing, if vector retriever is active |
| `Planning` | Autonomous planner deciding next action | Before each planner step |
| `Retrieving` | Executing a search/retrieval step | During each retriever pass |
| `Completed` | Search finished | Terminal event |

## Phase Data

```rust
#[derive(Clone, Debug)]
pub enum SearchProgress {
    Indexing {
        files_indexed: usize,
        total_files: Option<usize>,
    },
    Embedding {
        files_embedded: usize,
        total_files: Option<usize>,
    },
    Planning {
        step_index: usize,
        step_limit: usize,
        action: Option<String>,
    },
    Retrieving {
        step_index: usize,
        query: Option<String>,
    },
    Completed {
        total_steps: usize,
        retained_artifacts: usize,
    },
}
```

- `total_files` is `Option` because sift may not know the total count upfront
  during incremental indexing.
- `action` and `query` are optional to avoid coupling paddles to planner internals.
- `step_index` is 0-based and matches the planner trace step sequence.

## Integration Seam

The integration point is `Sift::search_autonomous`. The recommended approach:

1. **Sift side:** Add `search_autonomous_with_progress(request, progress_sender)` that
   internally calls `progress_sender.send(...)` at each phase transition. The existing
   `search_autonomous` continues to work without changes (no progress reported).

2. **Paddles side:** In `SiftAutonomousGathererAdapter::gather_context`:
   - Create `std::sync::mpsc::channel()`
   - Pass the sender to `search_autonomous_with_progress`
   - Inside `spawn_blocking`, sift sends progress events through the std channel
   - The async side bridges `std::sync::mpsc::Receiver` to `tokio::sync::mpsc` and
     emits `TurnEvent::GathererSearchProgress` with richer phase data
   - The current heartbeat timer becomes a fallback for when sift doesn't send
     updates within the 2s window

3. **Backward compatibility:** Callers that use `search_autonomous` (without progress)
   see no behavior change. The progress channel is purely additive.

## Consequences

- Paddles can display phase-specific progress ("Indexing 42/128 files", "Planning step 3/5")
- No runtime dependency added to sift
- Existing sift callers unaffected
- Future: percentage-based progress bars become possible with `total_files` data
