# Search And Retrieval

This document is the source of truth for how search works in `paddles`.

## Boundary

`paddles` owns recursive planning.

- It decides when to `search`, `refine`, `branch`, or `stop`.
- It chooses the retrieval mode, strategy, and optional retriever overrides for each bounded search action.
- It decides how gathered evidence is used in later planner and synthesizer turns.

`sift` owns retrieval execution.

- It indexes the workspace.
- It executes fast `bm25`, `vector`, and structural fuzzy retrieval plans against local artifacts.
- It ranks results and produces snippets/evidence payloads.
- It emits low-level progress during direct retrieval stages.

`sift` is not a second planner in the active runtime path. `paddles` does not delegate recursive search strategy back into the gatherer.

Paddles also owns the authored-workspace boundary around retrieval results.
When the workspace has a root `.gitignore`, that file is treated as the primary
boundary for planner-visible files, gatherer evidence, and local `list_files`
results. If no root `.gitignore` is present, Paddles falls back to a small
generated/vendored denylist so obviously non-authored paths stay out of the
search loop.

## Capabilities

The direct `sift` retrieval backend currently provides:

- local workspace indexing
- `bm25` and vector search execution
- structural fuzzy path lookup through `retrievers=["path-fuzzy"]`
- structural fuzzy definition/snippet lookup through `retrievers=["path-fuzzy","segment-fuzzy"]`
- ranked hits with snippets
- stage-level progress during initialization, indexing, embedding, retrieval, and ranking
- evidence bundles returned to the planner/synthesizer boundary with typed metadata

The default direct provider name is `sift-direct`.

## Constraints

The search boundary is intentionally narrow:

- Search only retrieves from the local workspace and attached local context sources.
- Search results are post-filtered against the authored-workspace boundary before they reach the planner or synthesizer.
- Search does not perform its own recursive planning or branch exploration.
- Search does not decide whether another query should run next.
- Search does not mutate workspace files.
- Search progress may have an unknown ETA when the underlying retrieval stage cannot estimate completion yet.

Those constraints are deliberate. They keep planning visible and controller-owned inside `paddles`.

## Planner Contract

Planner `search` and `refine` actions still use a coarse `strategy`:

- `bm25` for lexical lookup
- `vector` for semantic lookup

They may now also carry an optional `retrievers` array to ask `sift` for structural fuzzy help:

- `["path-fuzzy"]` maps to the upstream `path-hybrid` preset
- `["path-fuzzy", "segment-fuzzy"]` maps to the upstream `page-index-hybrid` preset

Paddles treats those as planner-selected retrieval hints, not as a second autonomous search policy.

## Provider Selection

Use `--gatherer-provider sift-direct` to select the direct retrieval backend.

`sift` is the direct retrieval backend for all planner workspace search and refine calls.

Other gatherer choices:

- `local`: use a distinct local gatherer model
- `context1`: opt into the external experimental boundary

## Operator Expectations

When a planner search step runs, the operator should expect:

1. `paddles` to show the planner-selected query, retrieval mode, strategy, and any explicit retriever overrides.
2. The gatherer to show direct retrieval stages rather than hidden autonomous planner states.
3. Evidence to come back as search results, not as a second planner trace that makes new decisions on its own.

If search is slow, the likely causes are indexing, embedding, retrieval, or ranking cost in `sift`, not a hidden recursive search loop inside the gatherer.
