---
# system-managed
id: VFOfsYAjc
index: 1
status: proposed
decided_at: 2026-03-30T16:38:28
supersedes: []
superseded-by: null
# authored
title: Upstream Sift Progress Callback Requirements
context: null
applies-to: []
---

# Upstream Sift Progress Callback Requirements

## Status

**Proposed** — Awaiting upstream sift team review and acceptance.

## Context

Paddles wraps `sift::Sift::search_autonomous()` to gather workspace context. This
call blocks for 30-60+ seconds during workspace indexing and multi-step graph search.
Today, paddles can only show elapsed-time heartbeats because sift exposes no progress
reporting mechanism. To display phase-level progress (indexing files, embedding,
planning, retrieving), sift must expose a callback contract that paddles can consume.

Paddles runs sift inside `tokio::task::spawn_blocking`; progress must cross thread
boundaries safely. The callback contract must not impose a tokio dependency on sift
(sift is runtime-agnostic).

## Decision

Sift should expose a `search_autonomous_with_progress` method that accepts a
`std::sync::mpsc::Sender<SearchProgress>` for progress updates.

### Recommended callback shape

```rust
pub type ProgressSender = std::sync::mpsc::Sender<SearchProgress>;

impl Sift {
    /// Existing API — unchanged.
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

### Progress phases

| Phase | Description | When Emitted |
|-------|-------------|--------------|
| `Indexing` | Building or refreshing the workspace file index | Start of search when index is stale |
| `Embedding` | Computing vector embeddings for new/changed files | After indexing, if vector retriever is active |
| `Planning` | Autonomous planner deciding next action | Before each planner step |
| `Retrieving` | Executing a search/retrieval step | During each retriever pass |
| `Completed` | Search finished | Terminal event |

### Phase data

```rust
#[derive(Clone, Debug)]
pub enum SearchProgress {
    Indexing { files_indexed: usize, total_files: Option<usize> },
    Embedding { files_embedded: usize, total_files: Option<usize> },
    Planning { step_index: usize, step_limit: usize, action: Option<String> },
    Retrieving { step_index: usize, query: Option<String> },
    Completed { total_steps: usize, retained_artifacts: usize },
}
```

### Integration seam on paddles side

In `SiftAutonomousGathererAdapter::gather_context`:
1. Create `std::sync::mpsc::channel()`
2. Pass sender to `search_autonomous_with_progress` inside `spawn_blocking`
3. Bridge `std::sync::mpsc::Receiver` to `tokio::sync::mpsc` on the async side
4. Emit `TurnEvent::GathererSearchProgress` with richer phase data
5. Current heartbeat timer becomes fallback when sift doesn't send updates within 2s

## Constraints

- **MUST:** Use `std::sync::mpsc::Sender` (not tokio) to keep sift runtime-agnostic
- **MUST:** Keep `search_autonomous` working without changes (additive API)
- **MUST NOT:** Add tokio as a dependency to the sift crate
- **SHOULD:** Emit progress at most every 2 seconds to avoid overhead

## Consequences

### Positive

- Paddles can display phase-specific progress ("Indexing 42/128 files", "Planning step 3/5")
- Percentage-based progress bars become possible with `total_files` data
- Existing sift callers unaffected

### Negative

- Sift must thread the sender through its internal call stack
- Minor overhead from channel sends at phase boundaries

### Neutral

- `total_files` is `Option` because sift may not know the total upfront during incremental indexing

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Progress events received during search | manual | Run paddles with verbose mode, verify phase events appear |
| No regression without progress sender | automated | Existing sift tests pass without progress parameter |

## References

- Mission VFNyOP874 (Interactive Sift Search Progress)
- Epic VFNyZ12IX (Sift Search Progress In Paddles TUI)
