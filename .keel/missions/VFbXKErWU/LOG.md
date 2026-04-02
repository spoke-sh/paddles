# Build Web Forensic Transit Inspector - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-01T21:49:30

Completed story `VFbXMBbl8` to make transit the exact source of truth for model exchanges. Transit now records exact assembled context, redacted provider request envelopes, raw provider responses, and normalized/rendered outputs as coherent forensic artifacts across HTTP and local Sift paths.

## 2026-04-01T22:02:00

Completed story `VFbXMBvl9` to make transit explain why context changed, not just what was exchanged. Trace replay now carries explicit lineage edges across conversation, turn, planner step, model call, artifact, and final output nodes plus force snapshots for context pressure, execution pressure, fallback, compaction, and budget effects with controller-derived contribution estimates.

## 2026-04-01T23:02:00

Completed story `VFbXMCHl6` to expose the forensic projection seam to the browser. The application service now replays conversation- and turn-scoped forensic projections from transit, emits live forensic record updates as dedicated observers, and the web adapter exposes replay plus SSE endpoints for browser reconstruction without page-local repair logic.
