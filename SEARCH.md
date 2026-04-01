# Search And Retrieval

This document is the source of truth for how search works in `paddles`.

## Boundary

`paddles` owns recursive planning.

- It decides when to `search`, `refine`, `branch`, or `stop`.
- It chooses the retrieval mode and strategy for each bounded search action.
- It decides how gathered evidence is used in later planner and synthesizer turns.

`sift` owns retrieval execution.

- It indexes the workspace.
- It executes lexical or hybrid retrieval against local artifacts.
- It ranks results and produces snippets/evidence payloads.
- It emits low-level progress during direct retrieval stages.

`sift` is not a second planner in the active runtime path. `paddles` does not delegate recursive search strategy back into `sift-autonomous`.

## Capabilities

The direct `sift` retrieval backend currently provides:

- local workspace indexing
- lexical and hybrid search execution
- ranked hits with snippets
- stage-level progress during initialization, indexing, embedding, retrieval, and ranking
- evidence bundles returned to the planner/synthesizer boundary with typed metadata

The default direct provider name is `sift-direct`.

## Constraints

The search boundary is intentionally narrow:

- Search only retrieves from the local workspace and attached local context sources.
- Search does not perform its own recursive planning or branch exploration.
- Search does not decide whether another query should run next.
- Search does not mutate workspace files.
- Search progress may have an unknown ETA when the underlying retrieval stage cannot estimate completion yet.

Those constraints are deliberate. They keep planning visible and controller-owned inside `paddles`.

## Provider Selection

Use `--gatherer-provider sift-direct` to select the direct retrieval backend.

The legacy config value `sift-autonomous` is still accepted as an explicit compatibility alias, but it is normalized to `sift-direct` and should not be used for new configuration.

Other gatherer choices:

- `local`: use a distinct local gatherer model
- `context1`: opt into the external experimental boundary

## Operator Expectations

When a planner search step runs, the operator should expect:

1. `paddles` to show the planner-selected query, retrieval mode, and strategy.
2. The gatherer to show direct retrieval stages rather than hidden autonomous planner states.
3. Evidence to come back as search results, not as a second planner trace that makes new decisions on its own.

If search is slow, the likely causes are indexing, embedding, retrieval, or ranking cost in `sift`, not a hidden recursive search loop inside the gatherer.
