# Graph-Mode Gatherer Proof

## Summary

Mission `VFGy52oJs` upgrades `paddles` to the upstream `sift` graph runtime and
keeps that capability behind the generic gatherer boundary.

The delivered slice does four concrete things:

1. updates `Cargo.lock` from `sift` `ecae3c46...` to `2020875a...`
2. extends the gatherer planning contract with generic `linear | graph` mode
3. maps upstream graph episode/frontier/branch state into `paddles`-owned typed
   planner metadata with stable ids
4. routes recursive planner `search` / `refine` actions through graph-mode
   gatherer requests and surfaces graph summaries in the default event stream

## Code Proof

- Gatherer config surface:
  - `src/domain/ports/context_gathering.rs`
  - `PlannerConfig.mode: RetrievalMode`
- Graph metadata boundary:
  - `src/domain/ports/context_gathering.rs`
  - `PlannerGraphEpisode`, `PlannerGraphBranch`, `PlannerGraphFrontierEntry`,
    `PlannerGraphNode`, `PlannerGraphEdge`
- Upstream graph-mode execution:
  - `src/infrastructure/adapters/sift_autonomous_gatherer.rs`
  - `AutonomousSearchRequest::with_mode(...)`
- Recursive routing:
  - `src/application/mod.rs`
  - planner `search` / `refine` now lower to
    `PlannerConfig::default().with_mode(RetrievalMode::Graph).with_step_limit(1)`
- Default operator UX:
  - `src/application/mod.rs`
  - `src/infrastructure/cli/interactive_tui.rs`
  - planner summary events now render `mode`, `active branch`, `branches`, and
    `frontier`

## Behavioral Proof

The application-level regression
`application::tests::recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary`
proves the delivered behavior:

- the recursive planner selects a `search` action
- the lowered gather request uses `planning.mode == graph`
- the gathered planner metadata preserves graph state
- the default turn event stream emits a graph-aware planner summary

Expected graph-aware summary shape:

```text
• Reviewed planner trace
  └ strategy=heuristic, mode=graph, turns=1, steps=0, stop=goal-satisfied, active=branch-root, branches=2, frontier=1
```

## Verification

Executed locally:

```bash
cargo test -q
```

Result:

- 71 tests passed
- graph gatherer adapter unit coverage passed
- recursive planner graph-routing application coverage passed

## Current Limits

- Graph-mode retrieval is selected by the internal gatherer planning contract,
  not by a dedicated user-facing CLI flag yet.
- Graph traces remain inline in evidence and event payloads for now.
- The typed metadata leaves room for future external artifact references, but no
  embedded recorder boundary is attached in this mission.
