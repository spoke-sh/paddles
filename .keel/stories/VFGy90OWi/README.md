---
# system-managed
id: VFGy90OWi
status: backlog
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:05:52
# authored
title: Document And Prove Graph-Mode Gatherer Routing
type: docs
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 4
---

# Document And Prove Graph-Mode Gatherer Routing

## Summary

Update the foundational docs and capture proof artifacts so operators can see
how graph-mode gatherers fit into the recursive harness, how to configure them,
what telemetry to expect, and where the current implementation still stops
short of a fully unified resource graph.

## Acceptance Criteria

- [ ] README and companion architecture/config docs describe graph-mode gatherers as part of the recursive harness rather than as a special-case product feature. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end -->
- [ ] Operator guidance documents explain the config, local-first fallback behavior, and default telemetry for graph-mode gatherers. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] Proof artifacts show at least one graph-mode gatherer trace or before/after comparison against the prior linear-only gatherer behavior. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
