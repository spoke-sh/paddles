# Integrate Sift Graph Search Into The Gatherer Boundary - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-29T09:58:00-07:00

- advanced `sift` to upstream `2020875a3ea8c0c522d0de385e64ab100e94a14f`
- extended the gatherer contract with generic `linear | graph` retrieval mode
- mapped upstream graph episode/frontier/branch/node/edge state into `paddles`-owned planner metadata with stable ids
- routed recursive planner `search` / `refine` work through graph-mode gatherer requests and surfaced graph summaries in the default event stream
- updated foundational docs and added `GRAPH_MODE_GATHERER_PROOF.md`
- verified with `cargo test -q`, `just quality`, and `cargo nextest run`

## 2026-03-29T09:41:14

Mission achieved by local system user 'alex'
