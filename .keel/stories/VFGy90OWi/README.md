---
# system-managed
id: VFGy90OWi
status: done
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:40:49
# authored
title: Document And Prove Graph-Mode Gatherer Routing
type: docs
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 4
started_at: 2026-03-29T09:39:59
submitted_at: 2026-03-29T09:40:45
completed_at: 2026-03-29T09:40:49
---

# Document And Prove Graph-Mode Gatherer Routing

## Summary

Update the foundational docs and capture proof artifacts so operators can see
how graph-mode gatherers fit into the recursive harness, how to configure them,
what telemetry to expect, and where the current implementation still stops
short of a fully unified resource graph or durable recorder integration.

## Acceptance Criteria

- [x] README and companion architecture/config docs describe graph-mode gatherers as part of the recursive harness rather than as a special-case product feature. [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [x] Operator guidance documents explain the config, local-first fallback behavior, default telemetry, and future embedded-recorder seam for graph-mode gatherers. [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end -->
- [x] Proof artifacts show at least one graph-mode gatherer trace or before/after comparison against the prior linear-only gatherer behavior. [SRS-07/AC-03] <!-- verify: manual, SRS-07:start:end -->
- [x] The docs note that large graph traces and tool outputs may later move behind artifact references instead of remaining inline forever. [SRS-06/AC-04] <!-- verify: manual, SRS-06:start:end -->
