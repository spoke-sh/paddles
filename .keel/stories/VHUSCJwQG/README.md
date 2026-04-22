---
# system-managed
id: VHUSCJwQG
status: backlog
created_at: 2026-04-21T21:19:31
updated_at: 2026-04-21T21:24:11
# authored
title: Extract Chamber-Aligned Turn Services From MechSuitService
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS6H0Kd
index: 1
---

# Extract Chamber-Aligned Turn Services From MechSuitService

## Summary

Split the current monolithic turn service into chamber-aligned application
services or modules so interpretation, routing, recursive control, and
synthesis can change without dragging projection ownership through the same
file.

## Acceptance Criteria

- [ ] Turn orchestration responsibilities are extracted into chamber-aligned application seams rather than remaining concentrated in one monolithic service. [SRS-01/AC-01] <!-- verify: review, SRS-01:start:end -->
- [ ] The remaining top-level service composes those chambers instead of directly owning all recursive-control and projection behavior. [SRS-02/AC-02] <!-- verify: review, SRS-02:start:end -->
